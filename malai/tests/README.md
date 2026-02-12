# Integration Tests

This directory contains integration tests for the malai crate, verifying end-to-end connectivity through the kulfi P2P network.

## Test Files

### `client_server_connectivity.rs`
Tests basic client-server connectivity patterns:

- **`test_tcp_echo_connection`** ✅ - Verifies TCP client can connect to local server
  - Creates a TCP echo server on localhost
  - Starts `expose_tcp` to share the service over kulfi
  - Connects through the P2P network
  - Sends data and verifies echo response

- **`test_udp_echo_connection`** - Verifies UDP datagram forwarding
  - Creates a UDP echo server
  - Uses `expose_udp` to share over kulfi
  - Sends datagrams and verifies responses

- **`test_multiple_tcp_streams`** - Verifies concurrent connections
  - Tests 3 simultaneous TCP streams over a single kulfi connection
  - Demonstrates stream multiplexing

### `integration_tests.rs`
Comprehensive tests with longer scenarios:

- TCP echo with full bridge setup
- UDP datagram forwarding
- HTTP server connectivity
- Large data transfers (1MB)
- Multiple concurrent connections

## Running Tests

```bash
# Run all integration tests
cargo test -p malai --test client_server_connectivity

# Run a specific test
cargo test -p malai --test client_server_connectivity test_tcp_echo_connection -- --exact

# Run with output visible
cargo test -p malai --test client_server_connectivity -- --nocapture

# Run tests sequentially (recommended to avoid port conflicts)
cargo test -p malai --test client_server_connectivity -- --test-threads=1
```

## Test Architecture

Each test follows this pattern:

1. **Generate test identity** - Creates an Ed25519 keypair for the test service
2. **Start local server** - Spins up a simple echo server (TCP/UDP)
3. **Start expose service** - Runs `expose_tcp`/`expose_udp` to share the service
4. **Start bridge** - Creates a local bridge to connect to the remote service
5. **Test connectivity** - Sends data and verifies responses
6. **Cleanup** - Gracefully shuts down all services

## Connection Flow

```
Client → Bridge → Kulfi P2P → Expose → Local Server
  |        |         |           |          |
  |        └─────────┴───────────┘          |
  |              iroh network               |
  └──────────────────────────────────────────┘
             Data flow verified
```

## Key Concepts Tested

- **P2P Connection Establishment** - Tests verify that iroh can establish connections between peers
- **Protocol Multiplexing** - Multiple streams over a single iroh connection
- **Stream Piping** - Data correctly flows between local sockets and iroh streams
- **Echo Verification** - Ensures data integrity through the full roundtrip

## Notes

- Tests use randomly assigned ports to avoid conflicts
- Each test creates a fresh identity to avoid interference
- Tests include 2-second delays to allow services to initialize
- Connection manager ping mechanism is tested implicitly (12-second intervals)
- Tests demonstrate both relay and direct connections (iroh's automatic fallback)
