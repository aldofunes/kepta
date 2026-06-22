# Data Model

## Entity-Relationship Diagram

```mermaid
erDiagram
    users {
        uuid id PK
        text email UK
        text display_name
        text password_hash "null if SSO-only"
        text avatar_url
        timestamptz created_at
        timestamptz updated_at
    }

    oauth_accounts {
        uuid id PK
        uuid user_id FK
        text provider "google | apple | oidc"
        text provider_sub
    }

    households {
        uuid id PK
        text name
        jsonb address "street, city, state, zip, country"
        int year_built
        int sqft
        timestamptz created_at
        timestamptz updated_at
    }

    household_members {
        uuid household_id FK
        uuid user_id FK
        text role "owner | member"
        timestamptz invited_at
        timestamptz joined_at
    }

    rooms {
        uuid id PK
        uuid household_id FK
        text name
        int floor
    }

    items {
        uuid id PK
        uuid household_id FK
        uuid room_id FK
        text category "appliance | system | fixture | other"
        text name
        text brand
        text model_number
        text serial_number
        date purchase_date
        date install_date
        date warranty_expiry
        text notes
        timestamptz created_at
        timestamptz updated_at
    }

    maintenance_schedules {
        uuid id PK
        uuid item_id FK
        uuid household_id FK
        text title
        text description
        text frequency_unit "day | week | month | year"
        int frequency_value
        date next_due_at
        date last_completed_at
        text source "manual | ai_suggestion | template"
        timestamptz created_at
        timestamptz updated_at
    }

    maintenance_logs {
        uuid id PK
        uuid schedule_id FK
        uuid household_id FK
        uuid item_id FK
        uuid completed_by FK
        timestamptz completed_at
        text notes
        int cost_cents
        text currency
    }

    documents {
        uuid id PK
        uuid household_id FK
        uuid item_id FK
        text type "manual | receipt | warranty | other"
        text title
        text file_key "S3 object key"
        bigint file_size_bytes
        text mime_type
        text ocr_text "Phase 2"
        jsonb ocr_metadata "Phase 2"
        text ocr_status "pending | processing | done | failed"
        uuid uploaded_by FK
        timestamptz created_at
        timestamptz updated_at
    }

    service_provider_contacts {
        uuid id PK
        uuid household_id FK
        text name
        text type "plumber | electrician | hvac | other"
        text phone
        text email
        text website
        text notes
        timestamptz created_at
        timestamptz updated_at
    }

    webhooks {
        uuid id PK
        uuid household_id FK
        text url
        text secret "HMAC signing secret"
        text[] events
        bool enabled
        timestamptz created_at
    }

    subscription_plans {
        uuid id PK
        text slug UK "free | pro"
        text name
        int price_cents "per household/month"
        int household_limit "null = unlimited"
        int storage_limit_mb "null = unlimited"
        int document_limit "null = unlimited"
    }

    subscriptions {
        uuid id PK
        uuid user_id FK
        uuid plan_id FK
        text status "active | cancelled | past_due"
        timestamptz current_period_start
        timestamptz current_period_end
        text external_id "Stripe subscription ID"
        timestamptz created_at
        timestamptz updated_at
    }

    jobs {
        uuid id PK
        jsonb payload "JobType enum"
        text status "pending | running | done | failed"
        int attempts
        text last_error
        timestamptz created_at
        timestamptz updated_at
    }

    users ||--o{ oauth_accounts : "authenticates via"
    users ||--o{ household_members : "belongs to"
    households ||--o{ household_members : "has"
    households ||--o{ rooms : "contains"
    households ||--o{ items : "contains"
    households ||--o{ documents : "has"
    households ||--o{ service_provider_contacts : "has"
    households ||--o{ webhooks : "has"
    rooms ||--o{ items : "contains"
    items ||--o{ maintenance_schedules : "has"
    items ||--o{ documents : "linked to"
    maintenance_schedules ||--o{ maintenance_logs : "produces"
    users ||--o{ maintenance_logs : "completed by"
    users ||--o{ subscriptions : "has"
    subscription_plans ||--o{ subscriptions : "defines"
```

## Table Reference

### `users`
Central identity record. `password_hash` is null for accounts created exclusively via OAuth/OIDC.

### `oauth_accounts`
One row per linked OAuth provider. A user can have multiple (Google + Apple). `(provider, provider_sub)` is unique.

### `households`
A property. `address` is a JSONB object `{street, city, state, zip, country}`. `year_built` and `sqft` are nullable metadata used for AI suggestions (Phase 2).

### `household_members`
Join table between users and households. `role` is `owner` (can delete household, manage members) or `member` (read/write content). `joined_at` is null until an invitation is accepted.

### `rooms`
Optional sub-grouping within a household. Items may be assigned to a room but are not required to be.

### `items`
An appliance, system, or fixture within a household. `category` drives the AI suggestion prompt (Phase 2). `warranty_expiry` triggers the `warranty.expiring` notification event.

### `maintenance_schedules`
A recurring task attached to an item. `frequency_unit` + `frequency_value` define the recurrence (e.g. `month`/`3` = every 3 months). `next_due_at` is advanced by the worker after each completion. `source` distinguishes user-created schedules from AI-generated ones.

### `maintenance_logs`
Immutable completion record. `cost_cents` enables lifetime cost tracking per item.

### `documents`
File metadata; the actual bytes live in S3 at `file_key`. `ocr_*` columns are populated by the background worker in Phase 2.

### `service_provider_contacts`
A private, per-household contact book entry. Not shared across households or instances (Phase 3 introduces the federated `directory_providers` table).

### `webhooks`
Outbound webhook subscription. The worker delivers signed payloads to `url` for each event in `events[]`. The `secret` is used to sign with HMAC-SHA256 (`X-Kepta-Signature` header).

### `subscription_plans`
Seed data; two rows: `free` and `pro`. `null` limits mean unlimited.

### `subscriptions`
One per user. `external_id` holds the Stripe subscription ID in the hosted tier. Self-hosted instances leave this null.

### `jobs`
Postgres-backed job queue. Claimed with `SELECT … FOR UPDATE SKIP LOCKED`. No Redis required.

## Free Tier Limits

| Resource | Free | Pro |
|----------|------|-----|
| Households | 1 | Unlimited |
| Members per household | 2 | Unlimited |
| Documents per household | 25 | Unlimited |
| Storage per household | 500 MB | Unlimited |
| Webhooks | 0 | Unlimited |

Limits are enforced at the API level. Self-hosted instances set `BILLING_ENABLED=false` to disable all checks.
