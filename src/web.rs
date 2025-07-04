use crate::warp::WarpConfig;
use crate::misc::generate_safe_filename;
use tokio::net::TcpListener;
use axum::{
    Router,
    http::{HeaderMap, HeaderValue, header},
    response::IntoResponse,
    routing::get,
};
use tracing::{debug, info, instrument};

#[instrument(level = "info", skip_all)]
async fn web() -> impl IntoResponse {
    info!("Handling / request ...");
    let contents = WarpConfig::generate().await.unwrap();

    // Set headers to trigger file download
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    let filename = generate_safe_filename(10);
    debug!("Generated filename: {}", filename);
    let attachment_header = format!("attachment; filename=\"{}\"", filename)
        .parse::<HeaderValue>()
        .unwrap();
    headers.insert(header::CONTENT_DISPOSITION, attachment_header);

    (headers, contents)
}

#[instrument(level = "info", skip_all)]
pub async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    // Configure routes
    let app = Router::new().route("/", get(web));

    // Start server
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await?;
    info!("Server listening on {} ...", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
