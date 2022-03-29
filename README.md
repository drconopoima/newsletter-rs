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
curl -s -w'\n%{http_code}\n' "http://127.0.0.1:8000/subscription" -d "email=email%40drconopoima.com&name=Jane%20Doe"
```

Test correct operation by using `/healthcheck` endpoint

```bash
curl -s -w'\n%{http_code}\n' http://127.0.0.1:65080/healthcheck | jq '.'
```

```text
{
  "status": "pass",
  "checks": {
    "postgres_read": {
      "status": "pass",
      "time": "2022-03-06T23:32:10.555806Z",
      "output": "",
      "version": "PostgreSQL 14.2 (Debian 14.2-1.pgdg110+1) on x86_64-pc-linux-gnu, compiled by gcc (Debian 10.2.1-6) 10.2.1 20210110, 64-bit"
    },
    "postgres_write": {
      "status": "pass",
      "time": "2022-03-06T23:32:10.556877Z",
      "pg_is_in_recovery": false,
      "output": "",
      "version": "PostgreSQL 14.2 (Debian 14.2-1.pgdg110+1) on x86_64-pc-linux-gnu, compiled by gcc (Debian 10.2.1-6) 10.2.1 20210110, 64-bit"
    }
  },
  "output": "",
  "time": "2022-03-06T23:32:10.547917389Z",
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

Logging output from test logs is suppressed. Set environment variable `TEST_LOG` for output:

```sh
TEST_LOG=true cargo test
```

To beautify the tracing output, you can use a process substitution and a JSON processing library, like `jq`:

```sh
while read -r line; do echo "${line}" | jq -R 'fromjson? | .'; done< <(TEST_LOG=true cargo test)
```

## How to loadtest

Using K6

```bash
cargo run --release
k6 run --vus 200 ./testdata/k6_get_healthcheck.js --duration 60s
k6 run --vus 200 ./testdata/k6_post_subscription.js --duration 60s
```

## Configuration

Each configuration parameter is obtained from files within relative directory 'configuration'. The base configuration file is main.yaml

You can generate custom override configuration files, and launch by using variable 'APP__ENVIRONMENT'

E.g. to launch with example production settings, you would use:

```sh
APP__ENVIRONMENT=production cargo run --release
```

You can override any parameter with environment variables, by using the prefix "APP__" and field separator "_":

```sh
  export APP__DATABASE_PASSWORD='Some$ecretPassword'
  APP__APPLICATION_PORT=8080 cargo run --release
```

Would be equivalent as fabricating a custom override configuration file:

```yaml
application:
  port: 8080
database:
  password: 'Some$ecretPassword'
```

## Database details

Check the [database diagram](database_diagram.md) section.
