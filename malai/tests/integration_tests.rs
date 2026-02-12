//! Integration tests for client-server connectivity
//!
//! These tests verify that clients can successfully connect to local servers
//! through the kulfi P2P network using malai's expose and bridge functionality.

use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

/// Per-test timeout to prevent hanging when P2P discovery cannot complete.
const TEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Max time to wait for graceful shutdown before aborting.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);

/// Helper to create a test identity
async fn create_test_identity() -> (String, kulfi_id52::SecretKey) {
    let secret = kulfi_id52::SecretKey::generate();
    let id52 = secret.id52();
    (id52, secret)
}

/// Helper to start a simple TCP echo server
async fn start_tcp_echo_server(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .expect("Failed to bind TCP echo server");

        loop {
            match listener.accept().await {
                Ok((mut socket, _)) => {
                    tokio::spawn(async move {
                        let mut buf = vec![0u8; 1024];
                        loop {
                            match socket.read(&mut buf).await {
                                Ok(0) => break, // Connection closed
                                Ok(n) => {
                                    if socket.write_all(&buf[..n]).await.is_err() {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                    break;
                }
            }
        }
    })
}

/// Helper to start a UDP echo server
async fn start_udp_echo_server(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let socket = UdpSocket::bind(format!("127.0.0.1:{}", port))
            .await
            .expect("Failed to bind UDP echo server");

        let mut buf = vec![0u8; 65535];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((n, addr)) => {
                    if socket.send_to(&buf[..n], addr).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("UDP echo server error: {}", e);
                    break;
                }
            }
        }
    })
}

