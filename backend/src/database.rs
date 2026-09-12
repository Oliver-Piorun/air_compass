use anyhow::Context;
use sqlx::{
    Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::env;

const POSTGRES_CA_CERT: &str = "/run/secrets/postgres_ca_cert";
const POSTGRES_CLIENT_CERT: &str = "/run/secrets/postgres_client_cert";
const POSTGRES_CLIENT_KEY: &str = "/run/secrets/postgres_client_key";

pub async fn init_postgres_pool() -> anyhow::Result<Pool<Postgres>> {
    let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "timescaledb".to_string());
    let port = env::var("POSTGRES_PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(5432);

    let ca_cert_path =
        env::var("POSTGRES_CA_CERT").unwrap_or_else(|_| POSTGRES_CA_CERT.to_string());
    let client_cert_path =
        env::var("POSTGRES_CLIENT_CERT").unwrap_or_else(|_| POSTGRES_CLIENT_CERT.to_string());
    let client_key_path =
        env::var("POSTGRES_CLIENT_KEY").unwrap_or_else(|_| POSTGRES_CLIENT_KEY.to_string());

    let options = PgConnectOptions::new()
        .host(&host)
        .port(port)
        .database("air_compass")
        .username("backend")
        .ssl_mode(sqlx::postgres::PgSslMode::VerifyFull)
        .ssl_root_cert(ca_cert_path)
        .ssl_client_cert(client_cert_path)
        .ssl_client_key(client_key_path);

    PgPoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await
        .context("Failed to connect to PostgreSQL")
}
