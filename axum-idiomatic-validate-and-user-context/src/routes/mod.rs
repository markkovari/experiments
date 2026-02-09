mod posts;
mod users;

use axum::Router;

use crate::AppState;

pub fn app_router(state: AppState) -> Router {
    Router::new()
        .nest("/users", users::router())
        .nest("/posts", posts::router())
        .with_state(state)
}
