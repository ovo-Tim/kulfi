use eyre::Context;

impl ftn::Identity {
    pub async fn run(
        self,
        _graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
    ) -> eyre::Result<()> {
        let ep = start_endpoint(self.id.as_str())
            .await
            .wrap_err_with(|| "failed to bind to iroh network")?;

        println!(
            "identity::run: ep_id: {}, self.id: {}",
            ep.node_id(),
            self.id
        );

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        Ok(())
    }
}

async fn start_endpoint(id: &str) -> eyre::Result<iroh::Endpoint> {
    let apns = b"hello-world";

    let secret_key = ftn::utils::get_secret(id)
        .wrap_err_with(|| format!("failed to get secret key from keychain for {id}"))?;

    match iroh::Endpoint::builder()
        .discovery_n0()
        .alpns(vec![apns.to_vec()])
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
