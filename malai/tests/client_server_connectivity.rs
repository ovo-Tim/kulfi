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
fn create_test_identity() -> (String, kulfi_id52::SecretKey) {
    let secret = kulfi_id52::SecretKey::generate();
    let id52 = secret.id52();
    (id52, secret)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_tcp_echo_connection() {
    tokio::time::timeout(TEST_TIMEOUT, test_tcp_echo_connection_inner())
        .await
        .expect("test_tcp_echo_connection timed out");
}

async fn test_tcp_echo_connection_inner() {
    // Setup logging for debugging
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Create test identity
    let (id52, secret) = create_test_identity();
    println!("Test identity: {}", id52);

    // Start a simple TCP echo server
    let echo_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind echo server");
    let echo_port = echo_listener.local_addr().unwrap().port();
    println!("Echo server on port {}", echo_port);

    let echo_handle = tokio::spawn(async move {
        if let Ok((mut socket, _)) = echo_listener.accept().await {
            let mut buf = vec![0u8; 1024];
            if let Ok(n) = socket.read(&mut buf).await {
                let _ = socket.write_all(&buf[..n]).await;
            }
        }
    });

    // Start expose_tcp server
    let graceful = kulfi_utils::Graceful::new();
    let expose_graceful = graceful.clone();
    let expose_id52 = id52.clone();
    let expose_host = "127.0.0.1".to_string();

    let expose_handle = tokio::spawn(async move {
        malai::expose_tcp(expose_host, echo_port, expose_id52, secret, expose_graceful).await;
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Start tcp_bridge
    let bridge_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind bridge");
    let bridge_port = bridge_listener.local_addr().unwrap().port();

    let bridge_id52 = id52.clone();
    let bridge_graceful = graceful.clone();

    // Manual bridge implementation — use a fresh endpoint (not the global singleton)
    // to avoid cross-test contamination when tokio runtimes are recycled.
    let bridge_handle = tokio::spawn(async move {
        if let Ok((local_stream, _)) = bridge_listener.accept().await {
            let endpoint = iroh::Endpoint::builder()
                .discovery(iroh::discovery::pkarr::PkarrPublisher::n0_dns())
                .discovery(iroh::discovery::dns::DnsDiscovery::n0_dns())
                .discovery(iroh::discovery::mdns::MdnsDiscovery::builder())
                .alpns(vec![kulfi_utils::APNS_IDENTITY.into()])
                .bind()
                .await
                .expect("failed to create bridge iroh endpoint");
            let peer_connections = kulfi_utils::PeerStreamSenders::default();

            let _ = kulfi_utils::tcp_to_peer(
                kulfi_utils::Protocol::Tcp.into(),
                endpoint,
                local_stream,
                &bridge_id52,
                peer_connections,
                bridge_graceful,
            )
            .await;
        }
    });

    // Give bridge time to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Connect and test
    let test_data = b"Hello, kulfi!";
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", bridge_port))
        .await
        .expect("Failed to connect to bridge");

    stream
        .write_all(test_data)
        .await
        .expect("Failed to write test data");

    let mut response = vec![0u8; test_data.len()];
    let result =
        tokio::time::timeout(Duration::from_secs(30), stream.read_exact(&mut response)).await;

    match result {
        Ok(Ok(_)) => {
            assert_eq!(&response[..], test_data, "Echo response mismatch");
            println!("✓ TCP echo test passed!");
        }
        Ok(Err(e)) => panic!("Read error: {}", e),
        Err(_) => panic!("Timeout waiting for response"),
    }

    // Cleanup: best-effort, don't block the test
    drop(stream);
    let _ = tokio::time::timeout(SHUTDOWN_TIMEOUT, graceful.shutdown()).await;
    echo_handle.abort();
    expose_handle.abort();
    bridge_handle.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_udp_echo_connection() {
    tokio::time::timeout(TEST_TIMEOUT, test_udp_echo_connection_inner())
        .await
        .expect("test_udp_echo_connection timed out");
}

async fn test_udp_echo_connection_inner() {
    // Setup logging
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Create test identity
    let (id52, secret) = create_test_identity();
    println!("Test UDP identity: {}", id52);

    // Start UDP echo server
    let echo_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind UDP echo server");
    let echo_port = echo_socket.local_addr().unwrap().port();
    println!("UDP echo server on port {}", echo_port);

    let echo_handle = tokio::spawn(async move {
        let mut buf = vec![0u8; 65535];
        if let Ok((n, addr)) = echo_socket.recv_from(&mut buf).await {
            let _ = echo_socket.send_to(&buf[..n], addr).await;
        }
    });

    // Start expose_udp server
    let graceful = kulfi_utils::Graceful::new();
    let expose_graceful = graceful.clone();
    let expose_id52 = id52.clone();
    let expose_host = "127.0.0.1".to_string();

    let expose_handle = tokio::spawn(async move {
        malai::expose_udp(expose_host, echo_port, expose_id52, secret, expose_graceful).await;
    });

    // Give server more time to register with relay for discovery
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Start udp_bridge
    let bridge_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind bridge socket");
    let bridge_port = bridge_socket.local_addr().unwrap().port();

    let bridge_id52 = id52.clone();
    let bridge_graceful = graceful.clone();

    // Simplified bridge for testing
    let bridge_socket_arc = std::sync::Arc::new(bridge_socket);
    let bridge_socket_clone = bridge_socket_arc.clone();
    let bridge_handle = tokio::spawn(async move {
        let mut buf = vec![0u8; 65535];
        if let Ok((n, client_addr)) = bridge_socket_clone.recv_from(&mut buf).await {
            let endpoint = iroh::Endpoint::builder()
                .discovery(iroh::discovery::pkarr::PkarrPublisher::n0_dns())
                .discovery(iroh::discovery::dns::DnsDiscovery::n0_dns())
                .discovery(iroh::discovery::mdns::MdnsDiscovery::builder())
                .alpns(vec![kulfi_utils::APNS_IDENTITY.into()])
                .bind()
                .await
                .expect("failed to create bridge iroh endpoint");
            let peer_connections = kulfi_utils::PeerStreamSenders::default();
            let data = buf[..n].to_vec();

            let _ = kulfi_utils::udp_to_peer(
                kulfi_utils::Protocol::Udp.into(),
                endpoint,
                bridge_socket_arc.clone(),
                client_addr,
                data,
                &bridge_id52,
                peer_connections,
                bridge_graceful,
            )
            .await;
        }
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Send test datagram
    let client_socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind client socket");

    let test_data = b"UDP test message";
    client_socket
        .send_to(test_data, format!("127.0.0.1:{}", bridge_port))
        .await
        .expect("Failed to send UDP datagram");

    let mut response = vec![0u8; 1024];
    let result = tokio::time::timeout(
        Duration::from_secs(30),
        client_socket.recv_from(&mut response),
    )
    .await;

    match result {
        Ok(Ok((n, _))) => {
            assert_eq!(&response[..n], test_data, "UDP echo response mismatch");
            println!("✓ UDP echo test passed!");
        }
        Ok(Err(e)) => panic!("UDP receive error: {}", e),
        Err(_) => panic!("Timeout waiting for UDP response"),
    }

    // Cleanup: best-effort, don't block the test
    let _ = tokio::time::timeout(SHUTDOWN_TIMEOUT, graceful.shutdown()).await;
    echo_handle.abort();
    expose_handle.abort();
    bridge_handle.abort();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multiple_tcp_streams() {
    tokio::time::timeout(TEST_TIMEOUT, test_multiple_tcp_streams_inner())
        .await
        .expect("test_multiple_tcp_streams timed out");
}

async fn test_multiple_tcp_streams_inner() {
    // Setup logging
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // Create test identity
    let (id52, secret) = create_test_identity();
    println!("Test multiple streams identity: {}", id52);

    // Start echo server that handles multiple connections
    let echo_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind echo server");
    let echo_port = echo_listener.local_addr().unwrap().port();

    let echo_handle = tokio::spawn(async move {
        for _ in 0..3 {
            if let Ok((mut socket, _)) = echo_listener.accept().await {
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 1024];
                    if let Ok(n) = socket.read(&mut buf).await {
                        let _ = socket.write_all(&buf[..n]).await;
                    }
                });
            }
        }
    });

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

    // Start tcp_bridge — bind temporarily to get a free port, then drop so tcp_bridge can bind it
    let bridge_graceful = graceful.clone();
    let bridge_id52 = id52.clone();
    let bridge_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind bridge");
    let bridge_port = bridge_listener.local_addr().unwrap().port();
    drop(bridge_listener);

    let bridge_handle = tokio::spawn(async move {
        malai::tcp_bridge(bridge_port, bridge_id52, bridge_graceful).await;
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Test 3 concurrent connections
    let mut handles = vec![];
    for i in 0..3 {
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
            println!("✓ Stream {} completed", i);
        });
        handles.push(handle);
    }

    // Wait for all streams
    for handle in handles {
        handle.await.expect("Stream task failed");
    }

    println!("✓ All 3 streams completed successfully");

    // Cleanup: best-effort, don't block the test
    let _ = tokio::time::timeout(SHUTDOWN_TIMEOUT, graceful.shutdown()).await;
    echo_handle.abort();
    expose_handle.abort();
    bridge_handle.abort();
}
