CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE observations (
    time                        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    temperature                 REAL NOT NULL,
    relative_humidity           REAL NOT NULL,
    absolute_humidity           REAL NOT NULL,
    outdoor_temperature         REAL NOT NULL,
    outdoor_relative_humidity   REAL NOT NULL,
    outdoor_absolute_humidity   REAL NOT NULL
);

SELECT create_hypertable(
    'observations',
    by_range('time')
);