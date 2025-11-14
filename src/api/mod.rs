use axum::Router;

pub mod announcements;
pub mod auth;
pub mod middleware;
pub mod periods;
pub mod registrations;
pub mod schools;
pub mod selection;
pub mod users;
pub mod verifications;

use crate::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .nest("/auth", auth::routes(state.clone()))
        .nest("/schools", schools::routes(state.clone()))
        .nest("/users", users::routes(state.clone()))
        .nest("/periods", periods::routes(state.clone()))
        .nest("/registrations", registrations::routes(state.clone()))
        .nest("/verifications", verifications::routes(state.clone()))
        .nest("/selection", selection::routes(state.clone()))
        .nest("/announcements", announcements::routes(state.clone()))
        .with_state(state)
}
