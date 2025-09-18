use crate::{
    TcError, TcResult,
    request::RequestBody,
    response::{Response, ResponseError},
    server::{
        backend::Backend,
        connection::{Connection, ConnectionReader, ConnectionWriter},
        queue::MessageQueue,
    },
};
use log::{info, warn};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::{TcpListener, ToSocketAddrs},
    sync::{Mutex, Notify},
};

pub struct SimpleServer {
    listener: TcpListener,
    users: Arc<Mutex<HashMap<String, MessageQueue>>>,
}

impl SimpleServer {
    pub async fn bind<T>(addr: T) -> TcResult<Self>
    where
        T: ToSocketAddrs,
    {
        Ok(Self {
            listener: TcpListener::bind(addr).await?,
            users: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Add an user to [SimpleServer::users] list.
    ///
    /// This method will throw [TcError::Duplicate] if user already exists.
    pub async fn create_user(&self, name: &str) -> TcResult<MessageQueue> {
        let mut users = self.users.lock().await;

        if users.contains_key(name) {
            return Err(TcError::Duplicate(format!("User '{}' exists.", name)));
        }

        let user = MessageQueue::new();
        users.insert(name.to_string(), user.clone());

        Ok(user)
    }

    /// Remove an user from [SimpleServer::users].
    pub async fn remove_user(&self, name: &str) {
        let mut users = self.users.lock().await;
        users.remove(name);
    }

    /// Rename an user in [SimpleServer::users].
    ///
    /// This method is safe for messages because it won't destruct queue.
    pub async fn rename_user(&self, original: &str, name: &str) -> TcResult<()> {
        let mut users = self.users.lock().await;
        let user = match users.get(original) {
            Some(user) => user.clone(),
            None => return Err(TcError::NotFound(format!("No user named '{}'.", original))),
        };

        if users.contains_key(name) {
            return Err(TcError::Duplicate(format!("User '{}' exists.", name)));
        }

        users.insert(name.to_string(), user);
        users.remove(original);

        Ok(())
    }
}

impl SimpleServer {
    pub(self) async fn serve_inner(&self, conn: Connection) -> TcResult<()> {
        let (mut rx, mut tx) = conn.socket.into_split();
        let mut reader = ConnectionReader::from_read(&mut rx);
        let mut writer = ConnectionWriter::from_write(&mut tx);

        let uname = self.serve_unauth(&mut reader, &mut writer).await?;
        let user = self.create_user(uname.as_str()).await?;

        let notify = Arc::new(Notify::new());

        let res = tokio::try_join!(
            self.serve_auth_read(uname.as_str(), user.clone(), &mut reader, notify.clone()),
            self.serve_auth_write(user, &mut writer, notify)
        );

        self.remove_user(uname.as_str()).await;

        match res {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    async fn serve_unauth<R, W>(
        &self,
        reader: &mut ConnectionReader<R>,
        writer: &mut ConnectionWriter<W>,
    ) -> TcResult<String>
    where
        R: AsyncBufReadExt + Unpin,
        W: AsyncWriteExt + Unpin,
    {
        loop {
            let request = match reader.get_request().await {
                Ok(request) => request,
                Err(TcError::Parse(_)) => {
                    writer
                        .write_response(Response::error(None, ResponseError::BadCommand))
                        .await?;
                    continue;
                }
                Err(err) => return Err(err),
            };

            match &request.body {
                RequestBody::Login(name) | RequestBody::SetName(name) => {
                    let uname = name.clone();

                    writer
                        .write_response(Response::success(Some(request), Some(uname.clone())))
                        .await?;

                    break Ok(uname);
                }
                RequestBody::Logout => return Err(TcError::Disconnect),
                RequestBody::Nop => {
                    writer
                        .write_response(Response::success(Some(request), None))
                        .await?;
                }

                #[allow(unreachable_patterns)] // TODO: Remove this
                _ => {
                    writer
                        .write_response(Response::error(None, ResponseError::Unauthorized))
                        .await?;
                }
            }
        }
    }

    async fn serve_auth_write<W>(
        &self,
        queue: MessageQueue,
        writer: &mut ConnectionWriter<W>,
        notify: Arc<Notify>,
    ) -> TcResult<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        loop {
            notify.notified().await;
            queue.transmit(writer).await?;
        }
    }

    async fn serve_auth_read<R>(
        &self,
        name: &str,
        queue: MessageQueue,
        reader: &mut ConnectionReader<R>,
        notify: Arc<Notify>,
    ) -> TcResult<()>
    where
        R: AsyncBufReadExt + Unpin,
    {
        loop {
            let request = {
                match reader.get_request().await {
                    Ok(request) => request,
                    Err(TcError::Parse(_)) => {
                        queue
                            .push(Response::error(None, ResponseError::BadCommand))
                            .await;
                        notify.notify_one();
                        continue;
                    }
                    Err(err) => return Err(err),
                }
            };

            match &request.body {
                RequestBody::Login(uname) | RequestBody::SetName(uname) => {
                    let uname_cp = uname.clone();

                    let resp = match self.rename_user(name, uname_cp.as_str()).await {
                        Ok(_) => Response::success(Some(request), Some(uname_cp)),
                        Err(TcError::Duplicate(_)) => {
                            Response::error(Some(request), ResponseError::Duplicate)
                        }
                        Err(TcError::NotFound(_)) => {
                            Response::error(Some(request), ResponseError::NotFound)
                        }
                        Err(err) => return Err(err),
                    };

                    queue.push(resp).await;
                }
                RequestBody::Logout => return Err(TcError::Disconnect),
                RequestBody::Nop => {
                    queue.push(Response::success(Some(request), None)).await;
                }
            }

            notify.notify_one();
        }
    }
}

impl Backend for SimpleServer {
    async fn accept(&self) -> TcResult<Connection> {
        let (socket, addr) = self.listener.accept().await?;
        Ok(Connection::new(socket, addr))
    }

    async fn serve(&self, conn: Connection) -> TcResult<()> {
        let addr = conn.addr;

        info!("Unauth connection {} accepted", addr);
        match self.serve_inner(conn).await {
            Ok(_) | Err(TcError::Disconnect) => {
                info!("Connection {} disconnected", addr);
                Ok(())
            }
            Err(err) => {
                warn!("{}", err);
                Err(err)
            }
        }
    }

    fn address(&self) -> TcResult<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }
}
