impl ftnet::Identity {
    pub async fn read(_path: &std::path::Path, id: String) -> eyre::Result<Self> {
        use eyre::WrapErr;

        let bytes = data_encoding::BASE32_DNSSEC.decode(id.as_bytes())?;
        if bytes.len() != 32 {
            return Err(eyre::anyhow!(
                "read: id has invalid length: {}",
                bytes.len()
            ));
        }

        let bytes: [u8; 32] = bytes.try_into().unwrap(); // unwrap ok as already asserted

        let public_key: iroh::PublicKey = iroh::PublicKey::from_bytes(&bytes)
            .wrap_err_with(|| "failed to parse id to public key")?;

        Ok(Self {
            id: data_encoding::BASE32_DNSSEC.encode(public_key.as_bytes()),
            public_key,
        })
    }
}
