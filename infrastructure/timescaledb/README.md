# TimescaleDB

## 1. Create the certificate authority

The Certificate Authority (CA) is used to sign the TimescaleDB server certificate and the backend client certificate.

### Generate the CA private key

```sh
openssl genrsa -out ca.key 4096
```

The CA private key is the most sensitive file in this setup. Anyone who obtains it can potentially create certificates trusted by your TimescaleDB infrastructure.

### Create the CA certificate

```sh
openssl req -x509 -new -nodes \
  -key ca.key \
  -sha256 \
  -days 3650 \
  -out ca.crt
```

When prompted for the Common Name, enter:

`TimescaleDB`

This creates:

- ca.key - CA private key
- ca.crt - CA certificate

The CA certificate is valid for 10 years.

The CA certificate can be distributed to systems that need to verify certificates signed by this CA. The CA **private key must not be distributed**.

## 2. Create the TimescaleDB server certificate

The server certificate is used by TimescaleDB to provide TLS encryption and authenticate the TimescaleDB server to clients.

### Generate the server private key

```sh
openssl genrsa -out server.key 2048
```

### Create the server CSR

Generate a Certificate Signing Request:

```sh
openssl req -new \
  -key server.key \
  -out server.csr
```

For the Common Name, enter:

`your-ip-address`

### Create the server certificate extensions

The server certificate should contain the server IP address and/or hostname that clients use to connect to TimescaleDB as a Subject Alternative Name (SAN).

Create `server-ext.cnf`:

```
subjectAltName = IP:127.0.0.1, DNS:localhost, DNS:timescaledb
extendedKeyUsage = serverAuth
```

The SAN is important. Modern TLS clients generally verify the server address against the SAN rather than relying on the Common Name.

### Sign the server certificate

```sh
openssl x509 -req \
  -in server.csr \
  -CA ca.crt \
  -CAkey ca.key \
  -CAcreateserial \
  -out server.crt \
  -days 365 \
  -sha256 \
  -extfile server-ext.cnf
```

This creates:

- server.key - TimescaleDB server private key
- server.csr - TimescaleDB server certificate signing request
- server.crt - signed TimescaleDB server certificate

The server certificate is valid for 1 year.

## 3. Create the backend client certificate

The client certificate is used by the backend to authenticate itself to TimescaleDB.

### Generate the backend client private key

```sh
openssl genrsa -out backend-client.key 2048
```

### Create the backend client CSR

```sh
openssl req -new \
  -key backend-client.key \
  -out backend-client.csr
```

For the Common Name, enter:

`backend`

### Create the backend client certificate extensions

Create `client-ext.cnf`:

```
basicConstraints = CA:FALSE
keyUsage = digitalSignature
extendedKeyUsage = clientAuth
```

This specifies that the certificate cannot act as a CA, can be used for digital signatures, and is intended for TLS client authentication.

### Sign the backend client certificate

```sh
openssl x509 -req \
  -in backend-client.csr \
  -CA ca.crt \
  -CAkey ca.key \
  -CAcreateserial \
  -out backend-client.crt \
  -days 365 \
  -sha256 \
  -extfile client-ext.cnf
```

This creates:

- backend-client.key - backend client private key
- backend-client.csr - backend client certificate signing request
- backend-client.crt - signed backend client certificate

The backend client certificate is valid for 1 year.

## 4. Install the server certificates for TimescaleDB

Once the certificates have been generated, copy the files required by TimescaleDB into its certificate directory.

The TimescaleDB server needs:

- ca.crt
- server.crt
- server.key

Do no copy:

- ca.key
- backend-client.key

to the certificate directory.

The CA private key (`ca.key`) must remain protected.

### Set permissions

The TimescaleDB server private key should only be readable by root and the user/group running the TimescaleDB service:

- `sudo chown root:70 server.key`
- `sudo chmod 640 server.key`

The CA and client private keys should only be readable by the user who manages the certificates:

- `sudo chmod 600 ca.key`
- `sudo chmod 600 backend-client.key`

The certificates can be readable by all users:

- `sudo chmod 644 ca.crt`
- `sudo chmod 644 server.crt`
- `sudo chmod 644 backend-client.crt`

## 5. Files required by the backend

The backend needs the files required to establish and authenticate its TLS connection to TimescaleDB:

- ca.crt
- backend-client.crt
- backend-client.key

The `ca.crt` is used by the backend to verify the TimescaleDB server certificate.

The client uses its own client certificate and private key to authenticate itself to TimescaleDB:

- Backend → `backend-client.crt` + `backend-client.key`

Do not copy the following files to the backend:

- ca.key
- server.key

The CA private key (`ca.key`) must remain protected and must **never** be distributed to the backend or TimescaleDB server.

The client private key must remain secret and should only be stored on the respective client.

## 6. Generate the PostgreSQL password

Generate a cryptographically secure random password file with OpenSSL:

```sh
openssl rand -base64 32 | sudo tee postgres_password > /dev/null
```

Restrict access to the password file:

```sh
sudo chmod 600 postgres_password
```

## 7. Configure the PostgreSQL authentication

Configure `pg_hba.conf` as follows:

```conf
# TYPE      DATABASE    USER       ADDRESS          METHOD

# Local administrative access
local       all         postgres                    peer

# Backend: mTLS authentication only
hostssl     air_compass backend    172.22.0.0/16    cert

# Reject non-TLS connections
hostnossl   air_compass backend    172.22.0.0/16    reject
```

Replace `172.22.0.0/16` with the subnet of the Docker network used by the backend and TimescaleDB:

```sh
docker network inspect timescaledb_network \
  --format '{{range .IPAM.Config}}{{.Subnet}}{{end}}'
```

The `backend` user must match the Common Name (`CN`) of the backend client certificate.
