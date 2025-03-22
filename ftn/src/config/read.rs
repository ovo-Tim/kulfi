#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("dotftn init error: {0}")]
    DotFtnInitError(ftn::config::dotftn::InitError),
    #[error("dotftn lock error: {0}")]
    DotFtnLockError(ftn::config::dotftn::LockError),
    #[error("dotftn lock file error: {0}")]
    DotFtnLockFileError(ftn::config::dotftn::LockFileError),
    #[error("ftn is already running")]
    AlreadyRunning,
}

impl ftn::Config {
    pub async fn lock(&self) -> Result<file_guard::FileGuard<&std::fs::File>, ReadError> {
        match ftn::config::dotftn::exclusive(&self.lock_file).await {
            Ok(lock) => Ok(lock),
            Err(ftn::config::dotftn::LockError::AlreadyLocked) => Err(ReadError::AlreadyRunning),
            Err(e) => Err(ReadError::DotFtnLockError(e)),
        }
    }

    pub async fn read(dir: Option<String>) -> Result<Self, ReadError> {
        let dir = match ftn::config::dotftn::init_if_required(dir).await {
            Ok(dir) => dir,
            Err(e) => {
                return Err(ReadError::DotFtnInitError(e));
            }
        };

        let lock_file = match ftn::config::dotftn::lock_file(&dir) {
            Ok(file) => file,
            Err(e) => {
                return Err(ReadError::DotFtnLockFileError(e));
            }
        };

        Ok(Self { dir, lock_file })
    }
}
