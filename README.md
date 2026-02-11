# Kulfi & Malai

Open Source, General Purpose, Sovereign, Decentralized, Peer to Peer Internet.

---

## Highlights

- Share your local HTTP/TCP with anyone, without any central server.
- Host your own HTTP bridge using `malai http-bridge` to access exposed services via web browser.
- Built on top of [iroh][iroh], a p2p networking library.

## Malai

Malai is a simple tool that can be used to expose any local service (HTTP, TCP
and, SSH, etc.) to the world. It can be paired up with an ACL system (like
Kulfi) to control access to the exposed services.

### Installation

```bash
# Clone the repository
git clone https://github.com/ovo-Tim/kulfi
cd kulfi

# Build malai
cargo build --release -p malai

# The binary will be available at target/release/malai
```

### Quick Start

**Step 1:** Run an HTTP bridge (on a server with a public domain):
```bash
# On your server (e.g., bridge.example.com)
malai http-bridge --port 80
```

**Step 2:** Expose a local HTTP service (e.g., running on port 3000):
```bash
# On your local machine
malai http 3000 --bridge bridge.example.com --public
```

This will generate or use an existing identity and expose your service. The service
will be accessible via `https://<your-id52>.bridge.example.com` through your HTTP bridge.

### Commands

#### HTTP Service Exposure

Expose a local HTTP service to the kulfi network:
```bash
malai http <PORT> [OPTIONS]

Options:
  --host <HOST>      Host serving the HTTP service [default: 127.0.0.1]
  --bridge <BRIDGE>  HTTP bridge domain to use (required for web access) [env: MALAI_HTTP_BRIDGE]
  --public           Make the service public (required)
```

Example:
```bash
malai http 8080 --host 127.0.0.1 --bridge bridge.example.com --public
```

**Note:** You need to run your own HTTP bridge for web browser access. See the HTTP Bridge section below.

#### TCP Service Exposure

Expose a local TCP service (SSH, database, etc.):
```bash
malai tcp <PORT> [OPTIONS]

Options:
  --host <HOST>  Host serving the TCP service [default: 127.0.0.1]
  --public       Make the service public (required)
```

Example:
```bash
malai tcp 22 --public  # Expose SSH
```

#### Folder Sharing

Share a folder over HTTP:
```bash
malai folder <PATH> [OPTIONS]

Options:
  --bridge <BRIDGE>  HTTP bridge domain to use (required for web access) [env: MALAI_HTTP_BRIDGE]
  --public           Make the folder public (required)
```

Example:
```bash
malai folder ./documents --bridge bridge.example.com --public
```

**Note:** You need to run your own HTTP bridge for web browser access.

#### Browse Kulfi Sites

Open a kulfi URL in your browser:
```bash
malai browse kulfi://<id52>/<path>
```

#### HTTP Bridge

Run an HTTP bridge server that forwards requests to kulfi services. **You must host your own bridge** to access HTTP services via web browser.

```bash
malai http-bridge [OPTIONS]

Options:
  -t, --proxy-target <ID52>  Forward to specific id52 (optional)
  -p, --port <PORT>          Port to listen on [default: 0 for random]
```

**Setting up your bridge:**
1. Get a server with a public IP and domain (e.g., `bridge.example.com`)
2. Configure wildcard DNS: `*.bridge.example.com` → your server IP
3. Run the bridge: `malai http-bridge --port 80` (or use a reverse proxy with SSL)
4. Use `--bridge bridge.example.com` when exposing services
5. Access services at: `https://<id52>.bridge.example.com`

#### TCP Bridge

Run a TCP bridge server:
```bash
malai tcp-bridge <PROXY_TARGET> [PORT]
```

#### HTTP Proxy

Run an HTTP proxy (requires a remote proxy server):
```bash
malai http-proxy-remote --public  # On remote server
malai http-proxy <REMOTE_ID52> --port 8080  # On local machine
```

#### Identity Management

Generate a new identity:
```bash
malai keygen [-f <FILE>]
```

Create identity in system keyring:
```bash
malai identity create [-f <FILE>]
```

Delete identity from system keyring:
```bash
malai identity delete --id52 <ID52>
# or
malai identity delete --file <FILE>
```

### Configuration File

For running multiple services, create a `malai.toml` file:

```toml
[malai]
log = "/var/log/malai.log"  # Optional: log file path

[http.my_web_app]
identity = "id52_abc123..."  # Optional: specific identity
secret_file = "/path/to/secret"  # Optional: load identity from file
port = 3000
host = "127.0.0.1"
bridge = "bridge.example.com"  # Your HTTP bridge domain
public = true
active = true

[http.another_service]
port = 8080
public = true
active = true

[tcp.ssh_service]
port = 22
host = "127.0.0.1"
public = true
active = true
```

Run all services from config:
```bash
malai run --home /path/to/config/dir
# or
malai run --home /path/to/malai.toml
# or set MALAI_HOME environment variable
export MALAI_HOME=/path/to/config/dir
malai run
```

### Identity System

Malai uses `id52` identities for peer-to-peer connections. Identities can be:

1. **Generated on-the-fly**: If no identity exists, one is created automatically
2. **Stored in system keyring**: Use `malai identity create` for persistent identities
3. **Stored in files**: Use secret key files with `secret_file` option in config
4. **Specified per service**: Each service in `malai.toml` can use a different identity

### How HTTP Bridge Works

An HTTP bridge allows you to access kulfi services through standard web browsers. **You must host your own bridge** on a server with a public domain.

**Setup:**
1. Get a domain (e.g., `bridge.example.com`) pointing to your server
2. Configure wildcard DNS: `*.bridge.example.com` → server IP
3. Run bridge: `malai http-bridge --port 80`
4. Optionally use a reverse proxy (nginx/caddy) for automatic HTTPS

**Usage:**
1. Expose service with bridge: `malai http 3000 --bridge bridge.example.com --public`
2. Your service gets a unique `id52` (e.g., `abc123...xyz`)
3. Access via: `https://abc123...xyz.bridge.example.com`
4. The bridge forwards requests to your local service via the kulfi P2P network

**Why it's needed:** Web browsers can't directly connect to kulfi's P2P protocol. The bridge acts as a gateway, translating HTTP requests to kulfi connections using the subdomain as the target `id52`.

### Environment Variables

- `MALAI_HTTP_BRIDGE`: Default HTTP bridge domain for your services (set to your bridge domain)
- `MALAI_HOME`: Default configuration directory for `malai run`

Example:
```bash
export MALAI_HTTP_BRIDGE=bridge.example.com
malai http 3000 --public  # Will use bridge.example.com automatically
```

### Security Notes

- The `--public` flag is required for all service exposure commands as a safety measure
- Each service can use a separate identity for access control
- Identities can be managed through the system keyring for security
- Services not marked as `active = true` in config will not start

### Common Use Cases

#### Share a Local Development Server

```bash
# First, ensure you have an HTTP bridge running on a public server
# On your server: malai http-bridge --port 80

# Start your dev server (e.g., React on port 3000)
npm start

# In another terminal, expose it
malai http 3000 --bridge bridge.example.com --public
# Access via https://<your-id52>.bridge.example.com
```

#### Remote SSH Access

```bash
# Expose SSH service
malai tcp 22 --public

# On another machine, create a TCP bridge
malai tcp-bridge <your-id52> 2222

# Connect via the bridge
ssh user@localhost -p 2222
```

#### Share Files Quickly

```bash
# Share current directory (requires HTTP bridge)
malai folder . --bridge bridge.example.com --public

# Others can browse via https://<your-id52>.bridge.example.com
```

#### Run Multiple Services

Create `malai.toml`:
```toml
[http.blog]
port = 8080
public = true
active = true

[http.api]
port = 3000
public = true
active = true

[tcp.database]
port = 5432
public = true
active = false  # Disabled for now
```

Run all active services:
```bash
malai run
```

## Kulfi

Kulfi is a peer to peer network, free from any corporate control. Data stays
with the user, and devices controlled by the user, and not with some central
company.

Kulfi will soon be available as an binary that you can download and run on your
computer. We will support Linux, Windows and MacOS from day one. We also want to
create Apps that can be distributed through App Stores, and also support mobile
devices.


`kulfi` and `malai` are built on top of [iroh][iroh], and uses [BitTorrent's
Mainline DHT][MainlineDHT] for peer discovery.


[iroh]: https://www.iroh.computer

[MainlineDHT]: https://en.wikipedia.org/wiki/Mainline_DHT

## Licence

This project is licensed under the [UPL](LICENSE) license. UPL is MIT like
license, with Apache 2.0 like patent grant clause.

## Contributing

We welcome contributions to Kulfi & Malai. Please read the
[CONTRIBUTING.md][cont] file for details on how to contribute.

[cont]: CONTRIBUTING.md
