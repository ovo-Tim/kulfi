pub async fn run(
    ep: iroh::Endpoint,
    fastn_port: u16,
    client_pools: ftnet_utils::HttpConnectionPools,
    _graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> eyre::Result<()> {
    loop {
        let conn = match ep.accept().await {
            Some(conn) => conn,
            None => {
                tracing::info!("no connection");
                break;
            }
        };
        let client_pools = client_pools.clone();
        tokio::spawn(async move {
            let start = std::time::Instant::now();
            let conn = match conn.await {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("failed to convert incoming to connection: {:?}", e);
                    return;
                }
            };
            // if let Err(e) = enqueue_connection(conn.clone(), peer_connections).await {
            //     tracing::error!("failed to enqueue connection: {:?}", e);
            //     return;
            // }
            if let Err(e) = handle_connection(conn, client_pools, fastn_port).await {
                tracing::error!("connection error3: {:?}", e);
            }
            tracing::info!("connection handled in {:?}", start.elapsed());
        });
    }

    ep.close().await;
    Ok(())
}

// async fn enqueue_connection(
//     conn: iroh::endpoint::Connection,
//     peer_connections: ftnet_utils::get_stream2::PeerStreamSenders,
// ) -> eyre::Result<()> {
//     let public_key = match conn.remote_node_id() {
//         Ok(v) => v,
//         Err(e) => {
//             tracing::error!("can not get remote id: {e:?}");
//             return Err(eyre::anyhow!("can not get remote id: {e:?}"));
//         }
//     };
//     let id = ftnet_utils::public_key_to_id52(&public_key);
//     let mut connections = peer_connections.lock().await;
//     connections.insert(id.clone(), conn);
//
//     Ok(())
// }

pub async fn handle_connection(
    conn: iroh::endpoint::Connection,
    client_pools: ftnet_utils::HttpConnectionPools,
    fastn_port: u16,
) -> eyre::Result<()> {
    tracing::info!("got connection from: {:?}", conn.remote_node_id());
    let remote_id52 = ftnet_utils::get_remote_id52(&conn)
        .await
        .inspect_err(|e| tracing::error!("failed to get remote id: {e:?}"))?;
    tracing::info!("new client: {remote_id52}, waiting for bidirectional stream");
    loop {
        let client_pools = client_pools.clone();
        let (mut send, recv, msg) = ftnet_utils::accept_bi(&conn)
            .await
            .inspect_err(|e| tracing::error!("failed to accept bidirectional stream: {e:?}"))?;
        tracing::info!("{remote_id52}: {msg:?}");
        match msg {
            ftnet_utils::Protocol::Quit => {
                if !recv.read_buffer().is_empty() {
                    send.write_all(b"error: quit message should not have payload\n")
                        .await?;
                } else {
                    send.write_all(b"see you later!\n").await?;
                }
                send.finish()?;
                // quit should close the connection, so we are breaking the for loop.
                break;
            }
            ftnet_utils::Protocol::Ping => {
                tracing::info!("got ping");
                if !recv.read_buffer().is_empty() {
                    send.write_all(b"error: ping message should not have payload\n")
                        .await?;
                    break;
                }
                tracing::info!("sending PONG");
                send.write_all(ftnet::client::PONG)
                    .await
                    .inspect_err(|e| tracing::error!("failed to write PONG: {e:?}"))?;
                tracing::info!("sent");
            }
            ftnet_utils::Protocol::WhatTimeIsIt => {
                if !recv.read_buffer().is_empty() {
                    send.write_all(b"error: quit message should not have payload\n")
                        .await?;
                } else {
                    let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;

                    send.write_all(format!("{}\n", d.as_nanos()).as_bytes())
                        .await?;
                }
            }
            ftnet_utils::Protocol::Identity => {
                if let Err(e) = ftnet_utils::peer_to_http(
                    &format!("127.0.0.1:{fastn_port}"),
                    client_pools,
                    &mut send,
                    recv,
                )
                .await
                {
                    tracing::error!("failed to proxy http: {e:?}");
                }
            }
            ftnet_utils::Protocol::Http { .. } => todo!(),
            ftnet_utils::Protocol::Socks5 { .. } => todo!(),
            ftnet_utils::Protocol::Tcp { id } => {
                if let Err(e) = ftnet_utils::peer_to_tcp(&remote_id52, &id, &mut send, recv).await {
                    tracing::error!("tcp error: {e}");
                }
            }
        };
        tracing::info!("closing send stream");
        send.finish()?;
    }

    let e = conn.closed().await;
    tracing::info!("connection closed by peer: {e}");
    conn.close(0u8.into(), &[]);
    Ok(())
}
