use http_body_util::BodyExt;

pub async fn proxy_pass<T>(
    mut req: hyper::Request<T>,
    pool: kulfi_utils::HttpConnectionPool,
    addr: &str,
    _patch: ftnet_sdk::RequestPatch,
) -> kulfi_utils::ProxyResult
where
    T: hyper::body::Body + Unpin + Send + Sync,
    T::Data: Into<hyper::body::Bytes> + Send + Sync,
    T::Error: std::error::Error + Send + Sync + 'static,
{
    use eyre::WrapErr;
    use http_body_util::BodyExt;

    let mut client = match pool.get().await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("proxy_pass: failed to get connection: {e:?}");
            return Err(eyre::anyhow!("proxy_pass: failed to get connection: {e:?}"));
        }
    };

    let path_query = req
        .uri()
        .path_and_query()
        .map_or_else(|| req.uri().path(), |v| v.as_str());

    let uri = format!("http://{addr}{path_query}");
    tracing::info!("proxying to {uri}");

    *req.uri_mut() = hyper::Uri::try_from(uri)?;

    let (parts, body) = req.into_parts();
    // let body = http_body_util::combinators::BoxBody::new(body.collect().await?);
    let body = http_body_util::Full::new(body.collect().await?.to_bytes());
    let body = http_body_util::combinators::BoxBody::new(body);

    let req = hyper::Request::from_parts(parts, body);
    // let req = req.map(|b| b.boxed());
    // let req = req.map(http_body_util::combinators::BoxBody::new);

    let resp = client
        .send_request(req)
        .await
        .wrap_err_with(|| "failed to send request")?;

    let (meta, body) = resp.into_parts();

    Ok(hyper::Response::from_parts(
        meta,
        http_body_util::combinators::BoxBody::new(body),
    ))
}

fn e(_: std::convert::Infallible) -> hyper::Error {
    panic!("e")
}
