pub async fn run(ep: iroh::Endpoint, _fastn_port: u16) -> eyre::Result<()> {
    loop {
        println!("waiting for incoming connection");
        let conn = match ep.accept().await {
            Some(conn) => conn,
            None => {
                println!("no connection");
                break;
            }
        };
        println!("got connection");
        tokio::spawn(async move {
            let start = std::time::Instant::now();
            if let Err(e) = handle_connection(conn).await {
                eprintln!("connection error: {:?}", e);
            }
            println!("connection handled in {:?}", start.elapsed());
        });
    }

    ep.close().await;
    Ok(())
}

async fn handle_connection(conn: iroh::endpoint::Incoming) -> eyre::Result<()> {
    let conn = conn.await?;
    println!("new client: {:?}", conn.remote_node_id());
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
            Ok((ftnet::Protocol::WhatTimeItIs, rest)) => {
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
            Ok((ftnet::Protocol::Tcp { .. }, _)) => todo!(),
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
