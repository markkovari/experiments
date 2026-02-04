use axum::{
    routing::{delete, get, post, put},
    Router,
};

use super::handlers::{
    delete_user_handler, get_all_users_handler, get_profile_handler, get_user_by_id_handler,
    login_handler, register_handler, update_user_handler,
};
use crate::AppState;

pub fn create_user_routes() -> Router<AppState> {
    Router::new()
        // Public routes
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        // Protected routes
        .route("/profile", get(get_profile_handler))
        .route("/", get(get_all_users_handler))
        .route("/:id", get(get_user_by_id_handler))
        .route("/:id", put(update_user_handler))
        .route("/:id", delete(delete_user_handler))
}
