use eyre::WrapErr;

impl bb8::ManageConnection for ftn::Identity {
    type Connection = iroh::endpoint::Connection;
    type Error = eyre::Error;

    fn connect(&self) -> impl Future<Output = Result<Self::Connection, Self::Error>> + Send {
        Box::pin(async move {
            // creating a new endpoint takes about 30 milliseconds, so we can do it here.
            // since we create just a single connection via this endpoint, the overhead is
            // negligible, compared to 800 milliseconds or so it takes to create a new connection.
            let ep = start_endpoint(self.public_key.to_string().as_str())
                .await
                .wrap_err_with(|| "failed to bind to iroh network")?;
            ep.connect(self.public_key, ftn::APNS)
                .await
                .map_err(|e| eyre::anyhow!("failed to connect to iroh network: {e}"))
        })
    }

    fn is_valid(
        &self,
        _conn: &mut Self::Connection,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        // TODO: send a ping and wait for a pong
        Box::pin(async move { Ok(()) })
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

async fn start_endpoint(id: &str) -> eyre::Result<iroh::Endpoint> {
    let secret_key = ftn::utils::get_secret(id)
        .wrap_err_with(|| format!("failed to get secret key from keychain for {id}"))?;

    match iroh::Endpoint::builder()
        .discovery_n0()
        .alpns(vec![ftn::APNS.into()])
        .secret_key(secret_key)
        .bind()
        .await
    {
        Ok(ep) => Ok(ep),
        Err(e) => {
            // https://github.com/n0-computer/iroh/issues/2741
            // this is why you MUST NOT use anyhow::Error etc. in library code.
            Err(eyre::anyhow!("failed to bind to iroh network: {e}"))
        }
    }
}
