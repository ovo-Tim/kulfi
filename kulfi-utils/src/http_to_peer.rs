#[tracing::instrument(skip_all)]
pub async fn http_to_peer(
    header: kulfi_utils::ProtocolHeader,
    req: hyper::Request<hyper::body::Incoming>,
    self_endpoint: iroh::Endpoint,
    remote_node_id52: &str,
    peer_connections: kulfi_utils::PeerStreamSenders,
    graceful: kulfi_utils::Graceful,
) -> kulfi_utils::http::ProxyResult<eyre::Error> {
    use http_body_util::BodyExt;

    tracing::info!("peer_proxy: {remote_node_id52}");

    let (mut send, mut recv) = kulfi_utils::get_stream(
        self_endpoint,
        header,
        remote_node_id52.to_string(),
        peer_connections.clone(),
        graceful,
    )
    .await?;

    tracing::info!("wrote protocol");

    let (head, mut body) = req.into_parts();
    send.write_all(&serde_json::to_vec(&crate::http::Request::from(head))?)
        .await?;
    send.write_all(b"\n").await?;

    tracing::info!("sent request header");

    while let Some(chunk) = body.frame().await {
        match chunk {
            Ok(v) => {
                let data = v
                    .data_ref()
                    .ok_or_else(|| eyre::anyhow!("chunk data is None"))?;
                tracing::trace!("sending chunk of size: {}", data.len());
                send.write_all(data).await?;
            }
            Err(e) => {
                tracing::error!("error reading chunk: {e:?}");
                return Err(eyre::anyhow!("read_chunk error: {e:?}"));
            }
        }
    }

    tracing::info!("sent body");

    let r: kulfi_utils::http::Response = kulfi_utils::next_json(&mut recv).await?;

    tracing::info!("got response header: {:?}", r);

    let stream = tokio_util::io::ReaderStream::new(recv);

    use futures_util::TryStreamExt;

    let stream_body = http_body_util::StreamBody::new(
        stream
            .map_ok(|b| {
                tracing::trace!("got chunk of size: {}", b.len());
                hyper::body::Frame::data(b)
            })
            .map_err(|e| {
                tracing::info!("error reading chunk: {e:?}");
                eyre::anyhow!("read_chunk error: {e:?}")
            }),
    );

    let boxed_body = http_body_util::BodyExt::boxed(stream_body);

    let mut res = hyper::Response::builder().status(hyper::http::StatusCode::from_u16(r.status)?);

    for (k, v) in r.headers {
        res = res.header(
            hyper::http::header::HeaderName::from_bytes(k.as_bytes())?,
            hyper::http::header::HeaderValue::from_bytes(&v)?,
        );
    }

    let res = res.body(boxed_body)?;

    tracing::info!("all done");
    Ok(res)
}

/// Use http_to_peer unless you have a clear reason
pub async fn http_to_peer_non_streaming(
    header: kulfi_utils::ProtocolHeader,
    req: hyper::Request<hyper::body::Bytes>,
    self_endpoint: iroh::Endpoint,
    remote_node_id52: &str,
    peer_connections: kulfi_utils::PeerStreamSenders,
    graceful: kulfi_utils::Graceful,
) -> kulfi_utils::http::ProxyResult {
    use http_body_util::BodyExt;

    tracing::info!("peer_proxy: {remote_node_id52}");

    let (mut send, mut recv) = kulfi_utils::get_stream(
        self_endpoint,
        header,
        remote_node_id52.to_string(),
        peer_connections.clone(),
        graceful,
    )
    .await?;

    tracing::info!("wrote protocol");

    let (head, body) = req.into_parts();
    send.write_all(&serde_json::to_vec(&crate::http::Request::from(head))?)
        .await?;
    send.write_all(b"\n").await?;

    tracing::info!("sent request header");

    send.write_all(&body).await?;

    tracing::info!("sent body");

    let r: kulfi_utils::http::Response = kulfi_utils::next_json(&mut recv).await?;

    tracing::info!("got response header: {r:?}");

    let mut body = Vec::with_capacity(1024 * 4);

    tracing::trace!("reading body");

    while let Some(v) = match recv.read_chunk(1024 * 64, true).await {
        Ok(v) => Ok(v),
        Err(e) => {
            tracing::error!("error reading chunk: {e:?}");
            Err(eyre::anyhow!("read_chunk error: {e:?}"))
        }
    }? {
        body.extend_from_slice(&v.bytes);
        tracing::trace!(
            "reading body, partial: {}, new body size: {} bytes",
            v.bytes.len(),
            body.len()
        );
    }

    tracing::debug!("got {} bytes of body", body.len());

    let mut res = hyper::Response::new(
        http_body_util::Full::new(body.into())
            .map_err(|e| match e {})
            .boxed(),
    );
    *res.status_mut() = hyper::http::StatusCode::from_u16(r.status)?;
    for (k, v) in r.headers {
        res.headers_mut().insert(
            hyper::http::header::HeaderName::from_bytes(k.as_bytes())?,
            hyper::http::header::HeaderValue::from_bytes(&v)?,
        );
    }

    tracing::info!("all done");
    Ok(res)
}
