use ftnet_utils::http::ProxyResponse;

pub mod client;

pub fn json<T: serde::Serialize>(o: T) -> ProxyResponse {
    let bytes = match serde_json::to_vec(&o) {
        Ok(v) => v,
        Err(e) => return server_error_(format!("failed to serialize json: {e:?}")),
    };
    bytes_to_resp(bytes, hyper::StatusCode::OK)
}

pub fn server_error_(s: String) -> ProxyResponse {
    bytes_to_resp(s.into_bytes(), hyper::StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn not_found_(m: String) -> ProxyResponse {
    bytes_to_resp(m.into_bytes(), hyper::StatusCode::NOT_FOUND)
}

pub fn bad_request_(m: String) -> ProxyResponse {
    bytes_to_resp(m.into_bytes(), hyper::StatusCode::BAD_REQUEST)
}

#[macro_export]
macro_rules! server_error {
    ($($t:tt)*) => {{
        ftnet::http::server_error_(format!($($t)*))
    }};
}

#[macro_export]
macro_rules! not_found {
    ($($t:tt)*) => {{
        ftnet::http::not_found_(format!($($t)*))
    }};
}

#[macro_export]
macro_rules! bad_request {
    ($($t:tt)*) => {{
        ftnet::http::bad_request_(format!($($t)*))
    }};
}

pub fn redirect<S: AsRef<str>>(url: S) -> ProxyResponse {
    let mut r = bytes_to_resp(vec![], hyper::StatusCode::PERMANENT_REDIRECT);
    *r.headers_mut().get_mut(hyper::header::LOCATION).unwrap() = url.as_ref().parse().unwrap();
    r
}

pub fn bytes_to_resp(bytes: Vec<u8>, status: hyper::StatusCode) -> ProxyResponse {
    use http_body_util::BodyExt;

    let mut r = hyper::Response::new(
        http_body_util::Full::new(hyper::body::Bytes::from(bytes))
            .map_err(|e| match e {})
            .boxed(),
    );
    *r.status_mut() = status;
    r
}
