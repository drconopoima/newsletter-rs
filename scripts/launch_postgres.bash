#!/usr/bin/env bash
# Script for launching a container running Postgres
# Use as:
#     export POSTGRES_VERSION=9; export POSTGRES_PASSWORD=pwd; launch_postgres.bash 
IFS=$' \n\t'
# set -x
set -Eeuo pipefail
#
## Section Script Identification
readonly SCRIPT_CALLNAME="${0}"
SCRIPT_NAME="$(basename -- "${SCRIPT_CALLNAME}" 2>/dev/null)"
readonly SCRIPT_NAME
readonly SCRIPT_VERSION="0.8.1"

## Section Help
function print_help {
    printf "%s [-h|--help] (v%s)\n" "${SCRIPT_NAME}" "${SCRIPT_VERSION}"
    printf "Launch a containerized PostgreSQL database for newsletter-rs\n"
    printf "    -h|--help: Show this help message\n"
    printf "\n"
    printf "Container technology defaults to Podman (if available in PATH). Otherwise Docker (if available)\n"
    printf "\n"
    printf "Parameters are customized by exporting any of the following environment variables:\n"
    printf "    POSTGRES_USER: User for postgres. Default: 'postgres'\n"
    printf "    POSTGRES_PASSWORD: Password for POSTGRES_USER. Default: 'password'\n"
    printf "    POSTGRES_DB: User for postgres. Default: 'newsletter'\n"
    printf "    POSTGRES_HOST: Host endpoint for postgres. Default: 'localhost'\n"
    printf "    POSTGRES_PORT: Bind Port for postgres. Default: '5432'\n"
    printf "    POSTGRES_VERSION: Container version at registry. Default: 'latest'\n"
    printf "    CONTAINER_REGISTRY: Container registry to pull from. Default: 'docker.io/library/postgres'\n"
    printf "    SKIP_CONTAINER: Skip container initialization step. Default: 0 (don't skip).\n"
    printf "\n"
    printf "Example:\n"
    printf "    export POSTGRES_VERSION=9; export POSTGRES_PASSWORD=pwd; %s" "${SCRIPT_NAME}"
}
readonly -f print_help

while test $# -gt 0; do
    _key="$1"
    case "$_key" in
        -h|--help)
            print_help
            exit 0
        ;;
        *)
            echo "[ERROR] Invalid option $*"
            print_help
            exit 2
    esac
    shift
done

# Section Parent Path
NEWSLETTER_RS_PATH="$( cd -- "$(dirname "${SCRIPT_CALLNAME}")/../" >/dev/null 2>&1 ; pwd -P )"
readonly NEWSLETTER_RS_PATH
NEWSLETTER_RS_VERSION="$(grep -m1 '^version' "${NEWSLETTER_RS_PATH}/Cargo.toml" | awk -F "\"" '{ print $2 }' )"
readonly NEWSLETTER_RS_VERSION
printf "%s (v%s, newsletter-rs v%s)\n" "${SCRIPT_NAME}" "${SCRIPT_VERSION}" "${NEWSLETTER_RS_VERSION}"\

## Section validate dependencies
if ! command -v psql 1>/dev/null 2>&1; then
echo >&2 "[ERROR] psql (PostgreSQL client) is not installed."
exit 1
fi
if command -v podman 1>/dev/null 2>&1; then
function containertech {
    podman "$@"
}
elif command -v docker 1>/dev/null 2>&1; then
function containertech {
    docker "$@"
}
else
echo >&2 "[ERROR] No container library (Podman or Docker) is installed."
exit 1
fi
readonly -f containertech
export containertech

## Section Global Variables
# Check if a custom user has been set, otherwise default to 'postgres'
readonly DB_USER=${POSTGRES_USER:="postgres"}
# Check if a custom password has been set, otherwise default to 'password'
readonly POSTGRES_PASSWORD=${POSTGRES_PASSWORD:="password"}
# Check if a custom database name has been set, otherwise default to 'newsletter'
readonly DB_NAME=${POSTGRES_DB:="newsletter"}
# Check if a custom port has been set, otherwise default to '5432'
readonly DB_PORT=${POSTGRES_PORT:="5432"}
# Check if a custom container version has been set, otherwise default to 'latest'
readonly DB_VERSION=${POSTGRES_VERSION:="latest"}
# Check if a custom container registry has been set, otherwise default to 'docker.io/library/postgres'
readonly DB_REGISTRY=${CONTAINER_REGISTRY:="docker.io/library/postgres"}
# Check if a custom postgres host endpoint has been set, otherwise default to 'localhost'
readonly DB_HOST=${POSTGRES_HOST:='localhost'}
readonly CONTAINER_NAME="newsletter-rs-db"

if [[ -z ${SKIP_CONTAINER+undeclared} ]]; then
    SKIP_CONTAINER='0'
fi
if [[ -z "${SKIP_CONTAINER-}" ]]; then
    echo "Constant \$SKIP_CONTAINER should not be null" >&2 && exit 1
elif [[ ${!SKIP_CONTAINER-x} == x && ${!SKIP_CONTAINER-y} == y && "${SKIP_CONTAINER}" != 0 ]]; then
        SKIP_CONTAINER=1
fi
readonly SKIP_CONTAINER