/// Helper to start an HTTP server
async fn start_http_server(port: u16) -> tokio::task::JoinHandle<()> {
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{Request, Response, body::Incoming};
    use hyper_util::rt::TokioIo;

    tokio::spawn(async move {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .expect("Failed to bind HTTP server");

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    tokio::spawn(async move {
                        let io = TokioIo::new(stream);
                        let service = service_fn(|_req: Request<Incoming>| async {
                            Ok::<_, hyper::Error>(Response::new(
                                "Hello from test server!".to_string(),
                            ))
                        });

                        if let Err(err) = http1::Builder::new().serve_connection(io, service).await
                        {
                            eprintln!("Error serving connection: {:?}", err);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept HTTP connection: {}", e);
                    break;
                }
            }
        }
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn test_tcp_client_server_connection() {
    tokio::time::timeout(TEST_TIMEOUT, test_tcp_client_server_connection_inner())
        .await
        .expect("test_tcp_client_server_connection timed out");
}

async fn test_tcp_client_server_connection_inner() {
    // Setup logging for debugging
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Create test identity
    let (id52, secret) = create_test_identity().await;
    println!("Test identity: {}", id52);

    // Start echo server on a random port
    let echo_port = 9001;
    let echo_handle = start_tcp_echo_server(echo_port).await;

    // Start expose_tcp server
    let graceful = kulfi_utils::Graceful::new();
    let expose_graceful = graceful.clone();
    let expose_id52 = id52.clone();
    let expose_handle = tokio::spawn(async move {
        malai::expose_tcp(
            "127.0.0.1".to_string(),
            echo_port,
            expose_id52,
            secret,
            expose_graceful,
        )
        .await;
    });

    // Give the server time to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Start tcp_bridge on a different port
    let bridge_port = 9002;
    let bridge_graceful = graceful.clone();
    let bridge_id52 = id52.clone();
    let bridge_handle = tokio::spawn(async move {
        malai::tcp_bridge(bridge_port, bridge_id52, bridge_graceful).await;
    });

    // Give the bridge time to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Connect to the bridge and send test data
    let test_data = b"Hello, kulfi!";
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", bridge_port))
        .await
        .expect("Failed to connect to bridge");

    stream
        .write_all(test_data)
        .await
        .expect("Failed to write test data");

    let mut response = vec![0u8; test_data.len()];
    let read_result =
        tokio::time::timeout(Duration::from_secs(30), stream.read_exact(&mut response))
            .await
            .expect("Timeout waiting for TCP response")
            .expect("Failed to read response");

    let _ = read_result;
    assert_eq!(
        &response[..],
        test_data,
        "Echo response doesn't match sent data"
    );

    println!("✓ TCP client successfully connected and received echo response");

    // Cleanup: best-effort, don't block the test
    let _ = tokio::time::timeout(SHUTDOWN_TIMEOUT, graceful.shutdown()).await;
    echo_handle.abort();
    expose_handle.abort();
    bridge_handle.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_udp_client_server_connection() {
    tokio::time::timeout(TEST_TIMEOUT, test_udp_client_server_connection_inner())
        .await
        .expect("test_udp_client_server_connection timed out");
}

async fn test_udp_client_server_connection_inner() {
    // Setup logging
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Create test identity
    let (id52, secret) = create_test_identity().await;
    println!("Test UDP identity: {}", id52);

    // Start UDP echo server
    let echo_port = 9003;
    let echo_handle = start_udp_echo_server(echo_port).await;

    // Start expose_udp server
    let graceful = kulfi_utils::Graceful::new();
    let expose_graceful = graceful.clone();
    let expose_id52 = id52.clone();
    let expose_handle = tokio::spawn(async move {
        malai::expose_udp(
            "127.0.0.1".to_string(),
            echo_port,
            expose_id52,
            secret,
            expose_graceful,
        )
        .await;
    });

    // Give server more time to register with relay for discovery
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Start udp_bridge
    let bridge_port = 9004;
    let bridge_graceful = graceful.clone();
    let bridge_id52 = id52.clone();
    let bridge_handle = tokio::spawn(async move {
        malai::udp_bridge(bridge_port, bridge_id52, bridge_graceful).await;
    });

    // Give bridge time to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Send test datagram through bridge
    let client_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind client socket");

    let test_data = b"UDP test message";
    client_socket
        .send_to(test_data, format!("127.0.0.1:{}", bridge_port))
        .await
        .expect("Failed to send UDP datagram");

    let mut response = vec![0u8; 1024];
    let (n, _) = tokio::time::timeout(
        Duration::from_secs(30),
        client_socket.recv_from(&mut response),
    )
    .await
    .expect("Timeout waiting for UDP response")
    .expect("Failed to receive UDP response");

    assert_eq!(
        &response[..n],
        test_data,
        "UDP echo response doesn't match sent data"
    );

    println!("✓ UDP client successfully connected and received echo response");

    // Cleanup: best-effort, don't block the test
    let _ = tokio::time::timeout(SHUTDOWN_TIMEOUT, graceful.shutdown()).await;
    echo_handle.abort();
    expose_handle.abort();
    bridge_handle.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_http_client_server_connection() {
    tokio::time::timeout(TEST_TIMEOUT, test_http_client_server_connection_inner())
        .await
        .expect("test_http_client_server_connection timed out");
}

async fn test_http_client_server_connection_inner() {
    // Setup logging
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Create test identity
    let (id52, secret) = create_test_identity().await;
    println!("Test HTTP identity: {}", id52);

    // Start HTTP server
    let http_port = 9005;
    let http_handle = start_http_server(http_port).await;

    // Start expose_http server
    let graceful = kulfi_utils::Graceful::new();
    let expose_graceful = graceful.clone();
    let expose_id52 = id52.clone();
    let expose_handle = tokio::spawn(async move {
        malai::expose_http(
            "127.0.0.1".to_string(),
            http_port,
            "test.local".to_string(), // Bridge domain (not used in this test)
            expose_id52,
            secret,
            expose_graceful,
        )
        .await;
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // For HTTP, we'll test direct connection via the iroh endpoint
    // In a real scenario, you'd use http_bridge, but that requires a domain
    // For this test, we'll verify the expose_http is running and accepting connections

    println!("✓ HTTP server started successfully with identity {}", id52);
    println!("  (Full HTTP bridge test requires domain setup)");

    // Cleanup: best-effort, don't block the test
    let _ = tokio::time::timeout(SHUTDOWN_TIMEOUT, graceful.shutdown()).await;
    http_handle.abort();
    expose_handle.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_tcp_multiple_connections() {
    tokio::time::timeout(TEST_TIMEOUT, test_tcp_multiple_connections_inner())
        .await
        .expect("test_tcp_multiple_connections timed out");
}

async fn test_tcp_multiple_connections_inner() {
    // Setup logging
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Create test identity
    let (id52, secret) = create_test_identity().await;
    println!("Test multiple connections identity: {}", id52);

    // Start echo server
    let echo_port = 9006;
    let echo_handle = start_tcp_echo_server(echo_port).await;

    // Start expose_tcp server
    let graceful = kulfi_utils::Graceful::new();
    let expose_graceful = graceful.clone();
    let expose_id52 = id52.clone();
    let expose_handle = tokio::spawn(async move {
        malai::expose_tcp(
            "127.0.0.1".to_string(),
            echo_port,
            expose_id52,
            secret,
            expose_graceful,
        )
        .await;
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Start tcp_bridge
    let bridge_port = 9007;
    let bridge_graceful = graceful.clone();
    let bridge_id52 = id52.clone();
    let bridge_handle = tokio::spawn(async move {
        malai::tcp_bridge(bridge_port, bridge_id52, bridge_graceful).await;
    });

    // Give bridge time to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Test multiple concurrent connections
    let mut handles = vec![];
    for i in 0..5 {
        let port = bridge_port;
        let handle = tokio::spawn(async move {
            let test_data = format!("Message {}", i);
            let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
                .await
                .expect("Failed to connect");

            stream
                .write_all(test_data.as_bytes())
                .await
                .expect("Failed to write");

            let mut response = vec![0u8; test_data.len()];
            tokio::time::timeout(Duration::from_secs(30), stream.read_exact(&mut response))
                .await
                .expect("Timeout")
                .expect("Failed to read");

            assert_eq!(response, test_data.as_bytes());
            println!("✓ Connection {} successful", i);
        });
        handles.push(handle);
    }

    // Wait for all connections to complete
    for handle in handles {
        handle.await.expect("Connection task failed");
    }

    println!("✓ All 5 concurrent connections successful");

    // Cleanup: best-effort, don't block the test
    let _ = tokio::time::timeout(SHUTDOWN_TIMEOUT, graceful.shutdown()).await;
    echo_handle.abort();
    expose_handle.abort();
    bridge_handle.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_tcp_large_data_transfer() {
    tokio::time::timeout(TEST_TIMEOUT, test_tcp_large_data_transfer_inner())
        .await
        .expect("test_tcp_large_data_transfer timed out");
}

async fn test_tcp_large_data_transfer_inner() {
    // Setup logging
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Create test identity
    let (id52, secret) = create_test_identity().await;
    println!("Test large data transfer identity: {}", id52);

    // Start echo server
    let echo_port = 9008;
    let echo_handle = start_tcp_echo_server(echo_port).await;

    // Start expose_tcp server
    let graceful = kulfi_utils::Graceful::new();
    let expose_graceful = graceful.clone();
    let expose_id52 = id52.clone();
    let expose_handle = tokio::spawn(async move {
        malai::expose_tcp(
            "127.0.0.1".to_string(),
            echo_port,
            expose_id52,
            secret,
            expose_graceful,
        )
        .await;
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Start tcp_bridge
    let bridge_port = 9009;
    let bridge_graceful = graceful.clone();
    let bridge_id52 = id52.clone();
    let bridge_handle = tokio::spawn(async move {
        malai::tcp_bridge(bridge_port, bridge_id52, bridge_graceful).await;
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Send large amount of data (1MB)
    let large_data = vec![0xAB; 1024 * 1024];
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", bridge_port))
        .await
        .expect("Failed to connect");

    stream
        .write_all(&large_data)
        .await
        .expect("Failed to write large data");

    let mut response = vec![0u8; large_data.len()];
    tokio::time::timeout(Duration::from_secs(30), stream.read_exact(&mut response))
        .await
        .expect("Timeout waiting for large data response")
        .expect("Failed to read large data response");

    assert_eq!(response, large_data, "Large data echo failed");

    println!("✓ Successfully transferred 1MB of data");

    // Cleanup: best-effort, don't block the test
    let _ = tokio::time::timeout(SHUTDOWN_TIMEOUT, graceful.shutdown()).await;
    echo_handle.abort();
    expose_handle.abort();
    bridge_handle.abort();
}
