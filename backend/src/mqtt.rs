use anyhow::Context;
use rumqttc::{
    Transport,
    v5::{
        AsyncClient, Event, MqttOptions,
        mqttbytes::{QoS, v5::Packet},
    },
};
use rustls::{
    ClientConfig, RootCertStore,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use sqlx::{Pool, Postgres};
use std::{env, fs, time::Duration};

use crate::{models::OutdoorWeather, observation::Observation, observation_store};

const MOSQUITTO_CA_CERT: &str = "/run/secrets/mosquitto_ca_cert";
const MOSQUITTO_CLIENT_CERT: &str = "/run/secrets/mosquitto_client_cert";
const MOSQUITTO_CLIENT_KEY: &str = "/run/secrets/mosquitto_client_key";

pub async fn run<F, Fut>(pool: Pool<Postgres>, get_outdoor_weather: F) -> anyhow::Result<()>
where
    F: Fn() -> Fut,
    Fut: Future<Output = anyhow::Result<OutdoorWeather>>,
{
    let (async_client, mut event_loop) = init_mqtt_client().await;

    async_client
        .subscribe("telemetry", QoS::AtMostOnce)
        .await
        .context("Failed to subscribe to telemetry topic")?;

    loop {
        match event_loop.poll().await {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                println!("Incoming publish! {publish:?}");

                let telemetry = match serde_json::from_slice(&publish.payload) {
                    Ok(telemetry) => telemetry,
                    Err(e) => {
                        eprintln!("Failed to deserialize telemetry JSON: {e}");
                        continue;
                    }
                };

                let outdoor_weather = match get_outdoor_weather().await {
                    Ok(outdoor_weather) => outdoor_weather,
                    Err(e) => {
                        eprintln!("Failed to get outdoor weather: {e}");
                        continue;
                    }
                };

                let observation = Observation::from_sources(telemetry, outdoor_weather);

                observation_store::store(&pool, &observation).await?;
            }

            Ok(event) => {
                println!("MQTT event: {event:?}");
            }

            Err(error) => {
                return Err(error).context("MQTT event loop failed");
            }
        }
    }
}

async fn init_mqtt_client() -> (rumqttc::v5::AsyncClient, rumqttc::v5::EventLoop) {
    let host = env::var("MOSQUITTO_HOST").unwrap_or_else(|_| "mosquitto".to_string());
    let port = env::var("MOSQUITTO_PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(8883);

    let ca_cert_path =
        env::var("MOSQUITTO_CA_CERT").unwrap_or_else(|_| MOSQUITTO_CA_CERT.to_string());
    let client_cert_path =
        env::var("MOSQUITTO_CLIENT_CERT").unwrap_or_else(|_| MOSQUITTO_CLIENT_CERT.to_string());
    let client_key_path =
        env::var("MOSQUITTO_CLIENT_KEY").unwrap_or_else(|_| MOSQUITTO_CLIENT_KEY.to_string());

    let ca_cert_pem = fs::read(ca_cert_path).unwrap();
    let client_cert_pem = fs::read(client_cert_path).unwrap();
    let client_key_pem = fs::read(client_key_path).unwrap();

    let ca_cert_der = CertificateDer::from_pem_slice(&ca_cert_pem).unwrap();
    let client_cert_der = CertificateDer::from_pem_slice(&client_cert_pem).unwrap();
    let client_key_der = PrivateKeyDer::from_pem_slice(&client_key_pem).unwrap();

    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add(ca_cert_der).unwrap();

    let tls_client_config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_client_auth_cert(vec![client_cert_der], client_key_der)
        .unwrap();

    let mut mqtt_options = MqttOptions::new("backend", host, port);
    mqtt_options.set_transport(Transport::tls_with_config(tls_client_config.into()));
    mqtt_options.set_keep_alive(Duration::from_secs(5));

    AsyncClient::new(mqtt_options, 10)
}
