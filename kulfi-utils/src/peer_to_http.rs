pub async fn peer_to_http(
    addr: &str,
    client_pools: kulfi_utils::HttpConnectionPools,
    send: &mut iroh::endpoint::SendStream,
    mut recv: kulfi_utils::FrameReader,
) -> eyre::Result<()> {
    use eyre::WrapErr;
    use http_body_util::BodyExt;
    use tokio::io::AsyncWriteExt;
    use tokio_stream::StreamExt;

    tracing::info!("http request with {addr}");
    let start = std::time::Instant::now();

    let req: kulfi_utils::http::Request = match recv.next().await {
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

    let mut r = hyper::Request::builder()
        .method(req.method.as_str())
        .uri(&req.uri);
    for (name, value) in req.headers {
        r = r.header(name, value);
    }

    tracing::debug!("request: {r:?}");

    let pool = get_pool(addr, client_pools).await?;
    tracing::trace!("got pool");
    let mut client = match pool.get().await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("failed to get connection: {e:?}");
            return Err(eyre::anyhow!("failed to get connection: {e:?}"));
        }
    };
    // tracing::info!("got client");

    use futures_util::TryStreamExt;
    let stream_body = http_body_util::StreamBody::new(
        recv.map_ok(|line| hyper::body::Frame::data(bytes::Bytes::from(line)))
            .map_err(|e| {
                tracing::error!("error reading chunk: {e:?}");
                eyre::anyhow!("read_chunk error: {e:?}")
            }),
    );

    let boxed_body = http_body_util::BodyExt::boxed(stream_body);

    let (resp, mut body) = client
        .send_request(r.body(boxed_body)?)
        .await
        .wrap_err_with(|| "failed to send request")?
        .into_parts();

    let r = kulfi_utils::http::Response {
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

    send.flush().await?;

    tracing::debug!(
        "got response body of size: {:?} bytes",
        hyper::body::Body::size_hint(&body)
    );

    while let Some(chunk) = body.frame().await {
        match chunk {
            Ok(v) => {
                let data = v
                    .data_ref()
                    .ok_or_else(|| eyre::anyhow!("chunk data is None"))?;
                tracing::info!("sending chunk of size: {}", data.len());
                send.write_all(data).await?;
            }
            Err(e) => {
                tracing::error!("error reading chunk: {e:?}");
                return Err(eyre::anyhow!("read_chunk error: {e:?}"));
            }
        }
    }

    send.flush().await?;

    tracing::info!("handled http request in {:?}", start.elapsed());

    {
        use colored::Colorize;
        println!(
            "{} {} {} in {}",
            req.method.to_uppercase().green(),
            req.uri,
            resp.status.as_str().on_blue().black(),
            format!("{}ms", start.elapsed().as_millis()).yellow()
        );
    }

    Ok(())
}

async fn get_pool(
    addr: &str,
    client_pools: kulfi_utils::HttpConnectionPools,
) -> eyre::Result<bb8::Pool<kulfi_utils::HttpConnectionManager>> {
    tracing::trace!("get pool called");
    let mut pools = client_pools.lock().await;

    Ok(match pools.get(addr) {
        Some(v) => {
            tracing::debug!("found existing pool for {addr}");
            v.clone()
        }
        None => {
            tracing::debug!("creating new pool for {addr}");

            let pool = bb8::Pool::builder()
                .build(kulfi_utils::HttpConnectionManager::new(addr.to_string()))
                .await?;

            pools.insert(addr.to_string(), pool.clone());
            pool
        }
    })
}
