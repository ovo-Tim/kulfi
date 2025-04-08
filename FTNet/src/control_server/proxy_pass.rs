pub async fn proxy_pass(
    mut req: hyper::Request<hyper::body::Incoming>,
    pool: ftnet::http::client::ConnectionPool,
    addr: &str,
    _patch: ftnet_common::RequestPatch,
) -> ftnet_utils::ProxyResult {
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

    let req = req.map(|b| b.boxed());

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
