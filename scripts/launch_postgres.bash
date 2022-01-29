#!/usr/bin/env bash
# set -x
set -Eeuo pipefail
#
## Section Script Identification
readonly SCRIPT_CALLNAME="${0}"
SCRIPT_NAME="$(basename -- "${SCRIPT_CALLNAME}" 2>/dev/null)"
readonly SCRIPT_NAME
readonly SCRIPT_VERSION="0.1.1"
NEWSLETTER_RS_PATH="$( cd -- "$(dirname "${SCRIPT_CALLNAME}")/../" >/dev/null 2>&1 ; pwd -P )"
readonly NEWSLETTER_RS_PATH
NEWSLETTER_RS_VERSION="$(grep -m1 '^version' "${NEWSLETTER_RS_PATH}/Cargo.toml" | awk -F "\"" '{ print $2 }' )"
readonly NEWSLETTER_RS_VERSION

## Section Functions
function help(){
printf "%s [-hq] (v%s)\n" "${SCRIPT_NAME}" "${SCRIPT_VERSION}"
printf "Launch a containerized PostgreSQL database for newsletter-rs\n"
printf "\n"
printf "\t -h: Show this help message\n"
printf "Container Engine: Defaults to Podman (if available in PATH). Otherwise Docker (if available in PATH)\n"
printf "\n"
printf "Parameters are customized by exporting any of the following environment variables:\n"
printf "\t POSTGRES_USER: User for postgres. Default: 'postgres'\n"
printf "\t POSTGRES_PASSWORD: Password for POSTGRES_USER. Default: 'password'\n"
printf "\t POSTGRES_DB: User for postgres. Default: 'newsletter'\n"
printf "\t POSTGRES_PORT: Bind Port for postgres. Default: '5432'\n"
printf "\t POSTGRES_VERSION: Container version at registry. Default: 'latest'\n"
printf "\t CONTAINER_REGISTRY: Container registry to pull from. Default: 'docker.io/library/postgres'\n"
}
readonly -f help

while getopts ':h' option; do
case "$option" in
    h)  # display help
        help
        exit 0
    ;;
    * )
        echo "Error: Invalid option $*"
        help
        exit 2
esac
done

printf "%s (v%s, newsletter-rs v%s)\n" "${SCRIPT_NAME}" "${SCRIPT_VERSION}" "${NEWSLETTER_RS_VERSION}"

if ! command -v psql 1>/dev/null 2>&1; then
  echo >&2 "ERROR: psql (PostgreSQL client) is not installed."
  exit 1
fi

## Section Global Variables
# Check if a custom user has been set, otherwise default to 'postgres'
readonly DB_USER=${POSTGRES_USER:="postgres"}
# Check if a custom password has been set, otherwise default to 'password'
readonly DB_PASSWORD=${POSTGRES_PASSWORD:="password"}
# Check if a custom database name has been set, otherwise default to 'newsletter'
readonly DB_NAME=${POSTGRES_DB:="newsletter"}
# Check if a custom port has been set, otherwise default to '5432'
readonly DB_PORT=${POSTGRES_PORT:="5432"}
# Check if a custom container version has been set, otherwise default to 'latest'
readonly DB_VERSION=${POSTGRES_VERSION:="latest"}
# Check if a custom container registry has been set, otherwise default to 'docker.io/library/postgres'
readonly DB_REGISTRY=${CONTAINER_REGISTRY:="docker.io/library/postgres"}
readonly CONTAINER_NAME="newsletter-rs-db"

