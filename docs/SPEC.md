# Kepta — Product Specification

> Home management platform. Feature-parity clone of HomeKeep. Open source, self-hostable on Kubernetes, with a hosted SaaS subscription tier.

---

## 1. Product Overview

Kepta helps homeowners track, maintain, and document their properties. Each account can manage multiple households; each household can have multiple members. The hosted version is subscription-based (per household/month). Self-hosters run their own instance on k8s.

### Target Users
- Homeowners managing one or more properties
- Landlords / property managers with multiple units
- Households with multiple members sharing one property

---

## 2. Platforms

| Platform | Stack |
|----------|-------|
| Backend API | Rust · axum · sqlx · tokio · aide (OpenAPI) · tracing |
| Database | PostgreSQL |
| File Storage | S3-compatible object storage |
| Web app | Leptos (Rust → WASM) |
| Desktop app | Tauri + Leptos (shared frontend) |
| iOS app | Swift (native) |

---

## 3. Phases

### Phase 1 — Core (MVP)
Household management · Inventory · Maintenance scheduling · Reminders · Document storage · Service provider contacts · Auth

### Phase 2 — Intelligence
AI-assisted maintenance suggestions (cached in DB) · OCR for uploaded documents

### Phase 3 — Integrations
JMAP calendar sync · Federated contractor directory

---

## 4. Data Model

### Users & Households

```
users
  id              uuid PK
  email           text UNIQUE NOT NULL
  display_name    text
  password_hash   text nullable          -- null if SSO-only
  avatar_url      text nullable
  created_at      timestamptz
  updated_at      timestamptz

oauth_accounts
  id              uuid PK
  user_id         uuid FK users
  provider        text                   -- "google" | "apple" | "oidc"
  provider_sub    text
  UNIQUE(provider, provider_sub)

households
  id              uuid PK
  name            text NOT NULL
  address         jsonb                  -- {street, city, state, zip, country}
  year_built      int nullable
  sqft            int nullable
  created_at      timestamptz
  updated_at      timestamptz

household_members
  household_id    uuid FK households
  user_id         uuid FK users
  role            text                   -- "owner" | "member"
  invited_at      timestamptz
  joined_at       timestamptz nullable
  PRIMARY KEY(household_id, user_id)
```

### Inventory

```
rooms
  id              uuid PK
  household_id    uuid FK households
  name            text                   -- "Kitchen", "Master Bedroom", etc.
  floor           int nullable

items
  id              uuid PK
  household_id    uuid FK households
  room_id         uuid FK rooms nullable
  category        text                   -- "appliance" | "system" | "fixture" | "other"
  name            text NOT NULL
  brand           text nullable
  model_number    text nullable
  serial_number   text nullable
  purchase_date   date nullable
  install_date    date nullable
  warranty_expiry date nullable
  notes           text nullable
  created_at      timestamptz
  updated_at      timestamptz
```

### Maintenance

```
maintenance_schedules
  id              uuid PK
  item_id         uuid FK items
  household_id    uuid FK households
  title           text NOT NULL
  description     text nullable
  frequency_unit  text                   -- "day" | "week" | "month" | "year"
  frequency_value int NOT NULL           -- e.g. 3 months → unit=month, value=3
  next_due_at     date NOT NULL
  last_completed_at date nullable
  source          text                   -- "manual" | "ai_suggestion" | "template"
  ai_suggestion_id uuid FK ai_suggestions nullable
  created_at      timestamptz
  updated_at      timestamptz

maintenance_logs
  id              uuid PK
  schedule_id     uuid FK maintenance_schedules nullable
  household_id    uuid FK households
  item_id         uuid FK items nullable
  completed_by    uuid FK users
  completed_at    timestamptz
  notes           text nullable
  cost_cents      int nullable
  currency        text default 'USD'
```

### AI Suggestions (Phase 2)

