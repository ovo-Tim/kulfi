pub async fn run(
    ep: iroh::Endpoint,
    fastn_port: u16,
    client_pools: ftnet::http::client::ConnectionPools,
    _peer_connections: ftnet::identity::PeerConnections,
    _graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> eyre::Result<()> {
    loop {
        // let peer_connections = peer_connections.clone();
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
                    tracing::error!("failed to convert connecting to connection: {:?}", e);
                    return;
                }
            };
            // if let Err(e) = enqueue_connection(
            //     conn.clone(),
            //     client_pools.clone(),
            //     peer_connections,
            //     fastn_port,
            // )
            // .await
            // {
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

#[expect(dead_code)]
async fn enqueue_connection(
    conn: iroh::endpoint::Connection,
    client_pools: ftnet::http::client::ConnectionPools,
    peer_connections: ftnet::identity::PeerConnections,
    fastn_port: u16,
) -> eyre::Result<()> {
    let public_key = match conn.remote_node_id() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("can not get remote id: {e:?}");
            return Err(eyre::anyhow!("can not get remote id: {e:?}"));
        }
    };
    let id = ftnet::utils::public_key_to_id52(&public_key);
    let mut map = peer_connections.lock().await;
    if let Some(v) = map.get_mut(&id) {
        if let Err(e) = v.add(conn.clone()) {
            tracing::error!("failed to add connection to peer_connections: {e:?}");
            return Err(eyre::anyhow!(
                "failed to add add connection to peer_connections: {e:?}"
            ));
        }
        return Ok(());
    };

    let pool = bb8::Pool::builder()
        .build(ftnet::PeerIdentity {
            self_id52: id.clone(),
            self_public_key: public_key,
            client_pools,
            peer_public_key: conn
                .remote_node_id()
                .map_err(|e| eyre::anyhow!("could not find remote node id: {e:?}"))?,
            fastn_port,
        })
        .await?;
    if let Err(e) = pool.add(conn.clone()) {
        tracing::error!("failed to add connection to peer_connections: {e:?}");
        return Err(eyre::anyhow!(
            "failed to add add connection to peer_connections: {e:?}"
        ));
    }
    map.insert(id.clone(), pool);

    Ok(())
}

pub async fn handle_connection(
    conn: iroh::endpoint::Connection,
    client_pools: ftnet::http::client::ConnectionPools,
    fastn_port: u16,
) -> eyre::Result<()> {
    use tokio_stream::StreamExt;

    tracing::info!("got connection from: {:?}", conn.remote_node_id());
    let remote_node_id = match conn.remote_node_id() {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("could not read remote node id: {e}, closing connection");
            // TODO: is this how we close the connection in error cases or do we send some error
            //       and wait for other side to close the connection?
            let e2 = conn.closed().await;
            tracing::info!("connection closed: {e2}");
            // TODO: send another error_code to indicate bad remote node id?
            conn.close(0u8.into(), &[]);
            return Err(eyre::anyhow!("could not read remote node id: {e}"));
        }
    };
    let remote_id52 = ftnet::utils::public_key_to_id52(&remote_node_id);
    tracing::info!("new client: {remote_id52}, waiting for bidirectional stream");
    loop {
        let client_pools = client_pools.clone();
        let (mut send, recv) = conn.accept_bi().await?;
        tracing::info!("got bidirectional stream");
        let mut recv = ftnet::utils::frame_reader(recv);
        let msg = match recv.next().await {
            Some(v) => v?,
            None => {
                tracing::error!("failed to read from incoming connection");
                continue;
            }
        };
        let msg = serde_json::from_str::<ftnet::Protocol>(&msg)
            .inspect_err(|e| tracing::error!("json error for {msg}: {e}"))?;
        tracing::info!("{remote_id52}: {msg:?}");
        match msg {
            ftnet::Protocol::Quit => {
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
            ftnet::Protocol::Ping => {
                tracing::info!("got ping");
                if !recv.read_buffer().is_empty() {
                    send.write_all(b"error: ping message should not have payload\n")
                        .await?;
                    break;
                }
                tracing::info!("sending PONG");
                send.write_all(&serde_json::to_vec(&ftnet::client::PONG)?)
                    .await
                    .inspect_err(|e| tracing::error!("failed to write PONG: {e:?}"))?;
                tracing::info!("sent");
            }
            ftnet::Protocol::WhatTimeIsIt => {
                if !recv.read_buffer().is_empty() {
                    send.write_all(b"error: quit message should not have payload\n")
                        .await?;
                } else {
                    let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;

                    send.write_all(format!("{}\n", d.as_nanos()).as_bytes())
                        .await?;
                }
            }
            ftnet::Protocol::Identity => {
                ftnet::peer_server::http(
                    &format!("127.0.0.1:{fastn_port}"),
                    client_pools,
                    &mut send,
                    recv,
                )
                .await
            }
            ftnet::Protocol::Http { .. } => todo!(),
            ftnet::Protocol::Socks5 { .. } => todo!(),
            ftnet::Protocol::Tcp { id } => {
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
