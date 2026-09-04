use axum::{Router, routing::get};
use rumqttc::{
    Transport,
    tokio_rustls::rustls::{
        ClientConfig, RootCertStore,
        pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    },
    v5::{
        AsyncClient, Event, MqttOptions,
        mqttbytes::{QoS, v5::Packet},
    },
};
use sqlx::{
    Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::{
    env,
    fs::{self},
    time::Duration,
};
use tokio::{net::TcpListener, task};

const MOSQUITTO_CA_CERT: &str = "/run/secrets/mosquitto_ca_cert";
const MOSQUITTO_CLIENT_CERT: &str = "/run/secrets/mosquitto_client_cert";
const MOSQUITTO_CLIENT_KEY: &str = "/run/secrets/mosquitto_client_key";

const POSTGRES_CA_CERT: &str = "/run/secrets/postgres_ca_cert";
const POSTGRES_CLIENT_CERT: &str = "/run/secrets/postgres_client_cert";
const POSTGRES_CLIENT_KEY: &str = "/run/secrets/postgres_client_key";

#[derive(Debug, serde::Deserialize)]
struct Telemetry {
    temperature: f32,
    humidity: f32,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let postgres_pool = init_postgres_pool().await;
    sqlx::migrate!().run(&postgres_pool).await.unwrap();

    let (mqtt_async_client, mut mqtt_event_loop) = init_mqtt_client().await;

    mqtt_async_client
        .subscribe("telemetry", QoS::AtMostOnce)
        .await
        .unwrap();

    task::spawn(async move {
        loop {
            match mqtt_event_loop.poll().await {
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    println!("Incoming publish! {publish:?}");
                    let telemetry = serde_json::from_slice::<Telemetry>(&publish.payload).unwrap();

                    sqlx::query!(
                        r#"
                        INSERT INTO telemetry (temperature, humidity)
                        VALUES ($1, $2)
                    "#,
                        telemetry.temperature,
                        telemetry.humidity
                    )
                    .execute(&postgres_pool)
                    .await
                    .unwrap();
                }
                Ok(event) => {
                    println!("Event! {event:?}");
                }
                Err(error) => {
                    eprintln!("Error! {error:?}");
                    break;
                }
            }
        }
    });

    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn init_postgres_pool() -> Pool<Postgres> {
    let ca_cert_path =
        env::var("POSTGRES_CA_CERT").unwrap_or_else(|_| POSTGRES_CA_CERT.to_string());
    let client_cert_path =
        env::var("POSTGRES_CLIENT_CERT").unwrap_or_else(|_| POSTGRES_CLIENT_CERT.to_string());
    let client_key_path =
        env::var("POSTGRES_CLIENT_KEY").unwrap_or_else(|_| POSTGRES_CLIENT_KEY.to_string());

    let pg_connect_options = PgConnectOptions::new()
        .host("127.0.0.1")
        .port(5432)
        .database("air_compass")
        .username("backend")
        .ssl_mode(sqlx::postgres::PgSslMode::VerifyFull)
        .ssl_root_cert(ca_cert_path)
        .ssl_client_cert(client_cert_path)
        .ssl_client_key(client_key_path);

    PgPoolOptions::new()
        .max_connections(10)
        .connect_with(pg_connect_options)
        .await
        .unwrap()
}

async fn init_mqtt_client() -> (rumqttc::v5::AsyncClient, rumqttc::v5::EventLoop) {
    let ca_cert_path =
        env::var("MOSQUITTO_CA_CERT").unwrap_or_else(|_| MOSQUITTO_CA_CERT.to_string());
    let client_cert_path =
        env::var("MOSQUITTO_CLIENT_CERT").unwrap_or_else(|_| MOSQUITTO_CLIENT_CERT.to_string());
    let client_key_path =
        env::var("MOSQUITTO_CLIENT_KEY").unwrap_or_else(|_| MOSQUITTO_CLIENT_KEY.to_string());

    let ca_pem = fs::read(ca_cert_path).unwrap();
    let client_cert_pem = fs::read(client_cert_path).unwrap();
    let client_key_pem = fs::read(client_key_path).unwrap();

    let ca_cert = CertificateDer::from_pem_slice(&ca_pem).unwrap();
    let client_cert = CertificateDer::from_pem_slice(&client_cert_pem).unwrap();
    let client_key = PrivateKeyDer::from_pem_slice(&client_key_pem).unwrap();

    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add(ca_cert).unwrap();

    let tls_client_config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_client_auth_cert(vec![client_cert], client_key)
        .unwrap();

    let host = env::var("MOSQUITTO_HOST").unwrap_or_else(|_| "mosquitto".to_string());
    let port = env::var("MOSQUITTO_PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(8883);

    let mut mqtt_options = MqttOptions::new("backend", host, port);
    mqtt_options.set_transport(Transport::tls_with_config(tls_client_config.into()));
    mqtt_options.set_keep_alive(Duration::from_secs(5));

    AsyncClient::new(mqtt_options, 10)
}
