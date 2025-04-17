pub async fn expose_http(host: String, port: u16) -> eyre::Result<()> {
    use eyre::WrapErr;
    use ftnet_utils::SecretStore;

    let id52 = ftnet_utils::read_or_create_key().await?;
    let secret_key = ftnet_utils::KeyringSecretStore::new(id52.clone()).get()?;
    let ep = ftnet_utils::get_endpoint(secret_key)
        .await
        .wrap_err_with(|| "failed to bind to iroh network")?;

    print_id52_info(&host, port, &id52);

    let client_pools = ftnet_utils::HttpConnectionPools::default();

    loop {
        let conn = match ep.accept().await {
            Some(conn) => conn,
            None => {
                tracing::info!("no connection");
                break;
            }
        };
        let client_pools = client_pools.clone();
        let host = host.clone();

        tokio::spawn(async move {
            let start = std::time::Instant::now();
            let conn = match conn.await {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("failed to convert incoming to connection: {:?}", e);
                    return;
                }
            };
            if let Err(e) = handle_connection(conn, client_pools, host, port).await {
                tracing::error!("connection error3: {:?}", e);
            }
            tracing::info!("connection handled in {:?}", start.elapsed());
        });
    }

    ep.close().await;
    Ok(())
}

async fn handle_connection(
    conn: iroh::endpoint::Connection,
    client_pools: ftnet_utils::HttpConnectionPools,
    host: String,
    port: u16,
) -> eyre::Result<()> {
    let remote_id52 = ftnet_utils::get_remote_id52(&conn)
        .await
        .inspect_err(|e| tracing::error!("failed to get remote id: {e:?}"))?;

    tracing::info!("new client: {remote_id52}, waiting for bidirectional stream");
    loop {
        let (mut send, recv, msg) = ftnet_utils::accept_bi(&conn)
            .await
            .inspect_err(|e| tracing::error!("failed to accept bidirectional stream: {e:?}"))?;
        tracing::info!("{remote_id52}: {msg:?}");
        let client_pools = client_pools.clone();
        match msg {
            ftnet_utils::Protocol::Identity => {
                if let Err(e) = ftnet_utils::peer_to_http(
                    &format!("{host}:{port}"),
                    client_pools,
                    &mut send,
                    recv,
                )
                .await
                {
                    tracing::error!("failed to proxy http: {e:?}");
                }
            }
            _ => {
                tracing::error!("unsupported protocol: {msg:?}");
                send.write_all(b"error: unsupported protocol\n").await?;
                break;
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

fn print_id52_info(host: &str, port: u16, id52: &str) {
    use colored::Colorize;

    println!(
        "{} is now serving {}{}:{}",
        "Malai".on_green().black(),
        "http://".yellow(),
        host.yellow(),
        port.to_string().yellow()
    );
    println!("ID52: {}", id52.yellow());
    println!(
        "{} {}{}{}",
        "Public kulfi proxy address:".dimmed(),
        "https://".yellow(),
        id52.yellow(),
        ".kulfi.site".yellow()
    );
}
