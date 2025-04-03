pub async fn handle_connection(
    stream: tokio::net::TcpStream,
    mut graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
    id_map: ftnet::identity::IDMap,
    client_pools: ftnet::http::client::ConnectionPools,
    peer_connections: ftnet::identity::PeerConnections,
) {
    ftnet::OPEN_CONTROL_CONNECTION_COUNT.incr();
    ftnet::CONTROL_CONNECTION_COUNT.incr();

    let io = hyper_util::rt::TokioIo::new(stream);

    let builder =
        hyper_util::server::conn::auto::Builder::new(hyper_util::rt::tokio::TokioExecutor::new());
    // the following builder runs only http2 service, whereas the hyper_util auto Builder runs an
    // http1.1 server that upgrades to http2 if the client requests.
    // let builder = hyper::server::conn::http2::Builder::new(hyper_util::rt::tokio::TokioExecutor::new());
    tokio::pin! {
        let conn = builder
            .serve_connection(
                io,
                // http/1.1 allows https://en.wikipedia.org/wiki/HTTP_pipelining
                // but hyper does not, https://github.com/hyperium/hyper/discussions/2747:
                //
                // > hyper does not support HTTP/1.1 pipelining, since it's a deprecated HTTP
                // > feature. it's better to use HTTP/2.
                //
                // so we will never have IN_FLIGHT_REQUESTS > OPEN_CONNECTION_COUNT.
                //
                // for hostn-edge contacting hostn-document / hostn-wasm, it may have been useful to
                // send multiple requests on the same connection as they are independent of each
                // other. without pipelining, we will end up having effectively more open
                // connections between edge and js/wasm.
                hyper::service::service_fn(|r| handle_request(r, id_map.clone(), client_pools.clone(), peer_connections.clone())),
            );
    }

    if let Err(e) = tokio::select! {
        _ = graceful_shutdown_rx.changed() => {
            conn.as_mut().graceful_shutdown();
            conn.await
        }
        r = &mut conn => r,
    } {
        eprintln!("connection error: {e:?}");
    }

    ftnet::OPEN_CONTROL_CONNECTION_COUNT.decr();
}

async fn handle_request(
    r: hyper::Request<hyper::body::Incoming>,
    id_map: ftnet::identity::IDMap,
    client_pools: ftnet::http::client::ConnectionPools,
    peer_connections: ftnet::identity::PeerConnections,
) -> ftnet::http::Result {
    ftnet::CONTROL_REQUEST_COUNT.incr();
    ftnet::IN_FLIGHT_REQUESTS.incr();
    let r = handle_request_(r, id_map, client_pools, peer_connections).await;
    ftnet::IN_FLIGHT_REQUESTS.decr();
    r
}

async fn handle_request_(
    r: hyper::Request<hyper::body::Incoming>,
    id_map: ftnet::identity::IDMap,
    client_pools: ftnet::http::client::ConnectionPools,
    peer_connections: ftnet::identity::PeerConnections,
) -> ftnet::http::Result {
    let id = match r
        .headers()
        .get("Host")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.split_once('.'))
    {
        Some((first, _)) => first,
        None => {
            eprintln!("got http request without Host header");
            return Ok(ftnet::bad_request!("got http request without Host header"));
        }
    };

    println!("got request for {id}");

    // if this is an identity, if so forward the request to fastn corresponding to that identity
    if let Some(fastn_port) = find_identity(id, id_map.clone()).await? {
        let addr = format!("127.0.0.1:{fastn_port}");
        return ftnet::control_server::proxy_pass(
            r,
            find_pool(client_pools, &addr).await?,
            &addr,
            Default::default(),
        )
        .await;
    }

    // TODO: maybe we should try all the identities not just default
    let (default_id, default_port) = default_identity(id_map).await?;
    match what_to_do(default_port, id).await {
        // if the id belongs to a friend of an identity, send the request to the friend over iroh
        Ok(WhatToDo::ForwardToPeer { peer_id, patch }) => {
            ftnet::control_server::peer_proxy(
                r,
                default_id.as_str(),
                peer_id.as_str(),
                peer_connections,
                client_pools,
                patch,
                default_port,
            )
            .await
        }
        // if not identity, find if the id is an http device owned by identity, if so proxy-pass the
        // request to that device
        Ok(WhatToDo::ProxyPass {
            port,
            extra_headers,
        }) => {
            let addr = format!("127.0.0.1:{port}");
            ftnet::control_server::proxy_pass(
                r,
                find_pool(client_pools, &addr).await?,
                &addr,
                extra_headers,
            )
            .await
        }
        Ok(WhatToDo::UnknownPeer) => {
            eprintln!("unknown peer: {id}");
            Ok(ftnet::server_error!("unknown peer"))
        }
        Err(e) => {
            eprintln!("proxy error: {e}");
            Ok(ftnet::server_error!(
                "failed to contact default identity service"
            ))
        }
    }
}

pub async fn find_pool(
    client_pools: ftnet::http::client::ConnectionPools,
    addr: &str,
) -> eyre::Result<ftnet::http::client::ConnectionPool> {
    {
        let pools = client_pools.lock().await;
        if let Some(v) = pools.get(addr) {
            return Ok(v.to_owned());
        }
    }

    let c = ftnet::http::client::ConnectionPool::builder()
        .build(ftnet::http::client::ConnectionManager::new(
            addr.to_string(),
        ))
        .await?;

    {
        client_pools
            .lock()
            .await
            .insert(addr.to_string(), c.clone());
    }

    Ok(c)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum WhatToDo {
    ForwardToPeer {
        peer_id: String,
        patch: ftnet::http::RequestPatch,
    },
    ProxyPass {
        port: u16,
        extra_headers: ftnet::http::RequestPatch,
    },
    UnknownPeer,
}

async fn what_to_do(_port: u16, id: &str) -> eyre::Result<WhatToDo> {
    // request to fastn server at /-/ftnet/v1/control/what-to-do/<id>/
    Ok(WhatToDo::ForwardToPeer {
        peer_id: id.to_string(),
        patch: Default::default(),
    })
}

async fn find_identity(id: &str, id_map: ftnet::identity::IDMap) -> eyre::Result<Option<u16>> {
    for (i, v) in id_map.lock().await.iter() {
        // if i.starts_with(id) {
        if i == id {
            return Ok(Some(*v));
        }
    }

    Ok(None)
}

async fn default_identity(id_map: ftnet::identity::IDMap) -> eyre::Result<(String, u16)> {
    Ok(id_map
        .lock()
        .await
        .first()
        .map(ToOwned::to_owned)
        // ftnet ensures there is at least one identity at the start
        .unwrap())
}