## Section Launch Container
if command -v podman 1>/dev/null 2>&1; then
    if [ ! "$(podman ps -aq -f name="^${CONTAINER_NAME}$")" ]; then
        printf "Launching podman postgres container at *:%s with user=%s and database=%s\n" "${DB_PORT}" "${DB_USER}" "${DB_NAME}"
        printf "When ready, clean-up by running:\n"
        printf "\t podman stop %s\n" "${CONTAINER_NAME}"
        podman run -d --rm --name ${CONTAINER_NAME} \
            -e POSTGRES_USER=${DB_USER} \
            -e POSTGRES_PASSWORD=${DB_PASSWORD} \
            -e POSTGRES_DB=${DB_NAME} \
            -p "${DB_PORT}":5432 \
            ${DB_REGISTRY}:${DB_VERSION} \
            postgres -N 1000 1>/dev/null
            # ^ Increased maximum number of connections for testing purposes
    else
        printf "ERROR: There exists a container called '%s'\n" "${CONTAINER_NAME}"
        printf "\n"
        podman ps -a -f name=${CONTAINER_NAME}
        printf "\n"
        printf "Please clean-up by running:\n"
        if [ "$(podman ps -aq -f name="^${CONTAINER_NAME}$" -f status=running)" ]; then
            printf "\t podman stop %s\n" "${CONTAINER_NAME}"
        fi
        printf "\t podman container rm %s\n" "${CONTAINER_NAME}"
        
    fi
elif command -v docker 1>/dev/null 2>&1; then
    if [ ! "$(docker ps -aq -f name="^${CONTAINER_NAME}$")" ]; then
        printf "Launching docker postgres container at *:%s with user=%s and database=%s\n" "${DB_PORT}" "${DB_USER}" "${DB_NAME}"
        printf "When ready, clean-up by running:\n"
        printf "\t docker stop %s\n" "${CONTAINER_NAME}"
        docker run -d --rm --name ${CONTAINER_NAME} \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        ${DB_REGISTRY}:${DB_VERSION} \
        postgres -N 1000 1>/dev/null
        # ^ Increased maximum number of connections for testing purposes
    else
        printf "ERROR: There exists a container called '%s'\n" "${CONTAINER_NAME}"
        printf "\n"
        docker ps -a -f name=${CONTAINER_NAME}
        printf "\n"
        printf "Please clean-up by running:\n"
        if [ "$(docker ps -aq -f name="^${CONTAINER_NAME}$" -f status=running)" ]; then
            printf "\t docker stop %s\n" "${CONTAINER_NAME}"
        fi
        printf "\t docker container rm %s\n" "${CONTAINER_NAME}"
    fi
fi

# Ping until Postgres startup is validated.
wait_time=1
until psql postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME} -c '\q' 2>/dev/null; do
  >&2 echo "[WARN] Postgres is still unavailable - waiting ${wait_time} second(s)..."
  sleep "${wait_time}"
  wait_time=$(( wait_time * 2 ))
done

printf "[PASS] Postgres is running and ready\n"

cd "${NEWSLETTER_RS_PATH}/migrations" || exit;
find . -type f -name "*.sql" -print0 | sort -z | while IFS= read -r -d '' script; do
    if command -v md5sum 1>/dev/null 2>&1; then
        md5="$(md5sum "${script}" | awk '{ print $1 }')";
    elif command -v md5 1>/dev/null 2>&1; then
        md5="$(md5 "${script}")";
    fi
    sqlfilename=$(basename ${script});
    set +Ee;
    if [[ "$(psql -t -v ON_ERROR_STOP=1 "postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}" -c "SELECT 1 FROM _initialization_migrations WHERE filename='${sqlfilename}' LIMIT 1" | awk '{ print $1 }')" -eq 1 ]]; then
        printf "[WARN] Skipping script '%s' as it's already applied\n" "${sqlfilename}";
        continue
    fi
    set -Ee;
    printf "Running migration script %s...\n" "${script}"
    if grep '^COMMIT;$' "${script}" 1>/dev/null 2>&1; then
        shopt -s lastpipe
        set +Ee
        sed 's/^COMMIT;/ROLLBACK;/g' "${script}" | psql -v ON_ERROR_STOP=1 --quiet "postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}" 2>&1 | rollbackoutput=$(</dev/stdin)
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
        psql "postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}" --file="${script}" && \
        psql "postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}" -c "INSERT into _initialization_migrations ( filename, md5_hash ) VALUES ( '${sqlfilename}', '${md5}' )" && \
        printf "[PASS] Applied DB migration script '%s' successfully\n" "${sqlfilename}"
    else
        printf "[WARN]: No transactions present at script '%s', applying without prior testing\n" "${sqlfilename}"
        shopt -s lastpipe
        set +Ee
        psql -v ON_ERROR_STOP=1 "postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}" --file="${script}"  2>&1 | rollbackoutput=$(</dev/stdin)
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
