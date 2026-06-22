
CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    email           TEXT UNIQUE NOT NULL,
    display_name    TEXT,
    password_hash   TEXT,
    avatar_url      TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE oauth_accounts (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider        TEXT NOT NULL,
    provider_sub    TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider, provider_sub)
);

CREATE TABLE households (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    name            TEXT NOT NULL,
    address         JSONB NOT NULL DEFAULT '{}',
    year_built      INT,
    sqft            INT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE household_members (
    household_id    UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role            TEXT NOT NULL DEFAULT 'member' CHECK (role IN ('owner', 'member')),
    invited_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    joined_at       TIMESTAMPTZ,
    PRIMARY KEY (household_id, user_id)
);

CREATE TABLE rooms (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    household_id    UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    floor           INT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE items (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    household_id    UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    room_id         UUID REFERENCES rooms(id) ON DELETE SET NULL,
    category        TEXT NOT NULL CHECK (category IN ('appliance', 'system', 'fixture', 'other')),
    name            TEXT NOT NULL,
    brand           TEXT,
    model_number    TEXT,
    serial_number   TEXT,
    purchase_date   DATE,
    install_date    DATE,
    warranty_expiry DATE,
    notes           TEXT,
    deleted_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE maintenance_schedules (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    item_id         UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    household_id    UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    description     TEXT,
    frequency_unit  TEXT NOT NULL CHECK (frequency_unit IN ('day', 'week', 'month', 'year')),
    frequency_value INT NOT NULL CHECK (frequency_value > 0),
    next_due_at     DATE NOT NULL,
    last_completed_at DATE,
    source          TEXT NOT NULL DEFAULT 'manual' CHECK (source IN ('manual', 'ai_suggestion', 'template')),
    ai_suggestion_id UUID,
    deleted_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE maintenance_logs (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    schedule_id     UUID REFERENCES maintenance_schedules(id) ON DELETE SET NULL,
    household_id    UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    item_id         UUID REFERENCES items(id) ON DELETE SET NULL,
    completed_by    UUID NOT NULL REFERENCES users(id),
    completed_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    notes           TEXT,
    cost_cents      INT,
    currency        TEXT NOT NULL DEFAULT 'USD'
);

CREATE TABLE documents (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    household_id    UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    item_id         UUID REFERENCES items(id) ON DELETE SET NULL,
    type            TEXT NOT NULL CHECK (type IN ('manual', 'receipt', 'warranty', 'contract', 'work_order', 'other')),
    title           TEXT NOT NULL,
    file_key        TEXT NOT NULL,
    file_size_bytes BIGINT,
    mime_type       TEXT,
    ocr_text        TEXT,
    ocr_metadata    JSONB,
    ocr_status      TEXT NOT NULL DEFAULT 'pending' CHECK (ocr_status IN ('pending', 'processing', 'done', 'failed', 'skipped')),
    uploaded_by     UUID NOT NULL REFERENCES users(id),
    deleted_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE service_provider_contacts (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    household_id    UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    type            TEXT NOT NULL DEFAULT 'other' CHECK (type IN ('plumber', 'electrician', 'cleaner', 'hvac', 'general', 'other')),
    phone           TEXT,
    email           TEXT,
    website         TEXT,
    notes           TEXT,
    deleted_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE notification_preferences (
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    household_id    UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    channel         TEXT NOT NULL CHECK (channel IN ('email', 'push', 'webhook')),
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (user_id, household_id, channel)
);

CREATE TABLE webhooks (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    household_id    UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    url             TEXT NOT NULL,
    secret          TEXT NOT NULL,
    events          TEXT[] NOT NULL DEFAULT '{}',
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE notification_log (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id         UUID REFERENCES users(id) ON DELETE SET NULL,
    household_id    UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    channel         TEXT NOT NULL,
    event_type      TEXT NOT NULL,
    payload         JSONB NOT NULL DEFAULT '{}',
    status          TEXT NOT NULL CHECK (status IN ('sent', 'failed', 'skipped')),
    sent_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE subscription_plans (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    slug            TEXT UNIQUE NOT NULL,
    name            TEXT NOT NULL,
    price_cents     INT NOT NULL DEFAULT 0,
    household_limit INT,
    storage_limit_mb INT,
    document_limit  INT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE subscriptions (
    id                      UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id                 UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id                 UUID NOT NULL REFERENCES subscription_plans(id),
    status                  TEXT NOT NULL CHECK (status IN ('active', 'cancelled', 'past_due')),
    current_period_start    TIMESTAMPTZ NOT NULL,
    current_period_end      TIMESTAMPTZ NOT NULL,
    external_id             TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE household_subscriptions (
    household_id    UUID PRIMARY KEY REFERENCES households(id) ON DELETE CASCADE,
    subscription_id UUID NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE
);

CREATE TABLE jobs (
    id              UUID PRIMARY KEY DEFAULT uuidv7(),
    job_type        TEXT NOT NULL,
    payload         JSONB NOT NULL DEFAULT '{}',
    status          TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    attempts        INT NOT NULL DEFAULT 0,
    max_attempts    INT NOT NULL DEFAULT 3,
    error           TEXT,
    run_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default plans
INSERT INTO subscription_plans (id, slug, name, price_cents, household_limit, storage_limit_mb, document_limit)
VALUES
    (uuidv7(), 'free', 'Free', 0, 1, 500, 25),
    (uuidv7(), 'pro', 'Pro', 999, NULL, NULL, NULL);

-- Indexes
CREATE INDEX idx_household_members_user ON household_members(user_id);
CREATE INDEX idx_items_household ON items(household_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_schedules_household ON maintenance_schedules(household_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_schedules_due ON maintenance_schedules(next_due_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_documents_household ON documents(household_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_contacts_household ON service_provider_contacts(household_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_jobs_pending ON jobs(run_at) WHERE status = 'pending';
