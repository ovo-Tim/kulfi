use eyre::WrapErr;

impl ftn::Identity {
    pub async fn run(
        self,
        _graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
    ) -> eyre::Result<()> {
        let _port = start_fastn()
            .await
            .wrap_err_with(|| "failed to start fastn")?;

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        Ok(())
    }
}

/// launch fastn from the package directory and return the port
async fn start_fastn() -> eyre::Result<u16> {
    // TODO
    Ok(0)
}
