mod warp;
use tracing::{Level, info, span};
use axum::{
    response::IntoResponse,
    routing::get,
    Router,
    http::{header, HeaderMap},
};
use tokio::net::TcpListener;

async fn root() -> impl IntoResponse {
    info!("Handling / request ...");
    let contents = warp::generate().await;
    
    // Set headers to trigger file download
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"warpgen.conf\"".parse().unwrap(),
    );
    
    (headers, contents)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    let warpgen_span = span!(Level::INFO, "warpgen");
    let _enter = warpgen_span.enter();
    
    // Configure routes
    let app = Router::new()
        .route("/", get(root));

    // Start server
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await?;
    info!("Server listening on {}", addr);
    axum::serve(listener, app)
        .await?;

    Ok(())
}