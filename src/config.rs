use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // Server
    pub port: u16,
    pub host: String,

    // Database
    pub database_url: String,

    // Redis
    pub redis_url: String,

    // JWT
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,

    // Supabase
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub supabase_service_key: String,

    // Email (Resend)
    pub resend_api_key: String,
    pub from_email: String,

    // Payment (Midtrans)
    pub midtrans_server_key: String,
    pub midtrans_client_key: String,
    pub midtrans_is_production: bool,

    // Identity Provider (Keycloak)
    pub keycloak_url: String,
    pub keycloak_realm: String,
    pub keycloak_client_id: String,
    pub keycloak_client_secret: String,

    // CORS
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let config = Config {
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),

            database_url: std::env::var("DATABASE_URL")?,

            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),

            jwt_secret: std::env::var("JWT_SECRET")?,
            jwt_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()?,

            supabase_url: std::env::var("SUPABASE_URL")?,
            supabase_anon_key: std::env::var("SUPABASE_ANON_KEY")?,
            supabase_service_key: std::env::var("SUPABASE_SERVICE_KEY")?,

            resend_api_key: std::env::var("RESEND_API_KEY")
                .unwrap_or_else(|_| "".to_string()),
            from_email: std::env::var("FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@ppdb.com".to_string()),

            midtrans_server_key: std::env::var("MIDTRANS_SERVER_KEY")
                .unwrap_or_else(|_| "".to_string()),
            midtrans_client_key: std::env::var("MIDTRANS_CLIENT_KEY")
                .unwrap_or_else(|_| "".to_string()),
            midtrans_is_production: std::env::var("MIDTRANS_IS_PRODUCTION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),

            keycloak_url: std::env::var("KEYCLOAK_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            keycloak_realm: std::env::var("KEYCLOAK_REALM")
                .unwrap_or_else(|_| "ppdb".to_string()),
            keycloak_client_id: std::env::var("KEYCLOAK_CLIENT_ID")
                .unwrap_or_else(|_| "ppdb-backend".to_string()),
            keycloak_client_secret: std::env::var("KEYCLOAK_CLIENT_SECRET")
                .unwrap_or_else(|_| "".to_string()),

            allowed_origins: std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        };

        Ok(config)
    }
}
