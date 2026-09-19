use anyhow::Context;
use std::fs::OpenOptions;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() -> anyhow::Result<()> {
    let console_filter =
        EnvFilter::try_from_env("RUST_LOG_CONSOLE").unwrap_or_else(|_| EnvFilter::new("info"));
    let file_filter =
        EnvFilter::try_from_env("RUST_LOG_FILE").unwrap_or_else(|_| EnvFilter::new("debug"));

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("app.log")
        .context("Failed to open log file")?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_filter(console_filter),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_ansi(false)
                .with_writer(log_file)
                .with_filter(file_filter),
        )
        .init();

    Ok(())
}
