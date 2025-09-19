use crate::{
    TcError, TcResult,
    server::{
        backend::Backend,
        connection::{Connection, ConnectionReader, ConnectionWriter},
        service::{auth, unauth},
        user::User,
    },
};
use log::{info, warn};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    sync::Mutex,
};

pub struct SimpleServer {
    listener: TcpListener,
    users: Arc<Mutex<HashMap<String, User>>>,
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
    pub(self) async fn serve_inner(&self, conn: Connection) -> TcResult<()> {
        let (mut rx, mut tx) = conn.socket.into_split();

        let mut reader = ConnectionReader::from_read(&mut rx);
        let mut writer = ConnectionWriter::from_write(&mut tx);

        let user = unauth::serve(self, &mut reader, &mut writer).await?;

        let res = tokio::try_join!(
            auth::serve_read(self, user.clone(), &mut reader),
            auth::serve_write(self, user.clone(), &mut writer)
        );

        self.remove_user(user).await?;

        match res {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
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

    async fn add_user(&self, user: User) -> TcResult<()> {
        let mut users = self.users.lock().await;
        let user_data = user.data.lock().await;

        if users.contains_key(user_data.name.as_str()) {
            return Err(TcError::Duplicate(format!(
                "User '{}' exists.",
                user_data.name
            )));
        }

        users.insert(user_data.name.clone(), user.clone());

        Ok(())
    }

    async fn remove_user(&self, user: User) -> TcResult<()> {
        let mut users = self.users.lock().await;
        let user_data = user.data.lock().await;

        users.remove(user_data.name.as_str());

        Ok(())
    }

    async fn rename_user(&self, original: &str, name: &str) -> TcResult<()> {
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

    async fn find_user(&self, name: &str) -> TcResult<User> {
        let users = self.users.lock().await;
        match users.get(name) {
            Some(user) => Ok(user.clone()),
            None => return Err(TcError::NotFound(format!("No user named '{}'.", name))),
        }
    }
}
