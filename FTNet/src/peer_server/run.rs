pub async fn run(
    ep: iroh::Endpoint,
    fastn_port: u16,
    client_pools: ftnet::http::client::ConnectionPools,
    peer_connections: ftnet::identity::PeerConnections,
    _graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> eyre::Result<()> {
    loop {
        let peer_connections = peer_connections.clone();
        let conn = match ep.accept().await {
            Some(conn) => conn,
            None => {
                println!("no connection");
                break;
            }
        };
        let client_pools = client_pools.clone();
        tokio::spawn(async move {
            let start = std::time::Instant::now();
            let conn = match conn.await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to convert connecting to connection: {:?}", e);
                    return;
                }
            };
            if let Err(e) = enqueue_connection(
                conn.clone(),
                client_pools.clone(),
                peer_connections,
                fastn_port,
            )
            .await
            {
                eprintln!("failed to enqueue connection: {:?}", e);
                return;
            }
            if let Err(e) = handle_connection(conn, client_pools, fastn_port).await {
                eprintln!("connection error: {:?}", e);
            }
            println!("connection handled in {:?}", start.elapsed());
        });
    }

    ep.close().await;
    Ok(())
}

async fn enqueue_connection(
    conn: iroh::endpoint::Connection,
    client_pools: ftnet::http::client::ConnectionPools,
    peer_connections: ftnet::identity::PeerConnections,
    fastn_port: u16,
) -> eyre::Result<()> {
    let public_key = match conn.remote_node_id() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("can not get remote id: {e:?}");
            return Err(eyre::anyhow!("can not get remote id: {e:?}"));
        }
    };
    let id = data_encoding::BASE32_DNSSEC.encode(public_key.as_bytes());
    let mut map = peer_connections.lock().await;
    if let Some(v) = map.get_mut(&id) {
        if let Err(e) = v.add(conn.clone()) {
            eprintln!("failed to add connection to peer_connections: {e:?}");
            return Err(eyre::anyhow!(
                "failed to add add connection to peer_connections: {e:?}"
            ));
        }
        return Ok(());
    };

    let pool = bb8::Pool::builder()
        .build(ftnet::Identity {
            id52: id.clone(),
            public_key,
            client_pools,
            fastn_port: Some(fastn_port),
        })
        .await?;
    if let Err(e) = pool.add(conn.clone()) {
        eprintln!("failed to add connection to peer_connections: {e:?}");
        return Err(eyre::anyhow!(
            "failed to add add connection to peer_connections: {e:?}"
        ));
    }
    map.insert(id.clone(), pool);

    Ok(())
}

pub async fn handle_connection(
    conn: iroh::endpoint::Connection,
    _client_pools: ftnet::http::client::ConnectionPools,
    fastn_port: u16,
) -> eyre::Result<()> {
    use tokio_stream::StreamExt;

    println!("got connection from: {:?}", conn.remote_node_id());
    let remote_node_id = match conn.remote_node_id() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("could not read remote node id: {e}, closing connection");
            // TODO: is this how we close the connection in error cases or do we send some error
            //       and wait for other side to close the connection?
            let e2 = conn.closed().await;
            println!("connection closed: {e2}");
            // TODO: send another error_code to indicate bad remote node id?
            conn.close(0u8.into(), &[]);
            return Err(eyre::anyhow!("could not read remote node id: {e}"));
        }
    };
    println!("new client: {remote_node_id:?}");
    loop {
        let (mut send, recv) = conn.accept_bi().await?;
        let mut recv =
            tokio_util::codec::FramedRead::new(recv, tokio_util::codec::LinesCodec::new());
        let msg = match recv.next().await {
            Some(v) => v?,
            None => {
                eprintln!("failed to read from incoming connection");
                continue;
            }
        };
        match serde_json::from_str::<ftnet::Protocol>(&msg)? {
            ftnet::Protocol::Quit => {
                if !recv.read_buffer().is_empty() {
                    send.write_all(b"error: quit message should not have payload\n")
                        .await?;
                } else {
                    send.write_all(b"see you later!\n").await?;
                }
                send.finish()?;
                break;
            }
            ftnet::Protocol::Ping => {
                if !recv.read_buffer().is_empty() {
                    send.write_all(b"error: ping message should not have payload\n")
                        .await?;
                    break;
                }
                send.write_all(ftnet::client::PONG).await?;
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
                send.finish()?;
                break;
            }
            ftnet::Protocol::Identity => {
                ftnet::peer_server::http(fastn_port, &mut send, recv).await
            }
            ftnet::Protocol::Http { .. } => todo!(),
            ftnet::Protocol::Socks5 { .. } => todo!(),
            ftnet::Protocol::Tcp { id } => {
                if let Err(e) = ftnet::peer_server::tcp(&remote_node_id, &id, &mut send, recv).await
                {
                    eprintln!("tcp error: {e}");
                    send.finish()?;
                }
            }
        };
        send.finish()?;
    }

    let e = conn.closed().await;
    println!("connection closed by peer: {e}");
    conn.close(0u8.into(), &[]);
    Ok(())
}
