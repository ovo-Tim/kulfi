/// start ftn service
///
/// on startup, we first check if another instance is running if so we exit.
pub async fn start(fg: bool, dir: Option<String>) {
    match start_(fg, dir).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("start error: {e:?}");
            std::process::exit(1);
        }
    }
}

async fn start_(_fg: bool, dir: Option<String>) -> eyre::Result<()> {
    use eyre::WrapErr;

    let config = ftn::Config::read(dir)
        .await
        .wrap_err("failed to read config")?;
    let _lock = config
        .lock()
        .await
        .wrap_err("looks like there is another instance of ftn running")?;
    println!("ftn service started: {config:?}");
    tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;

    Ok(())
}
