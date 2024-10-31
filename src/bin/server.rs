//! titrate server binary.

use titrate::server::{Params, Server};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let params = Params::new_from_env();
    Server::run(params).await.unwrap();

    unreachable!();
}
