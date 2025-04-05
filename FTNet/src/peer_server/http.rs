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

    let mut body = recv.read_buffer().to_vec();
    let mut recv = recv.into_inner();

    let mut buf = Vec::with_capacity(1024 * 64);

    tracing::info!("reading body");
    while let Some(v) = recv.read(&mut buf).await? {
        if v == 0 {
            tracing::info!("finished reading");
            break;
        }
        tracing::info!("reading body, partial: {v}");
        body.extend_from_slice(&buf);
        buf.truncate(0);
    }
    tracing::info!("finished reading body");

    let mut r = hyper::Request::builder()
        .method(req.method.as_str())
        .uri(req.uri);
    for (name, value) in req.headers {
        r = r.header(name, value);
    }

    tracing::info!("request: {r:?}");
    todo!()
}
