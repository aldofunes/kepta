# Deployment

## Deployment Options

| Mode | When to use |
|------|------------|
| Docker Compose | Single-node self-hosted, development, small teams |
| Kubernetes + Helm | Production self-hosted, high availability |
| Hosted (kepta.app) | Managed — no ops required |

## Docker Compose (Single Node)

Create `docker-compose.yml`:

```yaml
services:
  postgres:
    image: postgres:18
    restart: unless-stopped
    environment:
      POSTGRES_DB: kepta
      POSTGRES_USER: kepta
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U kepta"]
      interval: 5s
      retries: 10

  minio:
    image: minio/minio
    restart: unless-stopped
    command: server /data --console-address ":9001"
    environment:
      MINIO_ROOT_USER: ${MINIO_ROOT_USER}
      MINIO_ROOT_PASSWORD: ${MINIO_ROOT_PASSWORD}
    volumes:
      - minio_data:/data

  api:
    image: ghcr.io/aldofunes/kepta/api:latest
    restart: unless-stopped
    depends_on:
      postgres:
        condition: service_healthy
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://kepta:${POSTGRES_PASSWORD}@postgres/kepta
      JWT_SECRET: ${JWT_SECRET}
      BASE_URL: ${BASE_URL}
      BILLING_ENABLED: "false"
      S3__BUCKET: kepta
      S3__REGION: us-east-1
      S3__ENDPOINT: http://minio:9000
      S3__ACCESS_KEY_ID: ${MINIO_ROOT_USER}
      S3__SECRET_ACCESS_KEY: ${MINIO_ROOT_PASSWORD}
      SMTP__HOST: ${SMTP_HOST}
      SMTP__PORT: ${SMTP_PORT}
      SMTP__USERNAME: ${SMTP_USERNAME}
      SMTP__PASSWORD: ${SMTP_PASSWORD}
      SMTP__FROM: ${SMTP_FROM}

  worker:
    image: ghcr.io/aldofunes/kepta/worker:latest
    restart: unless-stopped
    depends_on:
      postgres:
        condition: service_healthy
    environment:
      DATABASE_URL: postgres://kepta:${POSTGRES_PASSWORD}@postgres/kepta
      SMTP__HOST: ${SMTP_HOST}
      SMTP__PORT: ${SMTP_PORT}
      SMTP__USERNAME: ${SMTP_USERNAME}
      SMTP__PASSWORD: ${SMTP_PASSWORD}
      SMTP__FROM: ${SMTP_FROM}

volumes:
  postgres_data:
  minio_data:
```

Create `.env`:
```bash
POSTGRES_PASSWORD=change-me
MINIO_ROOT_USER=kepta
MINIO_ROOT_PASSWORD=change-me
JWT_SECRET=change-me-use-at-least-32-random-chars
BASE_URL=https://kepta.yourdomain.com
SMTP_HOST=smtp.mailgun.org
SMTP_PORT=587
SMTP_USERNAME=postmaster@yourdomain.com
SMTP_PASSWORD=change-me
SMTP_FROM=Kepta <noreply@yourdomain.com>
```

Start:
```bash
docker compose up -d
```

The API runs database migrations automatically on startup. Put a reverse proxy (nginx, Caddy, Traefik) in front of port 8080 to handle TLS termination.

## Kubernetes + Helm

### Install

```bash
helm repo add kepta https://charts.kepta.app
helm repo update

helm install kepta kepta/kepta \
  --namespace kepta \
  --create-namespace \
  -f values.yaml
```

### Minimal `values.yaml`

```yaml
image:
  tag: "0.1.0"

api:
  baseUrl: "https://kepta.yourdomain.com"
  billingEnabled: false
  jwtSecret: "change-me-use-at-least-32-random-chars"

s3:
  bucket: kepta
  region: us-east-1
  endpoint: ""                   # leave blank for AWS S3
  accessKeyId: "YOUR_KEY_ID"
  secretAccessKey: "YOUR_SECRET"

smtp:
  host: smtp.mailgun.org
  port: 587
  username: "postmaster@yourdomain.com"
  password: "change-me"
  from: "Kepta <noreply@yourdomain.com>"

postgresql:
  enabled: true
  auth:
    database: kepta
    username: kepta
    password: "change-me"

ingress:
  enabled: true
  className: nginx
  hosts:
    - host: kepta.yourdomain.com
      paths: [{ path: /, pathType: Prefix }]
  tls:
    - secretName: kepta-tls
      hosts: [kepta.yourdomain.com]
```

### Deployment Flow

