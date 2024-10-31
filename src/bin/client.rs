//! titrate client binary.

use titrate::client::{Client, Params};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let params = Params::new_from_env();
    Client::run(params).await
}
