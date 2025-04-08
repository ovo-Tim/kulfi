#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Request {
    pub uri: String,
    pub method: String,
    pub headers: Vec<(String, Vec<u8>)>,
}

impl From<hyper::http::request::Parts> for Request {
    fn from(r: hyper::http::request::Parts) -> Self {
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

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, Vec<u8>)>,
}

pub type ProxyResponse =
    hyper::Response<http_body_util::combinators::BoxBody<hyper::body::Bytes, hyper::Error>>;
pub type ProxyResult = eyre::Result<ProxyResponse>;
