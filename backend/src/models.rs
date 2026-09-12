#[derive(Debug, serde::Deserialize)]
pub struct Telemetry {
    pub temperature: f32,
    pub relative_humidity: f32,
}

#[derive(Debug, serde::Deserialize)]
pub struct OutdoorWeather {
    pub temperature: f32,
    pub relative_humidity: f32,
}
