pub async fn run(
    ep: iroh::Endpoint,
    fastn_port: u16,
    client_pools: ftnet_utils::ConnectionPools,
    peer_connections: ftnet_utils::PeerConnections,
    _graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> eyre::Result<()> {
    loop {
        let peer_connections = peer_connections.clone();
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
            if let Err(e) = enqueue_connection(conn.clone(), peer_connections).await {
                tracing::error!("failed to enqueue connection: {:?}", e);
                return;
            }
            if let Err(e) = handle_connection(conn, client_pools, fastn_port).await {
                tracing::error!("connection error3: {:?}", e);
            }
            tracing::info!("connection handled in {:?}", start.elapsed());
        });
    }

    ep.close().await;
    Ok(())
}

async fn enqueue_connection(
    conn: iroh::endpoint::Connection,
    peer_connections: ftnet_utils::PeerConnections,
) -> eyre::Result<()> {
    let public_key = match conn.remote_node_id() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("can not get remote id: {e:?}");
            return Err(eyre::anyhow!("can not get remote id: {e:?}"));
        }
    };
    let id = ftnet_utils::public_key_to_id52(&public_key);
    let mut connections = peer_connections.lock().await;
    connections.insert(id.clone(), conn);

    Ok(())
}

pub async fn handle_connection(
    conn: iroh::endpoint::Connection,
    client_pools: ftnet_utils::ConnectionPools,
    fastn_port: u16,
) -> eyre::Result<()> {
    use ftnet_utils::Protocol;
    use tokio_stream::StreamExt;

    tracing::info!("got connection from: {:?}", conn.remote_node_id());
    let remote_id52 = ftnet_utils::get_remote_id52(&conn)
        .await
        .inspect_err(|e| tracing::error!("failed to get remote id: {e:?}"))?;
    tracing::info!("new client: {remote_id52}, waiting for bidirectional stream");
    loop {
        let client_pools = client_pools.clone();
        let (mut send, recv) = conn.accept_bi().await?;
        tracing::info!("got bidirectional stream");
        let mut recv = ftnet_utils::frame_reader(recv);
        let msg = match recv.next().await {
            Some(v) => v?,
            None => {
                tracing::error!("failed to read from incoming connection");
                continue;
            }
        };
        let msg = serde_json::from_str::<Protocol>(&msg)
            .inspect_err(|e| tracing::error!("json error for {msg}: {e}"))?;
        tracing::info!("{remote_id52}: {msg:?}");
        ftnet_utils::ack(&mut send).await?;
        match msg {
            Protocol::Quit => {
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
            Protocol::Ping => {
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
            Protocol::WhatTimeIsIt => {
                if !recv.read_buffer().is_empty() {
                    send.write_all(b"error: quit message should not have payload\n")
                        .await?;
                } else {
                    let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;

                    send.write_all(format!("{}\n", d.as_nanos()).as_bytes())
                        .await?;
                }
            }
            Protocol::Identity => {
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
            Protocol::Http { .. } => todo!(),
            Protocol::Socks5 { .. } => todo!(),
            Protocol::Tcp { id } => {
                if let Err(e) = ftnet::peer_server::tcp(&remote_id52, &id, &mut send, recv).await {
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
