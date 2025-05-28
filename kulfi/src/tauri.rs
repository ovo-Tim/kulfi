#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn ui() -> eyre::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .register_asynchronous_uri_scheme_protocol("kulfi", |_ctx, request, responder| {
            tauri::async_runtime::spawn(async move {
                let mut request = kulfi_utils::http::vec_u8_to_bytes(request);

                let (new_uri, id52) = kulfi_uri_to_path_and_id52(&request.uri());

                *request.uri_mut() = new_uri.parse().expect("failed to parse new URI");

                // will get 400 Bad Request if the host is not set
                request.headers_mut().insert(
                    "HOST",
                    format!("kulfi://{id52}")
                        .parse()
                        .expect("failed to parse header value"),
                );

                let request = request; // remove mut bind

                tracing::info!(?request, "Sending Request");

                let graceful = kulfi_utils::Graceful::default();
                let peer_connections = kulfi_utils::PeerStreamSenders::default();
                let response = kulfi_utils::http_to_peer(
                    kulfi_utils::Protocol::Http.into(),
                    request,
                    global_iroh_endpoint().await,
                    &id52,
                    peer_connections,
                    Default::default(), /* RequestPatch */
                    graceful,
                )
                .await;

                responder.respond(
                    kulfi_utils::http::response_to_static(response)
                        .await
                        .expect("failed to convert response to static"),
                );
            });
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

pub async fn global_iroh_endpoint() -> iroh::Endpoint {
    async fn new_iroh_endpoint() -> iroh::Endpoint {
        // TODO: read secret key from ENV VAR
        iroh::Endpoint::builder()
            .discovery_n0()
            .discovery_local_network()
            .alpns(vec![kulfi_utils::APNS_IDENTITY.into()])
            .bind()
            .await
            .expect("failed to create iroh Endpoint")
    }

    static IROH_ENDPOINT: tokio::sync::OnceCell<iroh::Endpoint> =
        tokio::sync::OnceCell::const_new();
    IROH_ENDPOINT.get_or_init(new_iroh_endpoint).await.clone()
}

fn kulfi_uri_to_path_and_id52(uri: &hyper::Uri) -> (String, String) {
    // TODO: handle the following assert as error
    let uri_str = uri.to_string();
    assert!(uri_str.starts_with("kulfi://"));

    let id52 = uri.host().expect("URI must have a host");

    assert!(
        id52.len() == 52,
        "ID must be 52 characters long, got: {id52}"
    );

    // TODO: id52 must be alphanumeric only. should not have a dot (.)

    let new_uri = uri_str
        .strip_prefix(format!("kulfi://{id52}").as_str())
        .expect("already assert for kulfi://");

    let new_uri = if new_uri.is_empty() {
        "/".to_string()
    } else if !new_uri.starts_with('/') {
        format!("/{}", new_uri)
    } else {
        new_uri.to_string()
    };

    (new_uri, id52.to_string())
}