```
ai_suggestions
  id              uuid PK
  cache_key       text UNIQUE NOT NULL   -- hash of (category, brand, model, climate_zone)
  item_category   text
  brand           text nullable
  model_number    text nullable
  climate_zone    text nullable
  suggestions     jsonb                  -- [{title, description, frequency_unit, frequency_value}]
  model_version   text
  created_at      timestamptz
  expires_at      timestamptz
```

### Documents

```
documents
  id              uuid PK
  household_id    uuid FK households
  item_id         uuid FK items nullable
  type            text                   -- "manual" | "receipt" | "warranty" | "contract" | "work_order" | "other"
  title           text NOT NULL
  file_key        text NOT NULL          -- S3 object key
  file_size_bytes bigint
  mime_type       text
  ocr_text        text nullable          -- Phase 2
  ocr_metadata    jsonb nullable         -- Phase 2: extracted dates, amounts, etc.
  ocr_status      text default 'pending' -- "pending" | "processing" | "done" | "failed"
  uploaded_by     uuid FK users
  created_at      timestamptz
  updated_at      timestamptz
```

### Service Providers

```
-- Private per-household contact book
service_provider_contacts
  id                      uuid PK
  household_id            uuid FK households
  name                    text NOT NULL
  type                    text           -- "plumber" | "electrician" | "cleaner" | "hvac" | "general" | "other"
  phone                   text nullable
  email                   text nullable
  website                 text nullable
  notes                   text nullable
  directory_provider_id   uuid FK directory_providers nullable  -- Phase 3: link to shared directory
  created_at              timestamptz
  updated_at              timestamptz

-- Phase 3: Federated shared directory
directory_providers
  id              uuid PK
  name            text NOT NULL
  type            text
  address         jsonb nullable
  phone           text nullable
  email           text nullable
  website         text nullable
  geo             point nullable         -- lat/lon for radius search
  average_rating  numeric(3,2) nullable
  rating_count    int default 0
  source_instance text                   -- URL of originating Kepta instance
  created_at      timestamptz

directory_reviews
  id              uuid PK
  provider_id     uuid FK directory_providers
  user_id         uuid FK users
  rating          int CHECK(rating BETWEEN 1 AND 5)
  body            text nullable
  created_at      timestamptz
  UNIQUE(provider_id, user_id)
```

### Notifications

```
notification_preferences
  user_id         uuid FK users
  household_id    uuid FK households
  channel         text                   -- "email" | "push" | "webhook"
  enabled         bool default true
  PRIMARY KEY(user_id, household_id, channel)

webhooks
  id              uuid PK
  household_id    uuid FK households
  url             text NOT NULL
  secret          text NOT NULL          -- HMAC signing secret
  events          text[]                 -- ["maintenance.due", "document.uploaded", ...]
  enabled         bool default true
  created_at      timestamptz

notification_log
  id              uuid PK
  user_id         uuid FK users nullable
  household_id    uuid FK households
  channel         text
  event_type      text
  payload         jsonb
  status          text                   -- "sent" | "failed" | "skipped"
  sent_at         timestamptz
```

### Subscriptions & Billing

```
subscription_plans
  id              uuid PK
  slug            text UNIQUE            -- "free" | "pro"
  name            text
  price_cents     int                    -- per household per month
  household_limit int nullable           -- null = unlimited
  storage_limit_mb int nullable          -- null = unlimited
  document_limit  int nullable

subscriptions
  id              uuid PK
  user_id         uuid FK users
  plan_id         uuid FK subscription_plans
  status          text                   -- "active" | "cancelled" | "past_due"
  current_period_start timestamptz
  current_period_end   timestamptz
  external_id     text nullable          -- Stripe subscription ID
  created_at      timestamptz
  updated_at      timestamptz

household_subscriptions
  household_id    uuid FK households PK
  subscription_id uuid FK subscriptions
```

---

## 5. Free Tier Limits

| Resource | Free | Pro |
|----------|------|-----|
| Households | 1 | Unlimited |
| Members per household | 2 | Unlimited |
| Documents per household | 25 | Unlimited |
| Storage per household | 500 MB | Unlimited |
| Webhooks | 0 | Unlimited |

