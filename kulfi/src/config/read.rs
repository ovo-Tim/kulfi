use eyre::WrapErr;

impl kulfi::Config {
    pub async fn lock(&self) -> eyre::Result<file_guard::FileGuard<&std::fs::File>> {
        kulfi_utils::dot_kulfi::exclusive(&self.lock_file)
            .await
            .wrap_err_with(|| "Config::lock(): failed to take exclusive lock")
    }

    pub async fn read(
        dir: &std::path::Path,
        _client_pools: kulfi_utils::HttpConnectionPools,
    ) -> eyre::Result<Self> {
        let dir = kulfi_utils::dot_kulfi::init_if_required(dir)
            .await
            .wrap_err_with(|| "Config: failed to get init directory")?;
        let lock_file = kulfi_utils::dot_kulfi::kulfi_lock_file(&dir)
            .wrap_err_with(|| "failed to create lock file")?;
        Ok(Self { dir, lock_file })
    }
}
