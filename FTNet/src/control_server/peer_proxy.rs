#[expect(clippy::too_many_arguments)]
pub async fn peer_proxy(
    req: hyper::Request<hyper::body::Incoming>,
    self_id52: &str,
    remote_node_id52: &str,
    peer_connections: ftnet::identity::PeerConnections,
    client_pools: ftnet::http::client::ConnectionPools,
    _patch: ftnet_common::RequestPatch,
    fastn_port: u16,
    id_map: ftnet::identity::IDMap,
) -> ftnet::http::Result {
    use http_body_util::BodyExt;
    use tokio_stream::StreamExt;

    tracing::info!("peer_proxy: {remote_node_id52}");

    let (mut send, recv) = get_stream(
        self_id52,
        remote_node_id52,
        peer_connections,
        client_pools,
        id_map,
        fastn_port,
    )
        .await?;

    tracing::info!("got stream");
    send.write_all(&serde_json::to_vec(&ftnet::Protocol::Identity)?)
        .await?;
    send.write(b"\n").await?;

    tracing::info!("wrote protocol");

    let (head, body) = req.into_parts();
    send.write_all(&serde_json::to_vec(&Request::from(head))?)
        .await?;
    send.write_all("\n".as_bytes()).await?;
    tracing::info!("sent request header");

    let mut body = http_body_util::BodyDataStream::new(body);
    while let Some(v) = body.next().await {
        send.write_all(&v?).await?;
    }

    tracing::info!("sent body");

    let mut recv = ftnet::utils::frame_reader(recv);
    let r: ftnet::peer_server::http::Response = match recv.next().await {
        Some(v) => serde_json::from_str(&v?)?,
        None => {
            tracing::error!("failed to read from incoming connection");
            return Err(eyre::anyhow!("failed to read from incoming connection"));
        }
    };

    tracing::info!("got response header: {r:?}");

    let mut body = recv.read_buffer().to_vec();
    let mut recv = recv.into_inner();

    let mut buf = Vec::with_capacity(1024 * 64);

    while let Some(v) = recv.read(&mut buf).await? {
        if v == 0 {
            tracing::info!("finished reading body");
            break;
        }
        body.extend_from_slice(&buf);
        buf.truncate(0);
    }

    tracing::info!("got body");

    let mut res = hyper::Response::new(
        http_body_util::Full::new(hyper::body::Bytes::from(body))
            .map_err(|e| match e {})
            .boxed(),
    );
    *res.status_mut() = http::StatusCode::from_u16(r.status)?;
    for (k, v) in r.headers {
        res.headers_mut().insert(
            http::header::HeaderName::from_bytes(k.as_bytes())?,
            http::header::HeaderValue::from_bytes(&v)?,
        );
    }

    tracing::info!("all done");
    Ok(res)
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
    self_id52: &str,
    remote_node_id52: &str,
    peer_connections: ftnet::identity::PeerConnections,
    _client_pools: ftnet::http::client::ConnectionPools,
    id_map: ftnet::identity::IDMap,
    _fastn_port: u16,
) -> eyre::Result<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream)> {
    let connections = peer_connections.lock().await;
    let connection = connections.get(remote_node_id52).map(ToOwned::to_owned);

    // we drop the connections mutex guard so that we do not hold lock across await point.
    drop(connections);

    if let Some(conn) = connection {
        return Ok(conn.open_bi().await?);
    }

    let ep = get_endpoint(self_id52, id_map).await?;
    let conn = match ep.connect(ftnet::utils::id52_to_public_key(remote_node_id52)?, ftnet::APNS_IDENTITY).await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("failed to create connection: {e:?}");
            return Err(eyre::anyhow!("failed to create connection: {e:?}"));
        }
    };

    {
        let mut connections = peer_connections.lock().await;
        connections.insert(remote_node_id52.to_string(), conn.clone());
    }

    Ok(conn.open_bi().await?)
}

async fn get_endpoint(
    self_id52: &str,
    id_map: ftnet::identity::IDMap,
) -> eyre::Result<iroh::endpoint::Endpoint> {
    let map = id_map.lock().await;

    for (id, (_port, ep)) in map.iter() {
        if id == self_id52 {
            return Ok(ep.clone());
        }
    }

    tracing::error!("no entry for {self_id52} in the id_map: {id_map:?}");
    Err(eyre::anyhow!(
        "no entry for {self_id52} in the id_map: {id_map:?}"
    ))
}
