pub async fn http(
    addr: &str,
    _client_pools: ftnet::http::client::ConnectionPools,
    _send: &mut iroh::endpoint::SendStream,
    mut recv: ftnet::utils::FrameReader,
) -> eyre::Result<()> {
    use eyre::WrapErr;
    use tokio_stream::StreamExt;

    tracing::info!("http called with {addr}");
    let req: ftnet::control_server::Request = match recv.next().await {
        Some(Ok(v)) => serde_json::from_str(&v)
            .wrap_err_with(|| "failed to serialize json while reading http request")?,
        Some(Err(e)) => {
            tracing::error!("failed to read request: {e}");
            return Err(eyre::anyhow!("failed to read request: {e}"));
        }
        None => {
            tracing::error!("no request");
            return Err(eyre::anyhow!("no request"));
        }
    };

    tracing::info!("got request: {req:?}");

    todo!()
}
