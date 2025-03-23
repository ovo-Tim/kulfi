use eyre::WrapErr;

impl ftn::Config {
    pub async fn lock(&self) -> eyre::Result<file_guard::FileGuard<&std::fs::File>> {
        ftn::config::dotftn::exclusive(&self.lock_file)
            .await
            .wrap_err_with(|| "Config::lock(): failed to take exclusive lock")
    }

    pub async fn read(dir: Option<String>) -> eyre::Result<Self> {
        let dir = ftn::config::dotftn::init_if_required(dir)
            .await
            .wrap_err_with(|| "Config: failed to get init directory")?;
        let lock_file =
            ftn::config::dotftn::lock_file(&dir).wrap_err_with(|| "failed to create lock file")?;
        Ok(Self { dir, lock_file })
    }
}
