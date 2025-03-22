/// start ftn service
///
/// on startup, we first check if another instance is running if so we exit.
///
pub async fn start(fg: bool, dir: Option<String>) {
    match start_(fg, dir).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("start error: {e}");
            std::process::exit(1);
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum StartError {
    #[error("config read error: {0}")]
    ConfigReadError(#[from] ftn::config::ReadError),
}

async fn start_(_fg: bool, dir: Option<String>) -> Result<(), StartError> {
    let config = ftn::Config::read(dir).await?;
    let _lock = config.lock().await?;
    println!("ftn service started: {config:?}");
    tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;

    Ok(())
}
