pub async fn proxy_pass(
    mut req: hyper::Request<hyper::body::Incoming>,
    pool: ftnet::http::client::ConnectionPool,
    addr: &str,
    _patch: ftnet_common::RequestPatch,
) -> ftnet::http::Result {
    use eyre::WrapErr;

    let mut client = pool
        .get()
        .await
        // .wrap_err_with(|| "cant create connection")
        .unwrap();

    let path_query = req
        .uri()
        .path_and_query()
        .map_or_else(|| req.uri().path(), |v| v.as_str());

    let uri = format!("http://{addr}{path_query}");
    tracing::info!("proxying to {uri}");

    *req.uri_mut() = hyper::Uri::try_from(uri)?;

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
