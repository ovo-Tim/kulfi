pub async fn tcp_bridge(proxy_target: String, port: u16) -> eyre::Result<()> {
    use eyre::WrapErr;

    let (graceful_shutdown_tx, graceful_shutdown_rx) = tokio::sync::watch::channel(false);

    tokio::spawn(async move { http_bridge_(port, graceful_shutdown_rx, proxy_target).await });

    tokio::signal::ctrl_c()
        .await
        .wrap_err_with(|| "failed to get ctrl-c signal handler")?;

    graceful_shutdown_tx
        .send(true)
        .wrap_err_with(|| "failed to send graceful shutdown signal")?;

    tracing::info!("Stopping HTTP bridge.");

    Ok(())
}

async fn http_bridge_(
    port: u16,
    mut graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
    proxy_target: String,
) -> eyre::Result<()> {
    use eyre::WrapErr;

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .wrap_err_with(
            || "can not listen to port 80, is it busy, or you do not have root access?",
        )?;

    println!("Listening on http://127.0.0.1:{port}");

    let peer_connections = ftnet_utils::PeerStreamSenders::default();

    loop {
        tokio::select! {
            _ = graceful_shutdown_rx.changed() => {
                tracing::info!("Stopping control server.");
                break;
            }
            val = listener.accept() => {
                let self_endpoint = kulfi::global_iroh_endpoint().await;
                let graceful_shutdown_rx = graceful_shutdown_rx.clone();
                let peer_connections = peer_connections.clone();
                let proxy_target = proxy_target.clone();
                match val {
                    Ok((stream, _addr)) => {
                        tokio::spawn(async move { handle_connection(self_endpoint, stream, graceful_shutdown_rx, peer_connections, proxy_target).await });
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
    _graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
    _peer_connections: ftnet_utils::PeerStreamSenders,
    _proxy_target: String,
) {
    todo!()
}
