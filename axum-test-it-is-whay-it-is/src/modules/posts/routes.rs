use axum::{
    routing::{delete, get, post, put},
    Router,
};

use super::handlers::{
    create_post_handler, delete_post_handler, get_all_posts_handler, get_post_by_id_handler,
    get_posts_by_author_handler, get_published_posts_handler, update_post_handler,
};
use crate::AppState;

pub fn create_post_routes() -> Router<AppState> {
    Router::new()
        // Public routes
        .route("/", get(get_all_posts_handler))
        .route("/published", get(get_published_posts_handler))
        .route("/:id", get(get_post_by_id_handler))
        .route("/author/:author_id", get(get_posts_by_author_handler))
        // Protected routes
        .route("/", post(create_post_handler))
        .route("/:id", put(update_post_handler))
        .route("/:id", delete(delete_post_handler))
}
