# newsletter-rs
Email subscriptions newsletter build by using Rust and actix-web.

## Pre-requisites

You'll need to install:

- [Rust](https://www.rust-lang.org/tools/install)
- [Podman](https://podman.io/getting-started/installation) or [Docker](https://docs.docker.com/get-docker/)
- [PostgreSQL CLI client](https://www.postgresql.org/download/)
- Optionally for loadtesting: [k6](https://k6.io/docs/getting-started/installation/)

### Install Podman

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

### Install PostgreSQL client

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

## Launch

Launch a (migrated) Postgres database via a container engine (default Podman, optionally Docker):

```bash
./scripts/launch_postgres.bash
```

Run application using `cargo`:

```bash
cargo run
```

Send subscription entries by using the `/subscription` endpoint

```bash
curl -s -w'%{http_code}' "http://127.0.0.1:8000/subscription" -d "email=email%40drconopoima.com&name=Jane%20Doe"
```

Test correct operation by using `/healthcheck` endpoint

```bash
curl -s -w'\n%{http_code}' http://127.0.0.1:65080/healthcheck | jq '.'
```

```text
{
  "status": "pass",
  "checks": {
    "postgres_read": {
      "status": "pass",
      "time": "2022-03-06T18:15:12.14105Z",
      "output": ""
    },
    "postgres_write": {
      "status": "pass",
      "time": "2022-03-06T18:15:12.14125Z",
      "pg_is_in_recovery": false,
      "output": ""
    }
  },
  "output": "",
  "time": "2022-03-06T18:15:12.140507104Z",
  "version": "0.1.0"
}
200
```

### Customize logging level

By default, newsletter-rs is configured with Actix Logger Middleware in INFO logging level. It can be customized with RUST_LOG environment variable at runtime.

```sh
    export RUST_LOG=DEBUG # Valid options trace|debug|info|warn|error|fatal
    cargo run
```

### Enable backtrace

```sh
    export RUST_BACKTRACE=1 # Valid options 1|full
    cargo run
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

## How to loadtest

Using K6

```bash
cargo run --release
k6 run --vus 200 ./testdata/k6_get_healthcheck.js --duration 60s
k6 run --vus 200 ./testdata/k6_post_subscription.js --duration 60s
```

## Database details

Check the [database diagram](database_diagram.md) section.
