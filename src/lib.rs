pub mod api;
pub mod config;
pub mod dto;
pub mod integrations;
pub mod models;
pub mod repositories;
pub mod services;
pub mod utils;

pub use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Config,
}
