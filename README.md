# AirCompass

## Project Status

> 🚧 **Work in Progress**
>
> This project is currently under active development. Features, architecture, and implementation details may change as development progresses.
>
> - [ ] ESP32 firmware - In progress
> - [ ] Infrastructure - In progress
> - [ ] Backend service - In progress
> - [ ] Flutter app - Planned

## Overview

AirCompass is a smart room-monitoring system designed to help reduce indoor humidity and improve ventilation.

An ESP32 continuously measures the room's temperature and relative humidity and publishes the measurements via MQTT. A backend service consumes these measurements and stores them in a time-series database for historical analysis and visualization.

The backend also retrieves current local weather conditions and combines them with the indoor measurements to determine whether opening a window would effectively reduce indoor humidity. When ventilation is recommended, the backend sends a push notification to the user's Flutter app, allowing the user to take action.

The system is designed to:

- Continuously monitor indoor temperature and humidity
- Store historical measurements for analysis and visualization
- Combine indoor measurements with local weather data
- Determine whether ventilation can effectively reduce indoor humidity
- Provide actionable ventilation recommendations
- Notify users when ventilation is recommended
- Provide a mobile interface for monitoring the system
