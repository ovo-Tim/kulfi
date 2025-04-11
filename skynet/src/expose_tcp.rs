pub async fn expose_tcp(host: String, port: u16) -> eyre::Result<()> {
    use eyre::WrapErr;
    use ftnet_utils::SecretStore;

    let id52 = ftnet_utils::read_or_create_key().await?;
    let secret_key = ftnet_utils::KeyringSecretStore::new(id52.clone()).get()?;
    let ep = ftnet_utils::get_endpoint(secret_key)
        .await
        .wrap_err_with(|| "failed to bind to iroh network")?;

    println!(
        "Connect to {port} by running `skynet tcp-bridge {id52} <some-port>` from any machine.",
    );

    loop {
        let conn = match ep.accept().await {
            Some(conn) => conn,
            None => {
                tracing::info!("no connection");
                break;
            }
        };
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
            if let Err(e) = handle_connection(conn, host, port).await {
                tracing::error!("connection error3: {:?}", e);
            }
            tracing::info!("connection handled in {:?}", start.elapsed());
        });
    }

    ep.close().await;
    Ok(())
}

async fn handle_connection(
    _conn: iroh::endpoint::Connection,
    _host: String,
    _port: u16,
) -> eyre::Result<()> {
    todo!()
}
