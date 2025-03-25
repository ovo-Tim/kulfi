impl ftnet::Identity {
    pub async fn read(path: &std::path::Path, id: String) -> eyre::Result<Self> {
        println!("ftnet::Identity::run: {path:?}, {id}");
        let bytes: [u8; 32] = id.as_bytes().try_into()?; // unwrap ok as already asserted

        Ok(Self {
            public_key: iroh::PublicKey::from_bytes(&bytes)?,
        })
    }
}
