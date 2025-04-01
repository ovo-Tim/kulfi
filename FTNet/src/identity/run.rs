use eyre::WrapErr;

impl ftnet::Identity {
    pub async fn run(
        self,
        graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
        id_map: ftnet::identity::IDMap,
        peer_connections: ftnet::identity::PeerConnections,
    ) -> eyre::Result<()> {
        let port = start_fastn(id_map.clone(), graceful_shutdown_rx.clone())
            .await
            .wrap_err_with(|| "failed to start fastn")?;

        {
            id_map.lock().await.push((self.id52.to_string(), port));
        }

        let ep = ftnet::identity::get_endpoint(self.public_key.to_string().as_str())
            .await
            .wrap_err_with(|| "failed to bind to iroh network")?;

        ftnet::server::run(
            ep,
            port,
            self.client_pools.clone(),
            peer_connections,
            graceful_shutdown_rx,
        )
        .await
    }
}

/// launch fastn from the package directory and return the port
async fn start_fastn(
    _id_map: ftnet::identity::IDMap,
    _graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> eyre::Result<u16> {
    // TODO
    Ok(8000)
}