---

## 6. API Design

- **Transport**: HTTP/JSON via axum
- **Schema**: OpenAPI 3.1 generated via `aide`
- **Auth**: JWT (access token short-lived, refresh token long-lived stored in httpOnly cookie)
- **Versioning**: URL prefix `/api/v1/`
- **Errors**: RFC 9457 Problem Details (`application/problem+json`)
- **Pagination**: cursor-based (`after`, `limit`)
- **Rate limiting**: per-user, per-IP

### Key Endpoint Groups

```
POST   /api/v1/auth/register
POST   /api/v1/auth/login
POST   /api/v1/auth/logout
POST   /api/v1/auth/refresh
GET    /api/v1/auth/oauth/:provider
GET    /api/v1/auth/oauth/:provider/callback

GET    /api/v1/households
POST   /api/v1/households
GET    /api/v1/households/{id}
PATCH  /api/v1/households/{id}
DELETE /api/v1/households/{id}
GET    /api/v1/households/{id}/members
POST   /api/v1/households/{id}/members/invite
DELETE /api/v1/households/{id}/members/:user_id

GET    /api/v1/households/{id}/rooms
POST   /api/v1/households/{id}/rooms
PATCH  /api/v1/households/{id}/rooms/:room_id
DELETE /api/v1/households/{id}/rooms/:room_id

GET    /api/v1/households/{id}/items
POST   /api/v1/households/{id}/items
GET    /api/v1/households/{id}/items/:item_id
PATCH  /api/v1/households/{id}/items/:item_id
DELETE /api/v1/households/{id}/items/:item_id

GET    /api/v1/households/{id}/schedules
POST   /api/v1/households/{id}/schedules
PATCH  /api/v1/households/{id}/schedules/:schedule_id
DELETE /api/v1/households/{id}/schedules/:schedule_id
POST   /api/v1/households/{id}/schedules/:schedule_id/complete

GET    /api/v1/households/{id}/documents
POST   /api/v1/households/{id}/documents          -- multipart upload
GET    /api/v1/households/{id}/documents/:doc_id
PATCH  /api/v1/households/{id}/documents/:doc_id
DELETE /api/v1/households/{id}/documents/:doc_id
GET    /api/v1/households/{id}/documents/:doc_id/download

GET    /api/v1/households/{id}/contacts
POST   /api/v1/households/{id}/contacts
PATCH  /api/v1/households/{id}/contacts/:contact_id
DELETE /api/v1/households/{id}/contacts/:contact_id

GET    /api/v1/households/{id}/webhooks
POST   /api/v1/households/{id}/webhooks
PATCH  /api/v1/households/{id}/webhooks/:webhook_id
DELETE /api/v1/households/{id}/webhooks/:webhook_id

GET    /api/v1/users/me
PATCH  /api/v1/users/me
DELETE /api/v1/users/me

GET    /api/v1/subscriptions/plans
GET    /api/v1/subscriptions/current
POST   /api/v1/subscriptions/checkout
POST   /api/v1/subscriptions/portal

-- Phase 2
POST   /api/v1/households/{id}/items/:item_id/suggest-maintenance

-- Phase 3
GET    /api/v1/directory/providers
GET    /api/v1/directory/providers/{id}
POST   /api/v1/directory/providers/{id}/reviews
GET    /api/v1/jmap/calendars/:household_id
```

---

## 7. Notification Events

| Event | Trigger |
|-------|---------|
| `maintenance.due` | N days before `next_due_at` (configurable, default 3) |
| `maintenance.overdue` | Day of and each week after `next_due_at` until completed |
| `document.uploaded` | New document added to household |
| `member.invited` | Household invitation sent |
| `member.joined` | Invitation accepted |
| `warranty.expiring` | 30 days before `warranty_expiry` on an item |
| `subscription.expiring` | 7 days before period end |

Webhook payloads are signed with HMAC-SHA256 (`X-Kepta-Signature` header).

