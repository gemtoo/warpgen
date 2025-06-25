mod warp;
use tracing::{Level, info, debug, span};
use axum::{
    http::{header, HeaderMap, HeaderValue}, response::IntoResponse, routing::get, Router
};
use tokio::net::TcpListener;

async fn root() -> impl IntoResponse {
    info!("Handling / request ...");
    let contents = warp::generate().await.unwrap();
    
    // Set headers to trigger file download
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    let filename = generate_safe_filename(10);
    debug!("Generated filename: {}", filename);
    let attachment_header= format!("attachment; filename=\"{}\"", filename).parse::<HeaderValue>().unwrap();
    headers.insert(
        header::CONTENT_DISPOSITION,
        attachment_header,
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

use rand::Rng;

fn generate_safe_filename(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789_-";
    
    let mut rng = rand::thread_rng();
    let basename: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    return format!("{}.conf", basename);
}