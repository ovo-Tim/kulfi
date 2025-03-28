pub type HttpResponse =
    hyper::Response<http_body_util::combinators::BoxBody<hyper::body::Bytes, std::io::Error>>;
pub type HttpResult<E = std::io::Error> = Result<HttpResponse, E>;

pub async fn handle_connection(
    stream: tokio::net::TcpStream,
    mut graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    ftnet::OPEN_CONNECTION_COUNT.incr();
    ftnet::CONNECTION_COUNT.incr();

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
                hyper::service::service_fn(handle_request),
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

    ftnet::OPEN_CONNECTION_COUNT.decr();
}

async fn handle_request(r: hyper::Request<hyper::body::Incoming>) -> HttpResult {
    ftnet::REQUEST_COUNT.incr();
    ftnet::IN_FLIGHT_REQUESTS.incr();
    let r = handle_request_(r).await;
    ftnet::IN_FLIGHT_REQUESTS.decr();
    r
}

async fn handle_request_(_r: hyper::Request<hyper::body::Incoming>) -> HttpResult {
    todo!()
}
