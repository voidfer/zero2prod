#!/usr/bin/env bash
set -eo pipefail
set -x

# Run script from project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"
echo "Running from project root: $(pwd)"

# Check dependencies
if ! command -v psql >/dev/null 2>&1; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi

if ! command -v sqlx >/dev/null 2>&1; then
  echo >&2 "sqlx not found. Installing sqlx-cli..."
  if ! command -v cargo >/dev/null 2>&1; then
    echo >&2 "Error: Cargo (Rust) is not installed. Install Rust first: https://rustup.rs/"
    exit 1
  fi
  cargo install --version='~0.7' sqlx-cli --no-default-features --features rustls,postgres
  export PATH="$HOME/.cargo/bin:$PATH"
fi

# Database configuration
DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

# Remove old container if exists
docker rm -f postgres_db 2>/dev/null || true

# Start Postgres if not skipping Docker
if [[ -z "${SKIP_DOCKER}" ]]; then
  docker run -d \
    --name postgres_db \
    -e POSTGRES_USER="${DB_USER}" \
    -e POSTGRES_PASSWORD="${DB_PASSWORD}" \
    -e POSTGRES_DB="${DB_NAME}" \
    -p "${DB_PORT}:5432" \
    postgres \
    -N 1000
fi

# Wait for Postgres to be ready
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d postgres -c '\q' 2>/dev/null; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done
>&2 echo "Postgres is up on port ${DB_PORT}"

# Set DATABASE_URL
export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"
echo "Using DATABASE_URL=$DATABASE_URL"

# Enable pgcrypto extension
psql "$DATABASE_URL" -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;"

# Create database if needed
sqlx database create

# Run migrations
if [ -d "migrations" ]; then
  sqlx migrate run
  >&2 echo "Migrations applied successfully!"
else
  >&2 echo "No migrations folder found, skipping migrations."
fi

