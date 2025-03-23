impl ftn::Identity {
    pub async fn run(
        self,
        _graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
    ) -> eyre::Result<()> {
        use eyre::WrapErr;

        let apns = b"hello-world";

        let ep = {
            // using block to limit the scope of the secret_key variable
            let secret_key = ftn::utils::get_secret(self.id.as_str()).wrap_err_with(|| {
                format!("failed to get secret key from keychain for {}", self.id)
            })?;

            match iroh::Endpoint::builder()
                .discovery_n0()
                .alpns(vec![apns.to_vec()])
                .secret_key(secret_key)
                .bind()
                .await
            {
                Ok(ep) => ep,
                Err(e) => {
                    // https://github.com/n0-computer/iroh/issues/2741
                    // this is why you MUST not use anyhow::Error etc. in library code.
                    return Err(eyre::anyhow!("failed to bind to iroh network: {e}"));
                }
            }
            // .wrap_err("failed to bind to iroh network")?
        };

        println!(
            "identity::run: ep_id: {}, self.id: {}",
            ep.node_id(),
            self.id
        );

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        Ok(())
    }
}
