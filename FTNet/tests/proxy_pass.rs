use tracing::Instrument;

#[tokio::test]
async fn proxy_pass() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    tokio::spawn({
        let foreground = false;

        let data_dir = std::env::temp_dir().join(format!(
            "ftnet-test-data-dir-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));

        async move { ftnet::start(foreground, data_dir, 9090).await }
            .instrument(tracing::info_span!("ftnet_server"))
    });

    // - Send http request to 127.0.0.1:9090 (with host <id>.smth.else) and expect a ftnet dashboard response

    reqwest::Client::builder().build()?.get("http://127.0.0.1:9090")
        .header("Host", "id52.smth.else")
        .send().await;

    Ok(())
}
