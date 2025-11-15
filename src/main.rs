use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};

use ppdb_backend::{api, AppState, Config};
use ppdb_backend::api::docs::ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ppdb_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded successfully");

    // Database connection pool with aggressive prepared statement fixes
    // This fixes "prepared statement already exists" error
    let db_pool = PgPoolOptions::new()
        .max_connections(1)  // Single connection to avoid statement conflicts
        .max_lifetime(std::time::Duration::from_secs(30))  // Short connection lifetime
        .idle_timeout(std::time::Duration::from_secs(10))  // Quick connection release
        .connect_with(
            config.database_url
                .parse::<sqlx::postgres::PgConnectOptions>()?
                .statement_cache_capacity(0)  // Disable statement caching
        )
        .await?;
    tracing::info!("Database connection pool established (single connection, no cache)");

    // Run migrations
    // Note: Temporarily disabled due to prepared statement conflicts
    // Run manually: sqlx migrate run
    // Or use alternative migration method
    // sqlx::migrate!("./migrations").run(&db_pool).await?;
    // tracing::info!("Database migrations completed");

    // Build application state
    let app_state = AppState {
        db: db_pool,
        config: config.clone(),
    };

    // Build router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .nest("/api/v1", api::routes(app_state.clone()))
        // API Documentation routes
        .merge(SwaggerUi::new("/api/docs/swagger")
            .url("/api/docs/openapi.json", ApiDoc::openapi()))
        .merge(RapiDoc::new("/api/docs/openapi.json")
            .path("/api/docs/rapidoc"))
        .merge(Redoc::with_url("/api/docs/redoc", ApiDoc::openapi()))
        .layer(CorsLayer::permissive());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🚀 Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "PPDB Backend API - v0.1.0"
}

async fn health_check() -> &'static str {
    "OK"
}