---

## 8. Auth Flows

### Email/Password
1. Register → verify email → login → JWT issued
2. Forgot password → reset link (expires 1h)

### OAuth (Google, Apple)
1. Redirect to provider → callback → upsert `oauth_accounts` → JWT issued
2. If email already exists as password account, link accounts

### OIDC/SAML (self-hosters)
- Configured via environment variables per instance
- OIDC: standard code flow
- SAML: SP-initiated, metadata auto-generated at `/auth/saml/metadata`

---

## 9. Repository Structure

```
kepta/  (Cargo workspace)
│
├── crates/
│   ├── kepta-types/        # Shared API types, domain enums — WASM-compatible
│   ├── kepta-domain/       # Validation, schedule calc, limit rules — WASM-compatible
│   ├── kepta-db/           # sqlx models, queries, migrations — server only
│   ├── kepta-auth/         # JWT, OAuth2, OIDC, SAML — server only
│   ├── kepta-storage/      # S3 trait + impl — server only
│   ├── kepta-notify/       # email (lettre), APNs, Web Push, webhooks
│   ├── kepta-jobs/         # job definitions, Postgres-backed queue
│   ├── kepta-sync/         # delta sync engine — desktop only
│   └── kepta-localdb/      # LocalStore trait + SQLite impl — desktop only
│
├── services/
│   ├── kepta-api/          # axum HTTP server (routes, handlers, middleware)
│   └── kepta-worker/       # background job processor
│
├── frontend/
│   ├── kepta-ui/           # Leptos component library (shared: web + desktop)
│   └── kepta-web/          # web app entry point (Trunk)
│
├── desktop/
│   └── kepta-desktop/      # Tauri shell
│
├── ios/                    # Swift package (outside Cargo workspace)
│   ├── Sources/Kepta/
│   │   ├── API/            # swift-openapi-generator client
│   │   ├── Sync/           # delta sync in Swift
│   │   ├── DB/             # local SQLite
│   │   └── UI/             # SwiftUI views
│   └── Package.swift
│
└── infra/
    ├── k8s/
    │   ├── base/
    │   └── overlays/
    └── helm/               # Helm chart for self-hosters
```

### Crate Dependency Graph

```
kepta-types  ←─── kepta-web (WASM)
     ↑         └── kepta-ui  (WASM)
kepta-domain        kepta-desktop (native)
     ↑
kepta-db ←── kepta-auth
     ↑             ↑
kepta-storage  kepta-notify
          ↑        ↑
       kepta-jobs──┘
            ↑
    kepta-api    kepta-worker   (binaries)

kepta-sync ←── kepta-localdb ←── kepta-desktop
```

### Runtime Ownership

| Concern | Owner |
|---------|-------|
| HTTP API, auth, OpenAPI schema | `kepta-api` |
| Maintenance reminder scheduling | `kepta-worker` |
| Webhook dispatch | `kepta-worker` |
| OCR processing (Phase 2) | `kepta-worker` |
| AI suggestion calls (Phase 2) | `kepta-worker` |
| SSE push to connected web/desktop clients | `kepta-api` |
| Local SQLite + delta sync | `kepta-desktop` |
| iOS local SQLite + delta sync | iOS `Sync/` module |

---

## 10. Offline Support

### Web
Optimistic UI only. Leptos reactive signals hold in-memory state; API calls fire immediately and state reverts on error. No persistence across page reloads. No IndexedDB, no Service Worker.

### Desktop (Tauri)
Full offline via embedded SQLite (`kepta-localdb`). Delta sync on reconnect via `kepta-sync`:
- Every table has `updated_at` + soft-delete `deleted_at`
- `GET /api/v1/households/{id}/sync?since=<timestamp>` returns changed records
- `POST /api/v1/households/{id}/sync` pushes local writes
- Conflict resolution: last-write-wins by `updated_at` (home management has negligible write collisions)
- Pending local writes stored in a `pending_writes` SQLite table
- Documents: configurable local file cache (LRU eviction)

