pub async fn peer_proxy(
    req: hyper::Request<hyper::body::Incoming>,
    _requesting_id: &str,
    peer_id: &str,
    peer_connections: ftnet::identity::PeerConnections,
    client_pools: ftnet::http::client::ConnectionPools,
    _patch: ftnet::http::RequestPatch,
) -> ftnet::http::Result {
    use tokio_stream::StreamExt;

    let (mut send, mut recv) = get_stream(peer_id, peer_connections, client_pools).await?;

    send.write_all(&serde_json::to_vec(&ftnet::Protocol::Identity)?)
        .await?;
    send.write(b"\n").await?;

    let (head, body) = req.into_parts();
    send.write_all(&serde_json::to_vec(&Request::from(head))?)
        .await?;

    let mut body = http_body_util::BodyDataStream::new(body);

    while let Some(v) = body.next().await {
        send.write_all(&v?).await?;
    }

    // TODO: figure out how to do streaming response
    let mut buf = Vec::with_capacity(64 * 1024 * 1024);

    let _size = match recv.read(&mut buf).await? {
        Some(0) | None => {
            return Err(eyre::anyhow!("peer closed connection"));
        }
        Some(v) => v,
    };

    // let data = &buf[..size];
    //
    // let (r, rest): (Request, _) = ftnet::utils::read_newline_separated_json(data)?;
    //
    // let mut res = hyper::Response::new(
    //     http_body_util::Full::new(hyper::body::Bytes::from(rest.to_vec()))
    //         .map_err(|e| match e {})
    //         .boxed(),
    // );
    //
    // *res.status_mut() = r.method.parse::<http::StatusCode>()?;
    //
    // for (k, v) in r.headers {
    //     res.headers_mut().insert(
    //         http::header::HeaderName::from_bytes(k.as_bytes())?,
    //         http::header::HeaderValue::from_bytes(&v)?,
    //     );
    // }
    //
    // Ok(res)

    todo!()
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Request {
    pub uri: String,
    pub method: String,
    pub headers: Vec<(String, Vec<u8>)>,
}

impl From<http::request::Parts> for Request {
    fn from(r: http::request::Parts) -> Self {
        let mut headers = vec![];
        for (k, v) in r.headers {
            let k = match k {
                Some(v) => v.to_string(),
                None => continue,
            };
            headers.push((k, v.as_bytes().to_vec()));
        }

        Request {
            uri: r.uri.to_string(),
            method: r.method.to_string(),
            headers,
        }
    }
}

async fn get_stream(
    peer_id: &str,
    peer_connections: ftnet::identity::PeerConnections,
    client_pools: ftnet::http::client::ConnectionPools,
) -> eyre::Result<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream)> {
    let mut peers = peer_connections.lock().await;

    let pool = match peers.get(peer_id) {
        Some(v) => v.clone(),
        None => {
            let pool = bb8::Pool::builder()
                .build(ftnet::Identity::from_id52(peer_id, client_pools)?)
                .await?;

            peers.insert(peer_id.to_string(), pool.clone());
            pool
        }
    };

    Ok(pool
        .get()
        .await
        .map_err(|e| {
            eprintln!("failed to get connection: {e:?}");
            eyre::anyhow!("failed to get connection: {e:?}")
        })?
        .open_bi()
        .await?)
}
