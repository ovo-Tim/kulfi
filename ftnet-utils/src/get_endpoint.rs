pub enum Key {
    ID(String),
    ID52(String),
    SecretKey(iroh::SecretKey),
}

pub async fn get_endpoint(key: Key) -> eyre::Result<iroh::Endpoint> {
    use eyre::WrapErr;

    let secret_key = match key {
        Key::ID(id) => ftnet_utils::get_secret(id.as_str())
            .wrap_err_with(|| format!("failed to get secret key from keychain for {id}"))?,
        Key::SecretKey(key) => key,
        Key::ID52(v) => {
            let public_key = ftnet_utils::utils::id52_to_public_key(&v)?;
            ftnet_utils::get_secret(public_key.to_string().as_str())?
        }
    };

    match iroh::Endpoint::builder()
        .discovery_n0()
        .discovery_local_network()
        .alpns(vec![ftnet_utils::APNS_IDENTITY.into()])
        .secret_key(secret_key)
        .bind()
        .await
    {
        Ok(ep) => Ok(ep),
        Err(e) => {
            // https://github.com/n0-computer/iroh/issues/2741
            // this is why you MUST NOT use anyhow::Error etc. in library code.
            Err(eyre::anyhow!("failed to bind to iroh network2: {e:?}"))
        }
    }
}
