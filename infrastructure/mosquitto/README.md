# Mosquitto

## 1. Create the Certificate Authority

The Certificate Authority (CA) is used to sign both the Mosquitto server certificate and the ESP32 client certificate.

### Generate the CA private key

```sh
openssl genrsa -out ca.key 4096
chmod 600 ca.key
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

The CA certificate can be distributed to systems that need to verify certificates signed by this CA. The CA **private key must not be distributed**.

## 2. Create the Mosquitto Server Certificate

The server certificate is used by Mosquitto to provide TLS encryption and authenticate the MQTT server to clients.

### Generate the server private key

```sh
openssl genrsa -out server.key 2048
chmod 600 server.key
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

The server certificate should contain the server IP address as a Subject Alternative Name (SAN).

Create `server-ext.cnf`:

```
subjectAltName = IP:your-ip-address
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
- server.csr - server Certificate Signing Request
- server.crt - signed server certificate

The server certificate is valid for one year.

## 3. Create the ESP32 Client Certificate

The client certificate is used by the ESP32 to authenticate itself to Mosquitto.

### Generate the client private key

```sh
openssl genrsa -out client.key 2048
chmod 600 client.key
Create the client CSR
openssl req -new \
  -key client.key \
  -out client.csr
```

For the Common Name, enter:

`ESP32`

### Create the client certificate extensions

Create `client-ext.cnf`:

```
basicConstraints = CA:FALSE
keyUsage = digitalSignature
extendedKeyUsage = clientAuth
```

This identifies the certificate as being intended for client authentication.

### Sign the client certificate

```sh
openssl x509 -req \
  -in client.csr \
  -CA ca.crt \
  -CAkey ca.key \
  -CAcreateserial \
  -out client.crt \
  -days 365 \
  -sha256 \
  -extfile client-ext.cnf
```

This creates:

- client.key - ESP32 private key
- client.csr - client Certificate Signing Request
- client.crt - signed client certificate

The client certificate is valid for one year.

## 4. Install the Server Certificates for Mosquitto

Once the certificates have been generated, copy the files required by Mosquitto into its certificate directory.

The Mosquitto server needs:

- ca.crt
- server.crt
- server.key

Do no copy:

- ca.key
- client.key

to the certificate directory.

The CA private key (`ca.key`) must remain protected.

### Set permissions

The Mosquitto server private key should only be readable by root and the user/group running the Mosquitto service:

- `sudo chown root:1883 server.key`
- `sudo chmod 640 server.key`

The CA and client private keys should only be readable by the user who manages the certificates:

- `sudo chmod 600 ca.key`
- `sudo chmod 600 client.key`

The certificates can be readable by all users:

- `sudo chmod 644 ca.crt`
- `sudo chmod 644 server.crt`
- `sudo chmod 644 client.crt`

## 5. Files Required by the ESP32

The ESP32 needs only the files required to establish and authenticate the TLS connection:

- ca.crt
- client.crt
- client.key

Do not copy:

- ca.key
- server.key

to the ESP32.

The CA private key (`ca.key`) must remain protected.
