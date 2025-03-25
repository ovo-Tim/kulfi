use eyre::WrapErr;

impl ftn::Identity {
    pub async fn run(
        self,
        _graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
    ) -> eyre::Result<()> {
        let port = start_fastn()
            .await
            .wrap_err_with(|| "failed to start fastn")?;

        let ep = ftn::identity::get_endpoint(self.public_key.to_string().as_str())
            .await
            .wrap_err_with(|| "failed to bind to iroh network")?;

        ftn::server::run(ep, port).await
    }
}

/// launch fastn from the package directory and return the port
async fn start_fastn() -> eyre::Result<u16> {
    // TODO
    Ok(0)
}
