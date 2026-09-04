CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE telemetry (
    time        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    temperature REAL NOT NULL,
    humidity    REAL NOT NULL
);

SELECT create_hypertable('telemetry', by_range('time'));
