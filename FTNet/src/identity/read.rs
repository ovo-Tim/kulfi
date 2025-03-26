impl ftnet::Identity {
    pub async fn read(_path: &std::path::Path, id: String) -> eyre::Result<Self> {
        use eyre::WrapErr;

        Ok(Self {
            public_key: id
                .parse()
                .wrap_err_with(|| "failed to parse id to public key")?,
        })
    }
}
