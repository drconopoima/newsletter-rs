# newsletter-rs
Email subscriptions newsletter build by using Rust and actix-web.

## Pre-requisites

You'll need to install:

- [Rust](https://www.rust-lang.org/tools/install)
- [Podman](https://podman.io/getting-started/installation) or [Docker](https://docs.docker.com/get-docker/)

On Ubuntu/Debian

```bash
apt install podman
```

On CentOs >= 8

```bash
dnf install podman
```

On Max OS X

```sh
brew install podman
```

- [PostgreSQL CLI client](https://www.postgresql.org/download/)

On Ubuntu/Debian

```bash
apt install postgresql-client
```

On CentOs >= 8

```bash
dnf install postgresql
```

On Mac OS X using Homebrew

```sh
brew install libpq
echo 'export PATH="/usr/local/opt/libpq/bin:$PATH"' >> ~/.zshrc
```

Launch a (migrated) Postgres database via a container engine (default Podman, optionally Docker):

```bash
./scripts/launch_postgres.bash
```

## Launch

Using `cargo`:

```bash
cargo run
```

Test correct initialization by using /healthcheck endpoint

```bash
curl -s -w'%{http_code}' http://127.0.0.1:8000/healthcheck
```

Send subscription entries by using the /subscription endpoint

```bash
curl -s -w'%{http_code}' "http://127.0.0.1:8000/subscription" -d "email=email%40drconopoima.com&name=Jane%20Doe"
```

## How to build

Using `cargo`:

```bash
cargo build
```

## How to test

Using `cargo`:

```bash
cargo test 
```
