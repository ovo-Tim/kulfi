static IROH_ENDPOINT: tokio::sync::OnceCell<iroh::Endpoint> = tokio::sync::OnceCell::const_new();

pub async fn http_bridge(
    port: u16,
    mut graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> eyre::Result<()> {
    use eyre::WrapErr;

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .wrap_err_with(
            || "can not listen to port 80, is it busy, or you do not have root access?",
        )?;

    println!("Listening on http://127.0.0.1:{port}");

    let peer_connections = ftnet_utils::PeerConnections::default();

    loop {
        tokio::select! {
            _ = graceful_shutdown_rx.changed() => {
                tracing::info!("Stopping control server.");
                break;
            }
            val = listener.accept() => {
                let self_endpoint = IROH_ENDPOINT.get_or_init(new_iroh_endpoint).await.clone();
                let graceful_shutdown_rx = graceful_shutdown_rx.clone();
                let peer_connections = peer_connections.clone();
                match val {
                    Ok((stream, _addr)) => {
                        tokio::spawn(async move { handle_connection(self_endpoint, stream, graceful_shutdown_rx, peer_connections).await });
                    },
                    Err(e) => {
                        tracing::error!("failed to accept: {e:?}");
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn handle_connection(
    _self_endpoint: iroh::Endpoint,
    _stream: tokio::net::TcpStream,
    mut _graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
    _peer_connections: ftnet_utils::PeerConnections,
) {
    todo!()
}


async fn new_iroh_endpoint() -> iroh::Endpoint {
    // TODO: read secret key from ENV VAR
    iroh::Endpoint::builder()
        .discovery_n0()
        .discovery_local_network()
        .alpns(vec![ftnet_utils::APNS_IDENTITY.into()])
        .bind()
        .await
        .expect("failed to create iroh Endpoint")
}
