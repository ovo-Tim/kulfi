pub type ConnectionPool = bb8::Pool<ConnectionManager>;
pub type ConnectionPools =
    std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, ConnectionPool>>>;

pub struct ConnectionManager {
    addr: String,
}

impl ConnectionManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    pub async fn connect(
        &self,
    ) -> eyre::Result<hyper::client::conn::http1::SendRequest<hyper::body::Incoming>> {
        use eyre::WrapErr;

        let stream = tokio::net::TcpStream::connect(&self.addr)
            .await
            .wrap_err_with(|| "failed to open tcp connection")?;
        let io = hyper_util::rt::TokioIo::new(stream);

        let (sender, conn) = hyper::client::conn::http1::handshake(io)
            .await
            .wrap_err_with(|| "failed to do http1 handshake")?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await.wrap_err_with(|| "connection failed") {
                tracing::error!("Connection failed: {err:?}");
            }
        });

        Ok(sender)
    }
}

impl bb8::ManageConnection for ConnectionManager {
    type Connection = hyper::client::conn::http1::SendRequest<hyper::body::Incoming>;
    type Error = eyre::Error;

    fn connect(&self) -> impl Future<Output = Result<Self::Connection, Self::Error>> + Send {
        Box::pin(async move { self.connect().await })
    }

    fn is_valid(
        &self,
        conn: &mut Self::Connection,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        Box::pin(async {
            if conn.is_closed() {
                return Err(eyre::anyhow!("connection is closed"));
            }

            Ok(())
        })
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.is_closed()
    }
}
