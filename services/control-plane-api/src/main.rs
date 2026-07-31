use axum::{
    routing::{get, post},
    Router,
    Json,
};
use serde::Serialize;

use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod middleware;
mod routes;
mod models;
mod services;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    server_time: String,
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        server_time: chrono::Utc::now().to_rfc3339(),
    })
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // Load configuration
    let config = config::Config::load().expect("Failed to load configuration");

    // Connect to PostgreSQL
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database migrations completed successfully");

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(config.cors_origins.iter().map(|s| s.parse().unwrap()).collect::<Vec<_>>())
        .allow_methods(Any)
        .allow_headers(Any);

    // Build application routes
    let app = Router::new()
        .route("/api/v1/health", get(health_handler))
        .route("/api/v1/auth/login", post(routes::auth::login))
        .route("/api/v1/auth/refresh", post(routes::auth::refresh_token))
        .route("/api/v1/auth/logout", post(routes::auth::logout))
        .route("/api/v1/devices/activate", post(routes::devices::activate))
        .route("/api/v1/devices/activation-challenge", post(routes::devices::create_challenge))
        .route("/api/v1/devices/heartbeat", post(routes::devices::heartbeat))
        .route("/api/v1/licenses/refresh", post(routes::licenses::refresh))
        .route("/api/v1/subscriptions/manual-renewal", post(routes::subscriptions::manual_renewal))
        .route("/api/v1/updates/check", get(routes::updates::check))
        .route("/api/v1/updates/publish", post(routes::updates::publish))
        .route("/api/v1/backups/metadata", post(routes::backups::upload_metadata))
        .route("/api/v1/admin/merchants", get(routes::admin::merchants::list))
        .route("/api/v1/admin/merchants/:id", get(routes::admin::merchants::get))
        .route("/api/v1/admin/devices/:id/revoke", post(routes::admin::devices::revoke))
        .route("/api/v1/admin/audit-logs", get(routes::admin::audit::list))
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1MB limit
        .layer(cors)
        .with_state(pool);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Control Plane API Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
