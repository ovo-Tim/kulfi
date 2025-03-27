impl ftnet::Identity {
    pub async fn read(_path: &std::path::Path, id: String) -> eyre::Result<Self> {
        use eyre::WrapErr;

        let public_key: iroh::PublicKey = id
            .parse()
            .wrap_err_with(|| "failed to parse id to public key")?;

        Ok(Self {
            id: public_key.fmt_short(),
            public_key,
        })
    }
}
