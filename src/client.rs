//! a small client.
//!
//! based on: <https://github.com/hyperium/hyper/blob/master/examples/client.rs>

use {
    crate::{rt::TokioIo, Error},
    bytes::Bytes,
    http::{response, Uri},
    http_body_util::Empty,
    hyper::{body::Incoming, Request},
    std::time::Duration,
    tap::{Pipe, Tap, TapFallible},
    tokio::net::TcpStream,
    tracing::{error, info},
};

/// the client.
pub struct Client;

/// client parameters.
pub struct Params {
    url: Uri,
    interval: Duration,
}

// === impl Client ===

impl Client {
    /// runs the client.
    ///
    /// sends a request to the server at the given interval.
    pub async fn run(Params { url, interval }: Params) {
        let mut interval = tokio::time::interval(interval);
        loop {
            interval.tick().await;
            Self::send_request(&url)
                .tap(|_| info!(%url, "sending request"))
                .await
                .tap_ok(|_| info!(%url, "sent request"))
                .tap_err(|err| error!(?err, %url, "failed to send request"))
                .ok();
        }
    }

    /// sends a request.
    pub async fn send_request(url: &Uri) -> Result<(), Error> {
        // connect to the server.
        let conn = {
            let host = url.host().expect("uri has no host");
            let port = url.port_u16().unwrap_or(80);
            let addr = format!("{}:{}", host, port);
            let stream = TcpStream::connect(addr).await?;
            TokioIo::new(stream)
        };

        // perform a handshake.
        let (mut sender, conn) = hyper::client::conn::http1::handshake(conn).await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                tracing::error!("Connection failed: {:?}", err);
            }
        });

        // construct a request.
        let req = {
            let authority = url.authority().unwrap().clone();
            let path = url.path();
            Request::builder()
                .uri(path)
                .header(hyper::header::HOST, authority.as_str())
                .body(Empty::<Bytes>::new())?
        };

        // send the request.
        let res = sender.send_request(req).await?;
        let (
            response::Parts {
                status, headers, ..
            },
            body,
        ) = res.into_parts();

        // collect the response body.
        let start = std::time::Instant::now();
        let body = <Incoming as http_body_util::BodyExt>::collect(body)
            .await
            .unwrap()
            .to_bytes();
        let duration = start.elapsed();
        let len = body.len();

        tracing::info!(
            %status,
            ?headers,
            body.duration_ms = %duration.as_millis(),
            body.len = %len,
            "received response"
        );

        Ok(())
    }
}

// === impl Params ===

impl Params {
    const INTERVAL_ENV: &'static str = "TITRATE_INTERVAL_MS";
    const SERVER_ENV: &'static str = "TITRATE_SERVER";

    /// parses client parameters from the process environment.
    ///
    /// # Panics
    ///
    /// this will panic if any of the environment variables are not set, or if they contain invalid
    /// data.
    pub fn new_from_env() -> Self {
        let interval = std::env::var(Self::INTERVAL_ENV)
            .expect("`TITRATE_INTERVAL_MS` env var should be provided")
            .parse::<u64>()
            .unwrap()
            .pipe(Duration::from_millis);

        let url = std::env::var(Self::SERVER_ENV)
            .expect("`TITRATE_SERVER` env var should be provided")
            .parse::<Uri>()
            .expect("`TITRATE_SERVER` env var should be a url");

        Self { url, interval }
    }
}
