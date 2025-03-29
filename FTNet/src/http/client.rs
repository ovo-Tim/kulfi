pub struct Client {
    port: u16,
}

impl Client {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn connect(
        &self,
    ) -> eyre::Result<hyper::client::conn::http1::SendRequest<hyper::body::Incoming>> {
        use eyre::WrapErr;

        let addr = format!("127.0.0.1:{}", self.port);
        let stream = tokio::net::TcpStream::connect(addr)
            .await
            .wrap_err_with(|| "failed to open tcp connection")?;
        let io = hyper_util::rt::TokioIo::new(stream);

        let (sender, conn) = hyper::client::conn::http1::handshake(io)
            .await
            .wrap_err_with(|| "failed to do http1 handhsake")?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await.wrap_err_with(|| "connection failed") {
                eprintln!("Connection failed: {err:?}");
            }
        });

        Ok(sender)
    }
}

impl bb8::ManageConnection for Client {
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
