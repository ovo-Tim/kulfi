// #![deny(unused_extern_crates)]
// #![deny(unused_crate_dependencies)]
#![deny(unsafe_code)]

extern crate self as malai;

mod browse;
mod expose_http;
mod expose_tcp;
mod folder;
mod http_bridge;
mod tcp_bridge;

pub use browse::browse;
pub use expose_http::expose_http;
pub use expose_tcp::expose_tcp;
pub use folder::folder;
pub use http_bridge::http_bridge;
pub use tcp_bridge::tcp_bridge;

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

#[cfg(feature = "ui")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn ui() -> eyre::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
