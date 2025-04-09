pub async fn get_endpoint(secret_key: iroh::SecretKey) -> eyre::Result<iroh::Endpoint> {
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
