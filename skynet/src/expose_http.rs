use ftnet_utils::SecretStore;

pub async fn expose_http(host: String, port: u16) -> eyre::Result<()> {
    use eyre::WrapErr;

    let id52 = read_or_create_key().await?;
    let secret_key = ftnet_utils::KeyringSecretStore::new(id52.clone()).get()?;
    let ep = ftnet_utils::get_endpoint(secret_key)
        .await
        .wrap_err_with(|| "failed to bind to iroh network")?;

    println!("Connect to {port} by visiting http://{id52}.localhost.direct", );

    let client_pools = ftnet_utils::ConnectionPools::default();

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

async fn read_or_create_key() -> eyre::Result<String> {
    use ftnet_utils::SecretStore;

    match tokio::fs::read_to_string(".skynet.id52").await {
        Ok(v) => Ok(v),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::info!("no key found, creating new one");
            let public_key = ftnet_utils::KeyringSecretStore::generate(rand::rngs::OsRng)?;
            let public_key = ftnet_utils::public_key_to_id52(&public_key);
            tokio::fs::write(".skynet.id52", public_key.as_str()).await?;
            Ok(public_key)
        }
        Err(e) => {
            tracing::error!("failed to read key: {e}");
            Err(e.into())
        }
    }
}

async fn handle_connection(
    conn: iroh::endpoint::Connection,
    client_pools: ftnet_utils::ConnectionPools,
    host: String,
    port: u16,
) -> eyre::Result<()> {
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
        let msg = serde_json::from_str::<ftnet_utils::Protocol>(&msg)
            .inspect_err(|e| tracing::error!("json error for {msg}: {e}"))?;
        tracing::info!("{remote_id52}: {msg:?}");
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