### iOS
Same delta sync protocol implemented in Swift against local SQLite.

### Migrations
Applied at `kepta-api` startup via `sqlx::migrate!()`. Migration files live in `crates/kepta-db/migrations/`. k8s deployment uses a Postgres readiness probe to ensure DB is ready before the API pod starts.

---

## 11. Infrastructure (k8s)

### Services
| Service | Image |
|---------|-------|
| `kepta-api` | Rust binary (distroless) |
| `kepta-web` | Nginx serving WASM bundle |
| `postgres` | postgres:18 |
| `redis` | redis:7 (job queue, rate limiting, push) |
| `minio` | minio/minio (S3-compatible, optional — use real S3 in hosted) |

### Config (environment variables)
```
DATABASE_URL
REDIS_URL
S3_BUCKET / S3_ENDPOINT / S3_REGION / AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY
JWT_SECRET
SMTP_HOST / SMTP_PORT / SMTP_USER / SMTP_PASS / SMTP_FROM
STRIPE_SECRET_KEY / STRIPE_WEBHOOK_SECRET   # hosted only
OIDC_CLIENT_ID / OIDC_CLIENT_SECRET / OIDC_ISSUER_URL
GOOGLE_CLIENT_ID / GOOGLE_CLIENT_SECRET
APPLE_CLIENT_ID / APPLE_TEAM_ID / APPLE_KEY_ID / APPLE_PRIVATE_KEY
BASE_URL
```

### Helm chart
Self-hosters install via:
```
helm repo add kepta https://charts.kepta.app
helm install kepta kepta/kepta -f values.yaml
```

---

## 11. Phase 2 — AI Suggestions

- On `POST /api/v1/households/{id}/items/:item_id/suggest-maintenance`, the backend computes a `cache_key` from `(category, brand, model_number, climate_zone)`.
- If a non-expired `ai_suggestions` row exists for that key, return cached suggestions.
- Otherwise, call the configured AI provider (Claude), store the result with `expires_at = now() + 90 days`, and return.
- Returned suggestions are pre-populated into `maintenance_schedules` with `source = 'ai_suggestion'`.
- Climate zone is derived from the household's address (via a lookup table or geocoding API).

---

## 12. Phase 2 — OCR

- On document upload, enqueue an OCR job (via Redis queue / background Tokio task).
- Run OCR via `tesseract-rs` bindings (self-hosted) or a configurable external API.
- Extract and store: `ocr_text`, and structured metadata (`warranty_expiry`, `purchase_date`, `total_amount`, `vendor`) into `ocr_metadata` jsonb.
- If warranty/purchase dates are found and the document is linked to an item, offer to auto-update the item's fields.

---

## 13. Phase 3 — JMAP Calendar Sync

- Implement JMAP Core + CalDAV/JMAP Calendars (RFC 8620, RFC 8621).
- Each household's maintenance schedules are exposed as a JMAP calendar.
- Users can point any JMAP-compatible client (Fastmail, etc.) at `https://kepta.app/jmap/` to sync due tasks.
- Write-back: completing a task via the calendar client marks it done in Kepta.

---

## 14. Phase 3 — Federated Contractor Directory

- Each Kepta instance maintains its own `directory_providers` table.
- Instances announce themselves and sync provider records via a lightweight ActivityPub-inspired protocol (or simpler: periodic HTTP polling of a central registry operated by the Kepta project).
- Reviews are signed with the originating instance's key and replicated.
- Private `service_provider_contacts` can be linked to a `directory_providers` record for one-click import of shared data.
- Users can submit new providers from their private book to the shared directory.

---

## 15. Subscription Enforcement

- Middleware checks `household_subscriptions` + `subscription_plans` limits on each write.
- Free tier hard limits enforced at API level (return `402 Payment Required` with upgrade link).
- Self-hosted instances: billing is disabled by default (`BILLING_ENABLED=false`); all limits are removed.
