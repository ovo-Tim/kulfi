pub async fn tcp_bridge(
    port: u16,
    proxy_target: String,
    graceful: kulfi_utils::Graceful,
) -> eyre::Result<()> {
    use eyre::WrapErr;

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .wrap_err_with(|| {
            format!("can not listen to port {port}, is it busy, or you do not have root access?")
        })?;

    println!("Listening on 127.0.0.1:{port}");

    let peer_connections = kulfi_utils::PeerStreamSenders::default();

    loop {
        tokio::select! {
            _ = graceful.cancelled() => {
                tracing::info!("Stopping control server.");
                break;
            }
            val = listener.accept() => {
                let self_endpoint = malai::global_iroh_endpoint().await;
                let graceful_for_handle_connection = graceful.clone();
                let peer_connections = peer_connections.clone();
                let proxy_target = proxy_target.clone();
                match val {
                    Ok((stream, _addr)) => {
                        graceful.spawn(async move { handle_connection(self_endpoint, stream, graceful_for_handle_connection, peer_connections, proxy_target).await });
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
    self_endpoint: iroh::Endpoint,
    stream: tokio::net::TcpStream,
    graceful: kulfi_utils::Graceful,
    peer_connections: kulfi_utils::PeerStreamSenders,
    remote_node_id52: String,
) {
    println!("handling connection from {remote_node_id52}");
    if let Err(e) = kulfi_utils::tcp_to_peer(
        self_endpoint,
        stream,
        &remote_node_id52,
        peer_connections,
        graceful,
    )
    .await
    {
        tracing::error!("failed to proxy http: {e:?}");
    }
}
