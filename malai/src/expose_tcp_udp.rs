pub async fn expose_tcp_udp(
    host: String,
    port: u16,
    id52: String,
    secret_key: kulfi_id52::SecretKey,
    graceful: kulfi_utils::Graceful,
) {
    let ep = match kulfi_utils::get_endpoint(secret_key).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to bind to iroh network:");
            eprintln!("{e:?}");
            std::process::exit(1);
        }
    };

    InfoMode::Startup.print(port, &id52);

    let mut graceful_mut = graceful.clone();
    loop {
        let graceful_for_handle_connection = graceful.clone();

        tokio::select! {
            _ = graceful_mut.show_info() => {
                InfoMode::OnExit.print(port, &id52);
            }
            _ = graceful.cancelled() => {
                tracing::info!("Stopping control server.");
                break;
            }
            conn = ep.accept() => {
                let conn = match conn {
                    Some(conn) => conn,
                    None => {
                        tracing::info!("no connection");
                        break;
                    }
                };
                let host = host.clone();

                graceful.spawn(async move {
                    let start = std::time::Instant::now();
                    let conn = match conn.await {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::error!("failed to convert incoming to connection: {:?}", e);
                            return;
                        }
                    };
                    if let Err(e) = handle_connection(conn, host, port, graceful_for_handle_connection).await {
                        tracing::error!("connection error: {:?}", e);
                    }
                    tracing::info!("connection handled in {:?}", start.elapsed());
                });
            }
        }
    }

    ep.close().await;
}

async fn handle_connection(
    conn: iroh::endpoint::Connection,
    host: String,
    port: u16,
    graceful: kulfi_utils::Graceful,
) -> eyre::Result<()> {
    let remote_id52 = kulfi_utils::get_remote_id52(&conn);

    tracing::info!("new TCP+UDP client: {remote_id52}, waiting for bidirectional stream");
    let expected = [kulfi_utils::Protocol::Tcp, kulfi_utils::Protocol::Udp];
    loop {
        let (send, recv, protocol) = kulfi_utils::accept_bi_any(&conn, &expected)
            .await
            .inspect_err(|e| tracing::error!("failed to accept bidirectional stream: {e:?}"))?;
        tracing::info!("{remote_id52} protocol={protocol:?}");
        let addr = format!("{host}:{port}");
        match protocol {
            kulfi_utils::Protocol::Tcp => {
                graceful.spawn(async move {
                    if let Err(e) = kulfi_utils::peer_to_tcp(&addr, send, recv).await {
                        tracing::error!("failed to proxy tcp: {e:?}");
                    }
                    tracing::info!("closing TCP stream");
                });
            }
            kulfi_utils::Protocol::Udp => {
                graceful.spawn(async move {
                    if let Err(e) = kulfi_utils::peer_to_udp(&addr, send, recv).await {
                        tracing::error!("failed to proxy udp: {e:?}");
                    }
                    tracing::info!("closing UDP stream");
                });
            }
            _ => unreachable!(),
        }
    }
}

#[derive(PartialEq, Debug)]
enum InfoMode {
    Startup,
    OnExit,
}

impl InfoMode {
    fn print(&self, port: u16, id52: &str) {
        use colored::Colorize;

        if self == &InfoMode::OnExit {
            println!();
        }

        if self == &InfoMode::Startup {
            println!(
                "{}: Sharing TCP+UDP port {port}",
                "Malai".on_green().black()
            );
        }

        println!(
            "Run {} or {}",
            format!("malai tcp-bridge {id52} <some-port>").yellow(),
            format!("malai udp-bridge {id52} <some-port>").yellow(),
        );
        println!("to connect to it from any machine.");

        if self == &InfoMode::OnExit {
            println!("Press ctrl+c again to exit.");
        }
    }
}
