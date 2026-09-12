use crate::models::{OutdoorWeather, Telemetry};

pub fn absolute_humidity(temperature: f32, relative_humidity: f32) -> f32 {
    let saturation_vapor_pressure = 6.112 * ((17.67 * temperature) / (temperature + 243.5)).exp();
    let vapor_pressure = relative_humidity / 100.0 * saturation_vapor_pressure;

    216.7 * vapor_pressure / (273.15 + temperature)
}

#[derive(Debug, serde::Deserialize)]
pub struct Observation {
    pub temperature: f32,
    pub relative_humidity: f32,
    pub absolute_humidity: f32,
    pub outdoor_temperature: f32,
    pub outdoor_relative_humidity: f32,
    pub outdoor_absolute_humidity: f32,
}

impl Observation {
    pub fn from_sources(telemetry: Telemetry, outdoor_weather: OutdoorWeather) -> Self {
        Self {
            temperature: telemetry.temperature,
            relative_humidity: telemetry.relative_humidity,
            absolute_humidity: absolute_humidity(
                telemetry.temperature,
                telemetry.relative_humidity,
            ),
            outdoor_temperature: outdoor_weather.temperature,
            outdoor_relative_humidity: outdoor_weather.relative_humidity,
            outdoor_absolute_humidity: absolute_humidity(
                outdoor_weather.temperature,
                outdoor_weather.relative_humidity,
            ),
        }
    }
}
