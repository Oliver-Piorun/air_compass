# TimescaleDB

## Generate the PostgreSQL password

Generate a cryptographically secure random password file with OpenSSL:

```sh
openssl rand -base64 32 | sudo tee postgres_password > /dev/null
```

Restrict access to the password file:

```sh
sudo chmod 600 postgres_password
```
