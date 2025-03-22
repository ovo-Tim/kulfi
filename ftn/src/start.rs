/// start ftn service
///
/// on startup, we first check if another instance is running if so we exit.
///
pub async fn start(_fg: bool, dir: Option<String>) {
    let dir = match ftn::dotftn::init_if_required(dir).await {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };

    let lock_file = match ftn::dotftn::lock_file(&dir) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };

    let _lock = match ftn::dotftn::exclusive(&lock_file).await {
        Ok(lock) => lock,
        Err(ftn::dotftn::LockError::AlreadyLocked) => {
            eprintln!("ftn is already running.");
            // exit code?
            return;
        }
        Err(e) => {
            eprintln!("failed to acquire lock: {e}");
            return;
        }
    };

    println!("ftn service started");
    tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
}
