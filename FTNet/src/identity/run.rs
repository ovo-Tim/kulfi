use eyre::WrapErr;

impl ftnet::Identity {
    pub async fn run(
        self,
        _graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
        id_map: ftnet::identity::IDMap,
        client_pools: ftnet::http::client::ConnectionPools,
    ) -> eyre::Result<()> {
        let port = start_fastn(id_map.clone())
            .await
            .wrap_err_with(|| "failed to start fastn")?;

        {
            id_map.lock().await.push((self.id.to_string(), port));
        }

        let ep = ftnet::identity::get_endpoint(self.public_key.to_string().as_str())
            .await
            .wrap_err_with(|| "failed to bind to iroh network")?;

        ftnet::server::run(ep, port, client_pools).await
    }
}

/// launch fastn from the package directory and return the port
async fn start_fastn(_id_map: ftnet::identity::IDMap) -> eyre::Result<u16> {
    // TODO
    Ok(0)
}
