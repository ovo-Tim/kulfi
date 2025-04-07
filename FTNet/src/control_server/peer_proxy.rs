pub async fn peer_proxy(
    req: hyper::Request<hyper::body::Incoming>,
    self_id52: &str,
    remote_node_id52: &str,
    peer_connections: ftnet::identity::PeerConnections,
    client_pools: ftnet::http::client::ConnectionPools,
    _patch: ftnet_common::RequestPatch,
    fastn_port: u16,
) -> ftnet::http::Result {
    use http_body_util::BodyExt;
    use tokio_stream::StreamExt;

    tracing::info!("peer_proxy: {remote_node_id52}");

    let (mut send, recv) = get_stream(
        self_id52,
        remote_node_id52,
        peer_connections,
        client_pools,
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
    client_pools: ftnet::http::client::ConnectionPools,
    fastn_port: u16,
) -> eyre::Result<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream)> {
    let pool = {
        tracing::info!("get stream1");
        let mut peers = peer_connections.lock().await;
        tracing::info!("get stream1");

        let pool = match peers.get(remote_node_id52) {
            Some(v) => v.clone(),
            None => {
                let pool = bb8::Pool::builder()
                    .build(
                        ftnet::Identity::from_id52(self_id52, client_pools)?
                            .peer_identity(fastn_port, remote_node_id52)?,
                    )
                    .await?;

                peers.insert(remote_node_id52.to_string(), pool.clone());
                pool
            }
        };
        tracing::info!("get stream got pool");
        pool
    };

    Ok(pool
        .get()
        .await
        .inspect(|_v| tracing::info!("got connection"))
        .map_err(|e| {
            tracing::error!("failed to get connection: {e:?}");
            eyre::anyhow!("failed to get connection: {e:?}")
        })?
        .open_bi()
        .await?)
}
