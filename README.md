# AirCompass

## Project Status

> 🚧 **Work in Progress**
>
> This project is currently under active development. Features, architecture, and implementation details may change as development progresses.
>
> - [ ] ESP32 firmware - In progress
> - [ ] Backend service - Next up
> - [ ] Flutter app - Planned

## Motivation

The goal of this project is to build a smart room-monitoring system that helps reduce indoor humidity and improve ventilation.

An ESP32 measures the room's temperature and humidity and publishes the measurements via MQTT. A backend service consumes these measurements and stores them in a time-series database for later analysis.

The backend also retrieves local weather data and combines it with the indoor measurements to determine whether opening a window would be an effective way to reduce the room's humidity. If ventilation is recommended, the backend sends a push notification to the user's Flutter app.

## How It Works

1. The ESP32 measures temperature and relative humidity.
2. Measurements are published to an MQTT broker.
3. The backend service consumes the MQTT messages.
4. Measurements are stored in a time-series database.
5. The backend retrieves the current local weather conditions.
6. Indoor and outdoor conditions are analyzed to determine whether ventilation could reduce indoor humidity.
7. If ventilation is recommended, the backend sends a push notification.
8. The Flutter app receives and displays the notification.

## Goals

1. Monitor indoor temperature and humidity continuously
2. Store historical measurements for analysis and visualization
3. Combine indoor measurements with local weather data
4. Provide actionable ventilation recommendations
5. Notify users when ventilation is likely to reduce humidity
6. Provide a mobile interface for monitoring the system
