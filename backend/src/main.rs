use air_compass::models::OutdoorWeather;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    air_compass::run(|| async {
        Ok(OutdoorWeather {
            temperature: 0.0,
            relative_humidity: 0.0,
        })
    })
    .await
}
