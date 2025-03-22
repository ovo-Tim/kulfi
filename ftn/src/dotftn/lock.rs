pub const LOCK_FILE: &str = "ftn.lock";

#[derive(Debug, thiserror::Error)]
pub enum LockFileError {
    #[error("could not create lock file: {0}")]
    CreateLockFile(std::io::Error),
    #[error("could not open lock file: {0}")]
    OpenLockFile(std::io::Error),
    #[error("could not acquire lock: {0}")]
    AcquireLock(std::io::Error),
}

pub fn lock_file(dir: &std::path::Path) -> Result<std::fs::File, LockFileError> {
    let path = dir.join(LOCK_FILE);
    let file = std::fs::File::create(path).map_err(LockFileError::OpenLockFile)?;
    Ok(file)
}

pub async fn exclusive(
    lock_file: &std::fs::File,
) -> Result<file_guard::FileGuard<&std::fs::File>, LockError> {
    lock(lock_file, file_guard::Lock::Exclusive).await
}

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("could not acquire lock: {0}")]
    AcquireLock(std::io::Error),
    #[error("lock file already locked")]
    AlreadyLocked,
}

/// `lock()` is used to create lock on the `ftn` directory.
/// we do this by creating a `ftn.lock` file, and acquiring a lock on it.
pub async fn lock(
    lock_file: &std::fs::File,
    lock: file_guard::Lock,
) -> Result<file_guard::FileGuard<&std::fs::File>, LockError> {
    // check if file exists, if not create it
    match file_guard::try_lock(lock_file, lock, 0, 10) {
        Ok(lock) => Ok(lock),
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Err(LockError::AlreadyLocked),
        Err(e) => Err(LockError::AcquireLock(e)),
    }
}
