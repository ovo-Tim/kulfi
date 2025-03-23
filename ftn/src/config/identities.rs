impl ftn::Config {
    pub async fn identities(&self) -> eyre::Result<Vec<ftn::Identity>> {
        use eyre::WrapErr;

        let mut identities = Vec::new();
        let dir = self.dir.join("identities");
        for entry in std::fs::read_dir(dir.join("identities"))
            .wrap_err_with(|| format!("failed to read identities folder: {dir:?}"))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let identity = ftn::Identity::read(&path).await?;
                identities.push(identity);
            }
        }
        Ok(identities)
    }
}
