/// start ftn service
///
/// on startup, we first check if another instance is running if so we exit.
///
/// the main job of this function is to run an instance of `fastn` for every identity in the
/// identities folder, and set-up http device driver for each of them.
///
/// it also has to start the device "drivers" for every device in the <identities>/devices folder.
pub async fn start(_fg: bool, dir: Option<String>) -> eyre::Result<()> {
    use eyre::WrapErr;

    let config = ftnet::Config::read(dir)
        .await
        .wrap_err_with(|| "failed to run config")?;

    let _lock = config
        .lock()
        .await
        .wrap_err_with(|| "looks like there is another instance of ftn running")?;

    println!("ftn service started: {config:?}");
    let identities = config.identities().await?;

    let (graceful_shutdown_tx, graceful_shutdown_rx) = tokio::sync::watch::channel(false);

    for identity in identities {
        let graceful_shutdown_rx = graceful_shutdown_rx.clone();
        tokio::spawn(async move {
            let public_key = identity.public_key;
            println!("running: {public_key}");
            if let Err(e) = identity.run(graceful_shutdown_rx).await {
                eprintln!("failed to run identity: {public_key}: {e}");
            }
        });
    }

    tokio::signal::ctrl_c()
        .await
        .wrap_err_with(|| "failed to get ctrl-c signal handler")?;
    graceful_shutdown_tx
        .send(true)
        .wrap_err_with(|| "failed to send graceful shutdown signal")?;

    let mut count = 0;

    loop {
        count += 1;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let v = ftnet::OPEN_CONNECTION_COUNT.get();
        if v == 0 {
            println!("No inflight requests open.");
            break;
        }

        // every second print status
        if count % 10 == 0 {
            println!("Waiting for {v} inflight requests to finish.");
        }

        // give up in 1 min
        if count > 60 {
            println!("Giving up.");
            break;
        }
    }

    println!("Shutting down.");

    Ok(())
}
