use eyre::WrapErr;

impl ftnet::Identity {
    pub async fn run(
        self,
        graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
        id_map: ftnet::identity::IDMap,
        peer_connections: ftnet::identity::PeerConnections,
    ) -> eyre::Result<()> {
        let port = start_fastn(std::sync::Arc::clone(&id_map), graceful_shutdown_rx.clone())
            .await
            .wrap_err_with(|| "failed to start fastn")?;

        {
            id_map.lock().await.push((self.id52.to_string(), port));
        }

        let ep = ftnet::identity::get_endpoint(self.public_key.to_string().as_str())
            .await
            .wrap_err_with(|| "failed to bind to iroh network")?;

        ftnet::peer_server::run(
            ep,
            port,
            self.client_pools.clone(),
            peer_connections,
            graceful_shutdown_rx,
        )
        .await
    }
}

/// launch fastn from the package directory and return the port
async fn start_fastn(
    _id_map: ftnet::identity::IDMap,
    _graceful_shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> eyre::Result<u16> {
    // TODO: make [path] from data-dir/identities/<self.id52>/package/
    let path = std::path::Path::new("");
    let output = ftnet::utils::run_fastn(path, &["serve"])?;
    let port = parse_port_from_fastn_output(output);
    Ok(port)
}

#[tracing::instrument(skip_all)]
fn parse_port_from_fastn_output(output: String) -> u16 {
    // The following is a typical output of running `fastn serve`:
    //
    // ```
    // Checking dependencies for ftnet-template.fifthtry.site.
    // Checking ftnet.fifthtry.site: checked in 0.231s
    // All the 1 packages are up to date.
    // Applying Migration for fastn: initial
    // ### Server Started ###
    // Go to: http://127.0.0.1:8001
    // ```
    let prefix = "Go to: http://127.0.0.1:";

    output
        .lines()
        .filter(|l| l.starts_with(prefix))
        .next()
        .map(|l| l.trim())
        .and_then(|l| l.strip_prefix(prefix))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8000)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_port_from_fastn_output() {
        use super::parse_port_from_fastn_output;

        let output = r#"Checking dependencies for ftnet-template.fifthtry.site.
Checking ftnet.fifthtry.site: checked in 0.231s
All the 1 packages are up to date.
Applying Migration for fastn: initial
### Server Started ###
Go to: http://127.0.0.1:8001"#
            .to_string();

        assert_port(output, 8001);

        let output = r#"Checking dependencies for ftnet-template.fifthtry.site.
Checking ftnet.fifthtry.site: checked in 0.231s
All the 1 packages are up to date.
Applying Migration for fastn: initial
### Server Started ###
Go to: http://127.0.0.1:9800

Some garbage
"#
        .to_string();

        assert_port(output, 9800);

        fn assert_port(o: String, port: u16) {
            let p = parse_port_from_fastn_output(o);
            assert_eq!(p, port);
        }
    }
}