# Allow to skip Container launch if a containerized Postgres database is already running
if [[ ${SKIP_CONTAINER} -eq 0 ]]; then
    ## Section Launch Container
    if [ ! "$(containertech ps -aq -f name="^${CONTAINER_NAME}$")" ]; then
        printf "Launching {podman/docker} postgres container at *:%s with user=%s and database=%s\n" "${DB_PORT}" "${DB_USER}" "${DB_NAME}"
        printf "When ready, clean-up by running:\n"
        printf "\t {podman/docker} stop %s\n" "${CONTAINER_NAME}"
        containertech create --rm --name "${CONTAINER_NAME}" \
        -e "POSTGRES_USER=${DB_USER}" \
        -e "POSTGRES_PASSWORD=${POSTGRES_PASSWORD}" \
        -p "${DB_PORT}":5432 \
        --health-cmd pg_isready --health-interval 10s \
        --health-timeout 5s --health-retries 5 \
        "${DB_REGISTRY}:${DB_VERSION}" \
        postgres -N 1000 1>/dev/null
        containertech start "${CONTAINER_NAME}" 1>/dev/null
        # ^ Increased maximum number of connections for testing purposes`
    else
        printf "[WARNING] There exists a container called '%s'\n" "${CONTAINER_NAME}"
        printf "\n"
        containertech ps -a -f name="${CONTAINER_NAME}"
        printf "\n"
        printf "Continuing anyways. On error, clean-up by running:\n"
        if [ "$(containertech ps -aq -f name="^${CONTAINER_NAME}$" -f status=running)" ]; then
            printf "\t {podman/docker} stop %s\n" "${CONTAINER_NAME}"
        fi
        printf "\t {podman/docker} container rm %s\n" "${CONTAINER_NAME}"
        printf "\n"
    fi
fi
# Create temporary password file
readonly PGPASSFILE='/tmp/launch_postgres_pgpass'
export PGPASSFILE
echo -n "${DB_HOST}:${DB_PORT}:*:${DB_USER}:${POSTGRES_PASSWORD}" | tee "${PGPASSFILE}" >/dev/null
chmod 600 "${PGPASSFILE}"
trap 'rm '"${PGPASSFILE}"'' EXIT
# Ping until Postgres startup is validated.
wait_time=1
until pg_isready -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}"; do
    >&2 echo "[WARN] Postgres is still unavailable - waiting ${wait_time} second(s)..."
    sleep "${wait_time}"
    wait_time=$(( wait_time * 2 ))
done
printf "[PASS] Postgres is running and ready\n"
printf "Creating Database Newsletter if not available\n"
echo "SELECT 'CREATE DATABASE ${DB_NAME}' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '${DB_NAME}')\gexec" |  psql -v ON_ERROR_STOP=1 -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -v PGPASSFILE="${PGPASSFILE}" -w
printf "[PASS] Database '%s' ready to use\n" "${DB_NAME}"

cd "${NEWSLETTER_RS_PATH}/migrations" || exit;
find . -type f -name "*.sql" -print0 | sort -z | while IFS= read -r -d '' script; do
    if command -v md5sum 1>/dev/null 2>&1; then
        md5="$(md5sum "${script}" | awk '{ print $1 }')";
        elif command -v md5 1>/dev/null 2>&1; then
        md5="$(md5 "${script}")";
    fi
    sqlfilename=$(basename "${script}");
    if [[ "$( psql -t -h $\"${DB_HOST}\" -p \"${DB_PORT}\" -U \"${DB_USER}\" -d \"${DB_NAME}\" -v PGPASSFILE="${PGPASSFILE}" -w -c "SELECT 1 FROM _initialization_migrations WHERE filename='${sqlfilename}' LIMIT 1" 2>/dev/null | awk '{ print $1 }')" -eq 1 ]]; then
        printf "[WARN] Skipping script '%s' as it's already applied\n" "${sqlfilename}";
        continue
    fi
    printf "Running migration script '%s'...\n" "${sqlfilename}"
    if grep '^COMMIT;$' "${script}" 1>/dev/null 2>&1; then
        shopt -s lastpipe
        set +Ee
        sed 's/^COMMIT;/ROLLBACK;/g' "${script}" |  psql -v ON_ERROR_STOP=1 --quiet -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -v PGPASSFILE="${PGPASSFILE}" -w 2>&1 | rollbackoutput=$(</dev/stdin)
        returncode="$?"
        shopt -u lastpipe
        set -Ee
        if [[ returncode -eq 0 ]]; then
            printf "[PASS] Tested script '%s' successfully\n" "${sqlfilename}";
        else
            printf "[FAIL] Script '%s' validation failed with return code '%s'\n" "${sqlfilename}" "${returncode}";
            echo "${rollbackoutput}"
            exit 4
        fi
        psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -v PGPASSFILE="${PGPASSFILE}" -w --file="${script}" && \
        psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -v PGPASSFILE="${PGPASSFILE}" -w -c "INSERT into _initialization_migrations ( filename, md5_hash ) VALUES ( '${sqlfilename}', '${md5}' )" && \
        printf "[PASS] Applied DB migration script '%s' successfully\n" "${sqlfilename}"
    else
        printf "[WARN]: No transactions present at script '%s', applying without prior testing\n" "${sqlfilename}"
        shopt -s lastpipe
        set +Ee
        psql -v ON_ERROR_STOP=1 -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -v PGPASSFILE="${PGPASSFILE}" -w --file="${script}"  2>&1 | rollbackoutput=$(</dev/stdin)
        returncode="$?"
        shopt -u lastpipe
        set -Ee
        if [[ returncode -eq 0 ]]; then
            printf "[PASS] Applied DB migration script '%s' successfully\n" "${sqlfilename}"
        else
            printf "[FAIL] Script '%s' failed with return code '%s'\n" "${sqlfilename}" "${returncode}";
            echo "${rollbackoutput}"
            exit 4
        fi
    fi
done

echo "[PASS] All migration scripts have been run, ready to go!"
