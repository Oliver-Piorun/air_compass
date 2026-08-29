use std::{
    fs::{self},
    time::Duration,
};

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
use tokio::{net::TcpListener, task};

const CA_CERT: &str = "/run/secrets/mqtt-ca-cert";
const CLIENT_CERT: &str = "/run/secrets/mqtt-client-cert";
const CLIENT_KEY: &str = "/run/secrets/mqtt-client-key";

#[tokio::main]
async fn main() {
    let (ca_cert, client_cert, client_key) = load_certs().unwrap();

    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add(ca_cert).unwrap();

    let tls_client_config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_client_auth_cert(vec![client_cert], client_key)
        .unwrap();

    let mut mqtt_options = MqttOptions::new("air_compass_backend", "mosquitto", 8883);
    mqtt_options.set_transport(Transport::tls_with_config(tls_client_config.into()));
    mqtt_options.set_keep_alive(Duration::from_secs(5));

    let (async_client, mut event_loop) = AsyncClient::new(mqtt_options, 10);

    async_client
        .subscribe("telemetry", QoS::AtMostOnce)
        .await
        .unwrap();

    task::spawn(async move {
        loop {
            match event_loop.poll().await {
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    println!("a");
                }
                Ok(event) => {
                    println!("b");
                }
                Err(error) => {
                    eprintln!("c");
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

fn load_certs() -> Result<
    (
        CertificateDer<'static>,
        CertificateDer<'static>,
        PrivateKeyDer<'static>,
    ),
    Box<dyn std::error::Error>,
> {
    let ca_pem = fs::read(CA_CERT)?;
    let client_cert_pem = fs::read(CLIENT_CERT)?;
    let client_key_pem = fs::read(CLIENT_KEY)?;

    let ca_cert = CertificateDer::from_pem_slice(&ca_pem)?;
    let client_cert = CertificateDer::from_pem_slice(&client_cert_pem)?;
    let client_key = PrivateKeyDer::from_pem_slice(&client_key_pem)?;

    Ok((ca_cert, client_cert, client_key))
}
