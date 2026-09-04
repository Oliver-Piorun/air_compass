# Mosquitto

## 1. Create the certificate authority

The Certificate Authority (CA) is used to sign both the Mosquitto server certificate, the ESP32 client certificate and the backend client certificate.

### Generate the CA private key

```sh
openssl genrsa -out ca.key 4096
```

The CA private key is the most sensitive file in this setup. Anyone who obtains it can potentially create certificates trusted by your MQTT infrastructure.

### Create the CA certificate

```sh
openssl req -x509 -new -nodes \
  -key ca.key \
  -sha256 \
  -days 3650 \
  -out ca.crt
```

When prompted for the Common Name, enter:

`MQTT`

This creates:

- ca.key - CA private key
- ca.crt - CA certificate

The CA certificate is valid for 10 years.

The CA certificate can be distributed to systems that need to verify certificates signed by this CA. The CA **private key must not be distributed**.

## 2. Create the Mosquitto server certificate

The server certificate is used by Mosquitto to provide TLS encryption and authenticate the MQTT server to clients.

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

The server certificate should contain the server IP address and/or hostname that clients use to connect to Mosquitto as a Subject Alternative Name (SAN).

Create `server-ext.cnf`:

```
subjectAltName = IP:127.0.0.1, IP:your-ip-address, DNS:localhost, DNS:mosquitto
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

- server.key - Mosquitto server private key
- server.csr - Mosquitto server certificate signing request
- server.crt - signed Mosquitto server certificate

The server certificate is valid for 1 year.

## 3. Create the ESP32 client certificate

The client certificate is used by the ESP32 to authenticate itself to Mosquitto.

### Generate the ESP32 client private key

```sh
openssl genrsa -out esp32-client.key 2048
```

### Create the ESP32 client CSR

```sh
openssl req -new \
  -key esp32-client.key \
  -out esp32-client.csr
```

For the Common Name, enter:

`ESP32`

### Create the ESP32 client certificate extensions

Create `client-ext.cnf`:

```
basicConstraints = CA:FALSE
keyUsage = digitalSignature
extendedKeyUsage = clientAuth
```

This specifies that the certificate cannot act as a CA, can be used for digital signatures, and is intended for TLS client authentication.

### Sign the ESP32 client certificate

```sh
openssl x509 -req \
  -in esp32-client.csr \
  -CA ca.crt \
  -CAkey ca.key \
  -CAcreateserial \
  -out esp32-client.crt \
  -days 365 \
  -sha256 \
  -extfile client-ext.cnf
```

This creates:

- esp32-client.key - ESP32 client private key
- esp32-client.csr - ESP32 client certificate signing request
- esp32-client.crt - signed ESP32 client certificate

The ESP32 client certificate is valid for 1 year.

## 4. Create the backend client certificate

The backend is treated as a separate MQTT client and therefore receives its own private key and client certificate.

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

### Sign the backend client certificate

The backend can reuse the existing `client-ext.cnf` because it has the same purpose as the ESP32 certificate: TLS client authentication.

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

## 5. Install the server certificates for Mosquitto

Once the certificates have been generated, copy the files required by Mosquitto into its certificate directory.

The Mosquitto server needs:

- ca.crt
- server.crt
- server.key

Do no copy:

- ca.key
- esp32-client.key
- backend-client.key

to the certificate directory.

The CA private key (`ca.key`) must remain protected.

### Set permissions

The Mosquitto server private key should only be readable by root and the user/group running the Mosquitto service:

- `sudo chown root:1883 server.key`
- `sudo chmod 640 server.key`

The CA and client private keys should only be readable by the user who manages the certificates:

- `sudo chmod 600 ca.key`
- `sudo chmod 600 esp32-client.key`
- `sudo chmod 600 backend-client.key`

The certificates can be readable by all users:

- `sudo chmod 644 ca.crt`
- `sudo chmod 644 server.crt`
- `sudo chmod 644 esp32-client.crt`
- `sudo chmod 644 backend-client.crt`

## 6. Files required by the ESP32 and backend

The ESP32 and backend each need the files required to establish and authenticate their TLS connection to Mosquitto:

### ESP32

- ca.crt
- esp32-client.crt
- esp32-client.key

### Backend

- ca.crt
- backend-client.crt
- backend-client.key

The `ca.crt` is used by both the ESP32 and backend to verify the Mosquitto server certificate.

Each client uses its own client certificate and private key to authenticate itself to Mosquitto:

- ESP32 → `esp32-client.crt` + `esp32-client.key`
- Backend → `backend-client.crt` + `backend-client.key`

Do not copy the following files to either the ESP32 or backend:

- ca.key
- server.key
- The other client's private key

The CA private key (`ca.key`) must remain protected and must **never** be distributed to the ESP32, backend, or Mosquitto server.

The client private keys must remain secret and should only be stored on their respective clients.
