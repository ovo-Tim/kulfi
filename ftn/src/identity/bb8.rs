//! why do we need connection pooling?
//! ==================================
//!
//! let's first understand the types of connection we make over the network.
//!
//! every running ftn app opens one "connection" with the relay at startup. if the connection is
//! lost, it tries to reconnect. in `iroh` terms, this is called `iroh::Endpoint`. this is a
//! connection with the relay, and it is a long-lived connection. this is still not a
//! `iroh::Connection`, which is the connection we want to pool, so at startup we do not have
//! any `iroh::Connection`, but `iroh::Endpoint`. which is a different kind of connection, and it
//! must remain open for the lifetime of the app.
//!
//! let's call this the "jane's side". let's say jane has a friend, bob, and bob wants to connect to
//! the fastn server that is running on the jane's computer, listening at the loopback address.
//! bob cannot directly connect to it, so we need iroh.
//!
//! so when bob tries to access jane's fastn, bob starts an endpoint instance, and issues the
//! `endpoint.connect(jane_identity)` call. this creates two `iroh::Connection` instances, one on
//! bob's side and the other on the jane's side.
//!
//! in `iroh` world, once a connection is established, either party can initiate a stream, but the
//! other party must accept the stream. in our case, bob's side initiates the stream, and the
//! alice's side accepts the stream.
//!
//! let's recap the control flow. bob wants to access alice's fastn, so bob opens
//! https://<alice-id>.localhost.direct on their browser. localhost.direct distributes their
//! wildcard domain certificate[1], and maps *.localhost.direct to 127.0.0.1, so the request from bob's
//! browser lands on their own machine, to port 443, where bob's ftn-http-proxy is running. bob's
//! ftn-http-proxy gets the `alice-id` and is the main actor here, it gets an HTTP request, and for
//! that it request it creates an endpoint, initiates a connection, and creates a bidirectional
//! stream. on the stream it then writes the HTTP request, and waits for the response from the
//! alice's side, and converts it back as an HTTP response and send it to the browser.
//!
//! now creating the endpoint, and the connection takes time, of the order of a second or so. doing
//! that on every incoming HTTP request will make our system very slow. so we want to reuse the
//! existing connection. creating a bidirectional stream on existing connection is an inexpensive
//! operation, so we want to keep the connection alive, and reuse it to handle multiple HTTP requests.
//!
//! we also do not want to keep the connection open forever, bob may stop browsing alice's site, and
//! alice may have many friends, and they may all want to interact with her fastn server. so if we
//! leave the connection open forever, alice's machine will slowly get overwhelmed with connections.
//!
//! [1]: https://get.localhost.direct
//!
//! why did we pick bb8?
//! ====================
//!
//! when picking `bb8,` I evaluated various crates for connection pooling, namely `r2d2`, `bb8`,
//! `deadpool`. `r2d2` is the most popular, but it is not async, and we need async. `deadpool` is
//! async, but it does not cleanly support idle timeout and max connections, it does have facilities
//! through which you can implement them, but it is not as clean as `bb8`. `bb8` is a fork of `r2d2`
//! that is async, and it has a clean API for idle timeout and max connections.

use eyre::WrapErr;

impl bb8::ManageConnection for ftn::Identity {
    type Connection = iroh::endpoint::Connection;
    type Error = eyre::Error;

    fn connect(&self) -> impl Future<Output = Result<Self::Connection, Self::Error>> + Send {
        Box::pin(async move {
            // creating a new endpoint takes about 30 milliseconds, so we can do it here.
            // since we create just a single connection via this endpoint, the overhead is
            // negligible, compared to 800 milliseconds or so it takes to create a new connection.
            let ep = get_endpoint(self.public_key.to_string().as_str())
                .await
                .wrap_err_with(|| "failed to bind to iroh network")?;
            ep.connect(self.public_key, ftn::APNS)
                .await
                .map_err(|e| eyre::anyhow!("failed to connect to iroh network: {e}"))
        })
    }

    fn is_valid(
        &self,
        _conn: &mut Self::Connection,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        // TODO: send a ping and wait for a pong
        Box::pin(async move { Ok(()) })
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

pub async fn get_endpoint(id: &str) -> eyre::Result<iroh::Endpoint> {
    let secret_key = ftn::utils::get_secret(id)
        .wrap_err_with(|| format!("failed to get secret key from keychain for {id}"))?;

    match iroh::Endpoint::builder()
        .discovery_n0()
        .alpns(vec![ftn::APNS.into()])
        .secret_key(secret_key)
        .bind()
        .await
    {
        Ok(ep) => Ok(ep),
        Err(e) => {
            // https://github.com/n0-computer/iroh/issues/2741
            // this is why you MUST NOT use anyhow::Error etc. in library code.
            Err(eyre::anyhow!("failed to bind to iroh network: {e}"))
        }
    }
}
