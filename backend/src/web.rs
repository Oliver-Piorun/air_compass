use anyhow::Context;
use axum::{Router, routing::get};
use tokio::net::TcpListener;

pub async fn run() -> anyhow::Result<()> {
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .context("Failed to bind HTTP server")?;

    println!("Listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .context("HTTP server failed")?;

    Ok(())
}
