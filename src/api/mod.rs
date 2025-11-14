use axum::Router;

pub mod auth;
pub mod middleware;
pub mod periods;
pub mod schools;
pub mod users;

use crate::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .nest("/auth", auth::routes(state.clone()))
        .nest("/schools", schools::routes(state.clone()))
        .nest("/users", users::routes(state.clone()))
        .nest("/periods", periods::routes(state.clone()))
        .with_state(state)
}
