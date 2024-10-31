//! a small http/2 server.
//!
//! loosely based on: <https://github.com/hyperium/hyper/blob/master/examples/hello-http2.rs>

use {
    crate::{body::ResponseBody, rt::TokioIo, Error},
    hyper::Response,
    std::{net::SocketAddr, sync::Arc},
    tap::{Pipe, Tap, TapFallible},
    tokio::{
        net::{TcpListener, TcpStream},
        task::spawn,
    },
};

/// the server.
#[derive(Clone)]
pub struct Server;

/// server parameters.
#[derive(Debug)]
pub struct Params {
    pub port: u16,
    pub body_size: usize,
    pub frame_size: usize,
}

struct Connection {
    params: Arc<Params>,
}

// === impl Server ===

impl Server {
    /// runs the server.
    pub async fn run(params: Params) -> Result<(), Error> {
        tracing::info!(port = %params.port, "starting server");
        let addr = SocketAddr::from(([0, 0, 0, 0], params.port));
        let listener = TcpListener::bind(addr).await?;
        let params = Arc::new(params);
        tracing::info!(port = %params.port, "server is listening");

        loop {
            let (stream, addr) = listener.accept().await?;
            let io = TokioIo::new(stream);
            Arc::clone(&params)
                .pipe(Connection::new)
                .tap(|_| tracing::info!(?addr, "accepted a new connection"))
                .run(io)
                .pipe(spawn);
        }
    }
}

// === impl Connection ===

impl Connection {
    fn new(params: Arc<Params>) -> Self {
        Self { params }
    }

    async fn run(self, io: TokioIo<TcpStream>) {
        hyper::server::conn::http1::Builder::new()
            .timer(hyper_util::rt::TokioTimer::new())
            .serve_connection(io, self)
            .await
            .tap_err(|err| tracing::error!("Error serving connection: {:?}", err))
            .tap_ok(|_| tracing::info!("finished serving connection"))
            .ok();
    }
}

impl<B> hyper::service::Service<B> for Connection {
    type Error = Error;
    type Response = Response<ResponseBody>;
    type Future = std::future::Ready<Result<Self::Response, Self::Error>>;
    fn call(&self, _: B) -> Self::Future {
        let Self { params } = self;

        ResponseBody::new(params)
            .pipe(Response::new)
            .pipe(Ok)
            .pipe(std::future::ready)
    }
}

// === impl Params ===

impl Params {
    const PORT_ENV: &'static str = "TITRATE_PORT";
    const BODY_SIZE_ENV: &'static str = "TITRATE_BODY_SIZE";
    const FRAME_SIZE_ENV: &'static str = "TITRATE_FRAME_SIZE";

    /// parses server parameters from the process environment.
    ///
    /// # Panics
    ///
    /// this will panic if any of the environment variables are not set, or if they contain invalid
    /// data.
    pub fn new_from_env() -> Self {
        // determine which port to run on.
        let port = std::env::var(Self::PORT_ENV)
            .expect("port environment variable should exist")
            .parse()
            .expect("port environment variable should be a number");

        // determine how large the overall body should be.
        let body_size = std::env::var(Self::BODY_SIZE_ENV)
            .expect("body size environment variable should exist")
            .parse()
            .expect("body size environment variable should be a number");

        // determine how large each frame to yield is.
        let frame_size = std::env::var(Self::FRAME_SIZE_ENV)
            .expect("frame size environment variable should exist")
            .parse()
            .expect("frame size environment variable should be a number");

        assert!(frame_size <= body_size);
        assert!(body_size % frame_size == 0);

        Self {
            port,
            body_size,
            frame_size,
        }
        .tap(|params| tracing::info!(?params, "parsed parameters from environment"))
    }
}
