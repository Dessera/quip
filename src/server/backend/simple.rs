use crate::{
    TcError, TcResult,
    request::RequestBody,
    response::{Response, ResponseError},
    server::{
        backend::Backend,
        connection::{Connection, ConnectionStream},
    },
};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream},
    net::{TcpListener, ToSocketAddrs},
    sync::{Mutex, Notify},
};

pub struct SimpleServer {
    pub listener: TcpListener,
    pub users: Arc<Mutex<HashMap<String, Arc<Mutex<VecDeque<Response>>>>>>,
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
}

impl SimpleServer {
    pub(self) async fn serve_inner<RW>(&self, mut stream: ConnectionStream<RW>) -> TcResult<()>
    where
        RW: AsyncBufReadExt + AsyncWriteExt + Unpin,
    {
        let uname = self.serve_unauth(&mut stream).await?;

        let stream = Arc::new(Mutex::new(stream));
        let notify = Arc::new(Notify::new());

        let res = tokio::try_join!(
            self.serve_auth_read(&uname, stream.clone(), notify.clone()),
            self.serve_auth_write(&uname, stream, notify)
        );

        match res {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    async fn serve_unauth<RW>(&self, stream: &mut ConnectionStream<RW>) -> TcResult<String>
    where
        RW: AsyncBufReadExt + AsyncWriteExt + Unpin,
    {
        let uname: String;

        loop {
            let request = match stream.get_request().await {
                Ok(request) => request,
                Err(TcError::Parse(_)) => {
                    stream
                        .write_response(Response::error(None, ResponseError::BadCommand))
                        .await?;
                    continue;
                }
                Err(err) => return Err(err),
            };

            match &request.body {
                RequestBody::Login(name) | RequestBody::SetName(name) => {
                    let msg = name.clone();
                    uname = name.clone();

                    stream
                        .write_response(Response::success(Some(request), msg))
                        .await?;

                    break;
                }
                _ => {
                    stream
                        .write_response(Response::error(None, ResponseError::Unauthorized))
                        .await?;
                }
            }
        }

        let mut users = self.users.lock().await;
        users.insert(uname.clone(), Arc::new(Mutex::new(VecDeque::new())));

        Ok(uname)
    }

    async fn serve_auth_write<RW>(
        &self,
        name: &str,
        stream: Arc<Mutex<ConnectionStream<RW>>>,
        notify: Arc<Notify>,
    ) -> TcResult<()>
    where
        RW: AsyncWriteExt + Unpin,
    {
        let user = {
            let users = self.users.lock().await;
            match users.get(name) {
                Some(curr) => curr.clone(),
                None => return Err(TcError::Unknown(format!("No user named {}", name))),
            }
        };

        loop {
            notify.notified().await;

            let mut user = user.lock().await;
            while !user.is_empty() {
                let resp = match user.pop_front() {
                    Some(resp) => resp,
                    None => continue,
                };

                let mut stream = stream.lock().await;
                stream.write_response(resp).await?;
            }
        }
    }

    async fn serve_auth_read<RW>(
        &self,
        name: &str,
        _stream: Arc<Mutex<ConnectionStream<RW>>>,
        notify: Arc<Notify>,
    ) -> TcResult<()>
    where
        RW: AsyncBufReadExt + Unpin,
    {
        let user = {
            let users = self.users.lock().await;
            match users.get(name) {
                Some(curr) => curr.clone(),
                None => return Err(TcError::Unknown(format!("No user named {}", name))),
            }
        };

        loop {
            let mut user = user.lock().await;
            user.push_back(Response::success(None, "Hello".to_string()));
            notify.notify_one();

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

impl Backend for SimpleServer {
    async fn accept(&self) -> TcResult<Connection> {
        let (socket, addr) = self.listener.accept().await?;
        Ok(Connection::new(socket, addr))
    }

    async fn serve(&self, mut conn: Connection) -> TcResult<()> {
        let stream = ConnectionStream::new(BufStream::new(&mut conn.socket));

        match self.serve_inner(stream).await {
            Err(TcError::Disconnect) => Ok(()),
            res => res,
        }
    }
}
