# Infrastructure

## Docker networks

The backend and infrastructure services use dedicated Docker bridge networks for communication with the Mosquitto server and TimescaleDB server.

Create the required Docker networks before starting the services:

```sh
docker network create \
  --driver bridge \
  --subnet 172.21.0.0/16 \
  --opt com.docker.network.bridge.name=br-mosquitto \
  mosquitto_network
```

```sh
docker network create \
  --driver bridge \
  --subnet 172.22.0.0/16 \
  --opt com.docker.network.bridge.name=br-timescaledb \
  timescaledb_network
```

The networks are shared between the backend and infrastructure services.
