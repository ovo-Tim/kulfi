pub async fn run(
    ep: iroh::Endpoint,
    _fastn_port: u16,
    client_pools: ftnet::http::client::ConnectionPools,
) -> eyre::Result<()> {
    loop {
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
            if let Err(e) = handle_connection(conn, client_pools).await {
                eprintln!("connection error: {:?}", e);
            }
            println!("connection handled in {:?}", start.elapsed());
        });
    }

    ep.close().await;
    Ok(())
}

async fn handle_connection(
    conn: iroh::endpoint::Incoming,
    _client_pools: ftnet::http::client::ConnectionPools,
) -> eyre::Result<()> {
    let conn = conn.await?;
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
        let (mut send_stream, mut recv_stream) = conn.accept_bi().await?;
        let msg = recv_stream.read_to_end(1024).await?;
        match ftnet::Protocol::parse(&msg) {
            Ok((ftnet::Protocol::Quit, rest)) => {
                if !rest.is_empty() {
                    send_stream
                        .write_all(b"error: quit message should not have payload\n")
                        .await?;
                } else {
                    send_stream.write_all(b"see you later!\n").await?;
                }
                send_stream.finish()?;
                break;
            }
            Ok((ftnet::Protocol::Ping, rest)) => {
                if !rest.is_empty() {
                    send_stream
                        .write_all(b"error: ping message should not have payload\n")
                        .await?;
                    break;
                }
                send_stream.write_all(ftnet::client::PONG).await?;
            }
            Ok((ftnet::Protocol::WhatTimeIsIt, rest)) => {
                if !rest.is_empty() {
                    send_stream
                        .write_all(b"error: quit message should not have payload\n")
                        .await?;
                } else {
                    let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;

                    send_stream
                        .write_all(format!("{}\n", d.as_nanos()).as_bytes())
                        .await?;
                }
                send_stream.finish()?;
                break;
            }
            Ok((ftnet::Protocol::Identity, _)) => todo!(),
            Ok((ftnet::Protocol::Http { .. }, _)) => todo!(),
            Ok((ftnet::Protocol::Socks5 { .. }, _)) => todo!(),
            Ok((ftnet::Protocol::Tcp { id }, _)) => {
                if let Err(e) =
                    ftnet::server::tcp(&remote_node_id, &id, &mut send_stream, recv_stream).await
                {
                    eprintln!("tcp error: {e}");
                    send_stream.finish()?;
                }
            }
            Err(e) => {
                eprintln!("error parsing protocol: {e}");
                send_stream.write_all(b"error: invalid protocol\n").await?;
                send_stream.finish()?;
                break;
            }
        };
        send_stream.finish()?;
    }

    let e = conn.closed().await;
    println!("connection closed by peer: {e}");
    conn.close(0u8.into(), &[]);
    Ok(())
}
