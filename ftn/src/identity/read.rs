use eyre::WrapErr;

impl ftn::Identity {
    pub async fn read(path: &std::path::Path, id: String) -> eyre::Result<Self> {
        println!("ftn::Identity::read: {path:?}, {id}");

        // ensure private key exists in the keyring
        if let Err(e) = ftn::utils::get_secret(id.as_str()) {
            return Err(e)
                .wrap_err_with(|| format!("failed to get secret key from keychain for {id}"));
        }

        // TODO: start the http proxy service for this identity
        // TODO: pick an available port and start fastn service

        Ok(Self {
            id,
            fastn_port: 8000,
        })
    }
}
