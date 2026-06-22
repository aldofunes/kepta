# Building from Source

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | stable (≥ 1.87) | all Rust crates |
| Docker | any recent | PostgreSQL for dev and tests |
| `cargo-sqlx` | latest | compile-time query checking |
| `cargo-watch` | latest | live reload during development |

Install Rust via [rustup](https://rustup.rs). Install cargo tools:

```bash
cargo install sqlx-cli --no-default-features --features postgres,rustls
cargo install cargo-watch
```

## Clone and Build

```bash
git clone https://github.com/aldofunes/kepta
cd kepta
cargo build
```

The first build takes a few minutes — aws-lc-rs requires compiling C code. Subsequent builds are fast.

## Local Development

### 1. Start PostgreSQL

```bash
docker run -d \
  --name kepta-pg \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=kepta \
  -p 5432:5432 \
  postgres:18
```

### 2. Run Migrations

```bash
export DATABASE_URL="postgres://postgres:password@localhost/kepta"
cargo sqlx migrate run --source crates/kepta-db/migrations
```

### 3. Configure the API

Create a `.env` file or export variables:

```bash
export DATABASE_URL="postgres://postgres:password@localhost/kepta"
export JWT_SECRET="dev-secret-at-least-32-chars-long!!"
export BASE_URL="http://localhost:8080"
export BILLING_ENABLED=false

# S3 — use MinIO locally or any S3-compatible service
export S3__BUCKET=kepta
export S3__REGION=us-east-1
export S3__ENDPOINT=http://localhost:9000
export S3__ACCESS_KEY_ID=minioadmin
export S3__SECRET_ACCESS_KEY=minioadmin

# SMTP — use MailHog or similar for dev
export SMTP__HOST=localhost
export SMTP__PORT=1025
export SMTP__USERNAME=""
export SMTP__PASSWORD=""
export SMTP__FROM="Kepta Dev <dev@localhost>"
```

> **Config convention:** the `config` crate maps env vars to struct fields using `__` as a nesting separator. `S3__BUCKET` maps to `s3.bucket` in `AppConfig`.

### 4. Run the API

```bash
cargo run --bin kepta-api
# or with live reload:
cargo watch -x 'run --bin kepta-api'
```

The server binds on `0.0.0.0:8080` by default. OpenAPI schema is at `http://localhost:8080/openapi.json`.

### 5. Run the Worker

```bash
cargo run --bin kepta-worker
```

The worker polls the `jobs` table every 5 seconds. Run it alongside the API in a separate terminal.

## Running Tests

### Integration tests (kepta-api)

Tests require a live PostgreSQL instance. Each test creates its own isolated database (UUID-named) and runs migrations automatically.

```bash
# Start a dedicated test database (different port to avoid conflicts with dev DB)
docker run -d \
  --name kepta-pg-test \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=kepta \
  -p 5433:5432 \
  postgres:18

DATABASE_URL="postgres://postgres:password@localhost:5433/kepta" cargo test --package kepta-api
```

Enable request/response logging during tests:
```bash
TEST_LOG=1 DATABASE_URL="..." cargo test --package kepta-api -- --nocapture
```

### Unit tests (all crates)

```bash
SQLX_OFFLINE=true cargo test --workspace \
  --exclude kepta-tauri \
  --exclude kepta-web \
  --exclude kepta-ui
```

### sqlx offline cache

The `.sqlx/` directories in `crates/kepta-db/` and `services/kepta-api/` contain the compile-time SQL metadata cache. Regenerate after changing any `query!` macro:

```bash
# From the crate directory that changed:
cd crates/kepta-db
DATABASE_URL="postgres://postgres:password@localhost/kepta" cargo sqlx prepare

cd ../../services/kepta-api
DATABASE_URL="postgres://postgres:password@localhost/kepta" cargo sqlx prepare
```

Commit the updated `.sqlx/` files. CI compiles with `SQLX_OFFLINE=true` so no database is needed in CI.

## Useful Commands

```bash
# Check all server-side crates compile (fast, no codegen)
SQLX_OFFLINE=true cargo check --package kepta-api --package kepta-db \
  --package kepta-auth --package kepta-jobs --package kepta-storage --package kepta-notify

# Run only a subset of integration tests
DATABASE_URL="..." cargo test --package kepta-api auth::

# Apply a new migration
cargo sqlx migrate add <name> --source crates/kepta-db/migrations

# Revert last migration
cargo sqlx migrate revert --source crates/kepta-db/migrations
```
