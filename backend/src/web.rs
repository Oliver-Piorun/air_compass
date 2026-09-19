use anyhow::Context;
use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tracing::info;

pub async fn run() -> anyhow::Result<()> {
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .context("Failed to bind HTTP server")?;

    info!("HTTP server listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .context("Failed to run HTTP server")?;

    Ok(())
}
