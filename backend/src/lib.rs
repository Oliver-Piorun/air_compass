use crate::models::OutdoorWeather;

pub mod database;
pub mod logging;
pub mod models;
pub mod mqtt;
pub mod observation;
pub mod observation_store;
pub mod web;

pub async fn run<F, Fut>(get_outdoor_weather: F) -> anyhow::Result<()>
where
    F: Fn() -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<OutdoorWeather>> + Send + 'static,
{
    dotenvy::dotenv().ok();

    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("Failed to install ring TLS provider"))?;

    logging::init().unwrap();

    let postgres_pool = database::init_postgres_pool().await.unwrap();
    sqlx::migrate!().run(&postgres_pool).await.unwrap();

    tokio::spawn(mqtt::run(postgres_pool.clone(), get_outdoor_weather));

    web::run().await
}
