pub mod body;
pub mod client;
pub mod rt;
pub mod server;

/// a boxed error.
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
