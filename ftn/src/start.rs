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
