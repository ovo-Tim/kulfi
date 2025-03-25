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
        let (proto, _residue) = match ftn::Protocol::parse(msg) {
            Ok((ftn::Protocol::Quit, _)) => {
                send_stream.finish()?;
                break;
            }
            Ok((proto, msg)) => (proto, msg),
            Err(e) => {
                send_stream.write_all(b"error: invalid protocol\n").await?;
                send_stream.finish()?;
                return Err(e);
            }
        };
        println!("received: {proto:?}");
        send_stream.write_all(b"hello").await?;
        send_stream.finish()?;
    }

    let e = conn.closed().await;
    println!("connection closed by peer: {e}");
    conn.close(0u8.into(), &[]);
    Ok(())
}
