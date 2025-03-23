use eyre::WrapErr;

pub fn lock_file(dir: &std::path::Path) -> eyre::Result<std::fs::File> {
    let path = dir.join(super::LOCK_FILE);
    let file = std::fs::File::create(&path)
        .wrap_err_with(|| format!("failed to create lock file: {path:?}"))?;
    Ok(file)
}

pub async fn exclusive(
    lock_file: &std::fs::File,
) -> eyre::Result<file_guard::FileGuard<&std::fs::File>> {
    lock(lock_file, file_guard::Lock::Exclusive)
        .await
        .wrap_err("failed to take exclusive lock")
}

/// `lock()` is used to create lock on the `ftn` directory.
/// we do this by creating a `ftn.lock` file, and acquiring a lock on it.
pub async fn lock(
    lock_file: &std::fs::File,
    lock: file_guard::Lock,
) -> eyre::Result<file_guard::FileGuard<&std::fs::File>> {
    // check if file exists, if not create it
    file_guard::try_lock(lock_file, lock, 0, 10)
        .wrap_err_with(|| format!("file guard try_lock failed: {lock_file:?}, {lock:?}"))
}
