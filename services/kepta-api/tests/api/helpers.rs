use kepta_api::{
    config::{AppConfig, S3Config, SmtpConfig},
    router,
    state::AppState,
};
use kepta_auth::jwt::JwtKeys;
use kepta_jobs::JobQueue;
use kepta_storage::memory::MemoryStorage;
use reqwest::Response;
use serde_json::{json, Value};
use sqlx::{
    postgres::{PgConnectOptions, PgConnection},
    Connection, PgPool,
};
use std::sync::{Arc, OnceLock};
use tokio::net::TcpListener;
use uuid::Uuid;

static TRACING: OnceLock<()> = OnceLock::new();

// ── Test application ──────────────────────────────────────────────────────────

pub struct TestApp {
    pub address: String,
    pub db: PgPool,
    pub jwt: Arc<JwtKeys>,
    client: reqwest::Client,
}

pub struct TestUser {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub token: String,
}

/// Spawn the full application on a random OS port with an isolated database.
/// Mirrors zero2prod's spawn_app() pattern: real TCP server, reqwest client.
pub async fn spawn_app() -> TestApp {
    TRACING.get_or_init(|| {
        if std::env::var("TEST_LOG").is_ok() {
            tracing_subscriber::fmt().with_env_filter("debug").init();
        }
    });

    let db_name = Uuid::now_v7().to_string();
    let db = configure_database(&db_name).await;

    let jwt = Arc::new(JwtKeys::new("test-secret-that-is-long-enough-ok!"));
    let state = AppState {
        db: db.clone(),
        cfg: test_config(),
        jwt: jwt.clone(),
        storage: Arc::new(MemoryStorage::new()),
        jobs: Arc::new(JobQueue::new(db.clone())),
    };
    let app = router::build(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
        db,
        jwt,
        client: reqwest::Client::new(),
    }
}

/// Create a fresh Postgres database with a unique name and run all migrations.
async fn configure_database(db_name: &str) -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/kepta".to_string());

    let opts: PgConnectOptions = database_url.parse().unwrap();

    // Connect to the default "postgres" database to CREATE the test database.
    let mut admin = PgConnection::connect_with(&opts.clone().database("postgres"))
        .await
        .expect("failed to connect to postgres for database creation");
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!("CREATE DATABASE \"{}\"", db_name).as_str()))
        .execute(&mut admin)
        .await
        .expect("failed to create test database");

    // Connect to the new database and apply migrations.
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_with(opts.database(db_name))
        .await
        .expect("failed to connect to test database");
    kepta_db::migrate(&pool).await.expect("failed to run migrations on test database");

    pool
}

fn test_config() -> AppConfig {
    AppConfig {
        database_url: String::new(),
        jwt_secret: "test-secret-that-is-long-enough-ok!".to_string(),
        base_url: "http://localhost".to_string(),
        billing_enabled: false,
        s3: S3Config {
            bucket: "test".to_string(),
            endpoint: None,
            region: "us-east-1".to_string(),
            access_key_id: "test".to_string(),
            secret_access_key: "test".to_string(),
        },
        smtp: SmtpConfig {
            host: "localhost".to_string(),
            port: 1025,
            username: String::new(),
            password: String::new(),
            from: "test@kepta.local".to_string(),
        },
        apns: None,
        google_oauth: None,
        apple_oauth: None,
    }
}

// ── API client methods ────────────────────────────────────────────────────────
// Centralised here so renaming a route only requires a single change.

impl TestApp {
    pub async fn health(&self) -> Response {
        self.client
            .get(format!("{}/health", self.address))
            .send()
            .await
            .unwrap()
    }

    pub async fn register(&self, body: &Value) -> Response {
        self.client
            .post(format!("{}/api/v1/auth/register", self.address))
            .json(body)
            .send()
            .await
            .unwrap()
    }

    pub async fn login(&self, body: &Value) -> Response {
        self.client
            .post(format!("{}/api/v1/auth/login", self.address))
            .json(body)
            .send()
            .await
            .unwrap()
    }

    pub async fn get_me(&self, token: &str) -> Response {
        self.client
            .get(format!("{}/api/v1/users/me", self.address))
            .bearer_auth(token)
            .send()
            .await
            .unwrap()
    }

    pub async fn post_household(&self, body: &Value, token: &str) -> Response {
        self.client
            .post(format!("{}/api/v1/households", self.address))
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .unwrap()
    }

    pub async fn get_households(&self, token: &str) -> Response {
        self.client
            .get(format!("{}/api/v1/households", self.address))
            .bearer_auth(token)
            .send()
            .await
            .unwrap()
    }

    pub async fn get_household(&self, id: Uuid, token: &str) -> Response {
        self.client
            .get(format!("{}/api/v1/households/{id}", self.address))
            .bearer_auth(token)
            .send()
            .await
            .unwrap()
    }

    pub async fn get_unauthenticated(&self, path: &str) -> Response {
        self.client
            .get(format!("{}{path}", self.address))
            .send()
            .await
            .unwrap()
    }
}

// ── Fixture helpers ───────────────────────────────────────────────────────────

/// Register a new user via the API and return their credentials + token.
pub async fn register_user(app: &TestApp) -> TestUser {
    let email = format!("user-{}@test.kepta.dev", Uuid::now_v7());
    let password = "test-password-123!";

    let res = app
        .register(&json!({
            "email": email,
            "password": password,
            "display_name": "Test User"
        }))
        .await;

    assert_eq!(res.status().as_u16(), 200, "registration should succeed");
    let body: Value = res.json().await.unwrap();

    TestUser {
        id: Uuid::parse_str(body["user"]["id"].as_str().unwrap()).unwrap(),
        email,
        password: password.to_string(),
        token: body["access_token"].as_str().unwrap().to_string(),
    }
}
