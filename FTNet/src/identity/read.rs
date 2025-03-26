impl ftnet::Identity {
    pub async fn read(path: &std::path::Path, id: String) -> eyre::Result<Self> {
        use eyre::WrapErr;

        println!("FTNet::Identity::read: {path:?}, {id}, {}", id.len());
        Ok(Self {
            public_key: id
                .parse()
                .wrap_err_with(|| "failed to parse id to public key")?,
        })
    }
}
