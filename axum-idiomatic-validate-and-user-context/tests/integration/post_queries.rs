use axum_idiomatic_validate_and_user_context::models::post::{
    create_post, delete_post, get_post, list_posts, update_post,
};

use crate::common;

#[tokio::test]
async fn create_and_list_posts() {
    let pool = common::test_pool().await;
    let (user_id, _) = common::insert_test_user(&pool, "poster").await;

    create_post(&pool, user_id, "First", "Content 1")
        .await
        .unwrap();
    create_post(&pool, user_id, "Second", "Content 2")
        .await
        .unwrap();

    let posts = list_posts(&pool, user_id).await.unwrap();
    assert_eq!(posts.len(), 2);
}

#[tokio::test]
async fn get_post_by_id() {
    let pool = common::test_pool().await;
    let (user_id, _) = common::insert_test_user(&pool, "getter").await;

    let created = create_post(&pool, user_id, "My Post", "Body")
        .await
        .unwrap();

    let found = get_post(&pool, created.id, user_id).await.unwrap().unwrap();
    assert_eq!(found.title, "My Post");
}

#[tokio::test]
async fn get_post_wrong_user_returns_none() {
    let pool = common::test_pool().await;
    let (user_a, _) = common::insert_test_user(&pool, "user_a").await;
    let (user_b, _) = common::insert_test_user(&pool, "user_b").await;

    let post = create_post(&pool, user_a, "Private", "Secret")
        .await
        .unwrap();

    let found = get_post(&pool, post.id, user_b).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn update_post_title() {
    let pool = common::test_pool().await;
    let (user_id, _) = common::insert_test_user(&pool, "updater").await;

    let post = create_post(&pool, user_id, "Old Title", "Content")
        .await
        .unwrap();

    let updated = update_post(&pool, post.id, user_id, Some("New Title"), None)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.title, "New Title");
    assert_eq!(updated.content, "Content");
}

#[tokio::test]
async fn delete_post_succeeds() {
    let pool = common::test_pool().await;
    let (user_id, _) = common::insert_test_user(&pool, "deleter").await;

    let post = create_post(&pool, user_id, "To Delete", "Bye")
        .await
        .unwrap();

    let deleted = delete_post(&pool, post.id, user_id).await.unwrap();
    assert!(deleted);

    let gone = get_post(&pool, post.id, user_id).await.unwrap();
    assert!(gone.is_none());
}

#[tokio::test]
async fn delete_nonexistent_returns_false() {
    let pool = common::test_pool().await;
    let (user_id, _) = common::insert_test_user(&pool, "deleter2").await;

    let deleted = delete_post(&pool, uuid::Uuid::new_v4(), user_id)
        .await
        .unwrap();
    assert!(!deleted);
}