```mermaid
graph TD
    CI[CI build] -->|push image| GHR[ghcr.io registry]
    GHR -->|helm upgrade| K8S

    subgraph K8S[Kubernetes cluster]
        IN[Ingress / TLS]
        API[kepta-api\nDeployment × 2]
        WRK[kepta-worker\nDeployment × 1]
        PG[(PostgreSQL\nStatefulSet)]
        S3E[(S3 / MinIO)]
    end

    IN --> API
    API --> PG
    API --> S3E
    WRK --> PG
```

### Upgrade

```bash
helm upgrade kepta kepta/kepta -n kepta -f values.yaml --set image.tag=0.2.0
```

Migrations run automatically when the new `kepta-api` pod starts. Kubernetes rolling update ensures the old pod stays up until the new one passes its readiness probe.

## Environment Variables Reference

All variables for `kepta-api`. The worker only needs `DATABASE_URL` and the `SMTP__*` group.

```mermaid
graph LR
    subgraph "Required"
        DU[DATABASE_URL]
        JS[JWT_SECRET]
        BU[BASE_URL]
    end

    subgraph "S3 (required)"
        S3B[S3__BUCKET]
        S3R[S3__REGION]
        S3K[S3__ACCESS_KEY_ID]
        S3S[S3__SECRET_ACCESS_KEY]
        S3E[S3__ENDPOINT\noptional - for MinIO]
    end

    subgraph "SMTP (required)"
        SH[SMTP__HOST]
        SP[SMTP__PORT]
        SU[SMTP__USERNAME]
        SW[SMTP__PASSWORD]
        SF[SMTP__FROM]
    end

    subgraph "Optional"
        BE[BILLING_ENABLED\ndefault: false]
        GC[GOOGLE_OAUTH__CLIENT_ID\nGOOGLE_OAUTH__CLIENT_SECRET]
        AC[APPLE_OAUTH__CLIENT_ID\nAPPLE_OAUTH__CLIENT_SECRET]
        APNS[APNS__TEAM_ID\nAPNS__KEY_ID\nAPNS__PRIVATE_KEY_PATH\nAPNS__BUNDLE_ID\nAPNS__SANDBOX]
    end
```

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |
| `JWT_SECRET` | HS256 signing secret — minimum 32 characters, keep secret |
| `BASE_URL` | Public base URL (no trailing slash), used in email links |
| `BILLING_ENABLED` | Set `true` only for the hosted tier; disables all plan limits when `false` |
| `S3__BUCKET` | S3 bucket name |
| `S3__REGION` | AWS region (e.g. `us-east-1`) |
| `S3__ENDPOINT` | Custom endpoint URL for MinIO or other S3-compatible stores |
| `S3__ACCESS_KEY_ID` | S3 access key |
| `S3__SECRET_ACCESS_KEY` | S3 secret |
| `SMTP__HOST` | SMTP server hostname |
| `SMTP__PORT` | SMTP port (587 for STARTTLS, 465 for TLS) |
| `SMTP__USERNAME` | SMTP authentication username |
| `SMTP__PASSWORD` | SMTP authentication password |
| `SMTP__FROM` | From address for outgoing email |
| `GOOGLE_OAUTH__CLIENT_ID` | Google OAuth 2.0 client ID |
| `GOOGLE_OAUTH__CLIENT_SECRET` | Google OAuth 2.0 client secret |
| `APPLE_OAUTH__CLIENT_ID` | Apple Sign-In service ID |
| `APPLE_OAUTH__CLIENT_SECRET` | Apple Sign-In client secret |
| `APNS__TEAM_ID` | Apple Developer team ID for push notifications |
| `APNS__KEY_ID` | APNs key ID |
| `APNS__PRIVATE_KEY_PATH` | Path to APNs `.p8` private key file |
| `APNS__BUNDLE_ID` | iOS app bundle identifier |
| `APNS__SANDBOX` | `true` to use APNs sandbox endpoint |

## Database Migrations

Migrations live in `crates/kepta-db/migrations/` and are embedded in the binary via `sqlx::migrate!()`. They run automatically at API startup.

To run migrations manually (e.g. for inspection or rollback):

```bash
# Apply
cargo sqlx migrate run \
  --source crates/kepta-db/migrations \
  --database-url postgres://...

# Revert last migration
cargo sqlx migrate revert \
  --source crates/kepta-db/migrations \
  --database-url postgres://...

# List migration status
cargo sqlx migrate info \
  --source crates/kepta-db/migrations \
  --database-url postgres://...
```

## Health Check

`GET /health` returns `200 OK` with an empty body. Use this for:
- Kubernetes liveness / readiness probes
- Docker Compose `healthcheck`
- Load balancer health checks
