pub async fn http_proxy(
    port: u16,
    remote: String,
    graceful: kulfi_utils::Graceful,
    post_start: impl FnOnce(u16) -> eyre::Result<()>,
) -> eyre::Result<()> {
    use eyre::WrapErr;

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .wrap_err_with(|| {
            format!("can not listen on port {port}, is it busy, or you do not have root access?")
        })?;

    // because the caller can pass the port as 0 if they want to bind to a random port
    let port = listener.local_addr()?.port();

    post_start(port)?;

    println!("Listening on http://127.0.0.1:{port}");

    let peer_connections = kulfi_utils::PeerStreamSenders::default();

    let mut graceful_mut = graceful.clone();
    loop {
        tokio::select! {
            () = graceful.cancelled() => {
                tracing::info!("Stopping control server.");
                break;
            }
            r = graceful_mut.show_info() => {
                match r {
                    Ok(_) => {
                        println!("Listening on http://127.0.0.1:{port}");
                        println!("Press ctrl+c again to exit.");
                    }
                    Err(e) => {
                        tracing::error!("failed to show info: {e:?}");
                    }
                }
            }
            r = listener.accept() => {
                match r {
                    Ok((stream, _addr)) => {
                        tracing::info!("got connection");
                        let graceful_for_handle_connection = graceful.clone();
                        let peer_connections = peer_connections.clone();
                        let remote = remote.clone();
                        graceful.spawn(async move {
                            let self_endpoint = malai::global_iroh_endpoint().await;
                            handle_connection(
                                self_endpoint,
                                stream,
                                graceful_for_handle_connection,
                                peer_connections,
                                remote,
                            )
                            .await
                        });
                    }
                    Err(e) => {
                        tracing::error!("failed to accept: {e:?}");
                        break;
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
    remote: String,
) {
    let io = hyper_util::rt::TokioIo::new(stream);

    let conn = hyper::server::conn::http1::Builder::new().serve_connection(
        io,
        hyper::service::service_fn(|r| {
            handle_request(
                r,
                self_endpoint.clone(),
                peer_connections.clone(),
                remote.clone(),
                graceful.clone(),
            )
        }),
    );

    let mut conn = conn.with_upgrades();
    let mut conn = std::pin::Pin::new(&mut conn);

    if let Err(e) = tokio::select! {
        _ = graceful.cancelled() => {
            conn.as_mut().graceful_shutdown();
            conn.await
        }
        r = &mut conn => r,
    } {
        tracing::error!("connection error2: {e:?}");
    }
}

async fn handle_request(
    r: hyper::Request<hyper::body::Incoming>,
    self_endpoint: iroh::Endpoint,
    peer_connections: kulfi_utils::PeerStreamSenders,
    remote: String,
    graceful: kulfi_utils::Graceful,
) -> kulfi_utils::http::ProxyResult {
    tracing::info!("got request for {remote}");

    kulfi_utils::http_to_peer(
        kulfi_utils::ProtocolHeader {
            protocol: kulfi_utils::Protocol::HttpProxy,
            extra: None, // TODO: add extra
        },
        kulfi_utils::http::incoming_to_bytes(r).await?,
        self_endpoint,
        &remote,
        peer_connections,
        Default::default(), /* RequestPatch */
        graceful,
    )
    .await
}
