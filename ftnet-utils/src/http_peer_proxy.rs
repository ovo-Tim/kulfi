pub async fn http(
    addr: &str,
    client_pools: ftnet_utils::ConnectionPools,
    send: &mut iroh::endpoint::SendStream,
    mut recv: ftnet_utils::utils::FrameReader,
) -> eyre::Result<()> {
    use eyre::WrapErr;
    use http_body_util::BodyExt;
    use tokio_stream::StreamExt;

    tracing::info!("http called with {addr}");
    let req: ftnet_utils::http::Request = match recv.next().await {
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

    let pool = get_pool(addr, client_pools).await?;
    // tracing::info!("got pool");
    let mut client = match pool.get().await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("failed to get connection: {e:?}");
            return Err(eyre::anyhow!("failed to get connection: {e:?}"));
        }
    };
    // tracing::info!("got client");

    let (resp, body) = client
        .send_request(
            r.body(
                http_body_util::Full::new(hyper::body::Bytes::from(body))
                    .map_err(|e| match e {})
                    .boxed(),
            )?,
        )
        .await
        .wrap_err_with(|| "failed to send request")?
        .into_parts();

    let r = ftnet_utils::http::Response {
        status: resp.status.as_u16(),
        headers: resp
            .headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.as_bytes().to_vec()))
            .collect(),
    };

    send.write_all(
        serde_json::to_string(&r)
            .wrap_err_with(|| "failed to serialize json while writing http response")?
            .as_bytes(),
    )
    .await?;
    send.write_all(b"\n").await?;
    send.write_all(&(body.collect().await?.to_bytes())).await?;

    Ok(())
}

async fn get_pool(
    addr: &str,
    client_pools: ftnet_utils::ConnectionPools,
) -> eyre::Result<bb8::Pool<ftnet_utils::ConnectionManager>> {
    tracing::info!("get client");
    let mut pools = client_pools.lock().await;
    tracing::info!("get client1");

    Ok(match pools.get(addr) {
        Some(v) => v.clone(),
        None => {
            let pool = bb8::Pool::builder()
                .build(ftnet_utils::ConnectionManager::new(addr.to_string()))
                .await?;

            pools.insert(addr.to_string(), pool.clone());
            pool
        }
    })
}
