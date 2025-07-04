mod warp;
mod web;
mod misc;
use tracing::{Level, span};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let filter = EnvFilter::new("warpgen=debug");
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let warpgen_span = span!(Level::INFO, "warpgen");
    let _enter = warpgen_span.enter();
    web::serve().await
}
