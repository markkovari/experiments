use axum_test_it_is_whay_it_is::modules::{
    posts::{
        domain::{CreatePostInput, UpdatePostInput},
        repository::create_post_repository,
    },
    users::{
        domain::CreateUserInput,
        repository::{UserRepository, UserRepositoryTrait},
    },
};
use sqlx::PgPool;
use uuid::Uuid;

async fn create_test_user(pool: &PgPool) -> Uuid {
    let user_repo = UserRepository::new(pool.clone());
    let input = CreateUserInput {
        email: format!("author_{}@example.com", Uuid::new_v4()),
        name: "Test Author".to_string(),
        password: "password123".to_string(),
    };
    user_repo.create(input).await.unwrap().id
}

#[tokio::test]
async fn test_create_post() {
    let (_container, pool) = super::common::setup_test_db().await;
    let author_id = create_test_user(&pool).await;
    let repo: axum_test_it_is_whay_it_is::modules::posts::repository::PostRepository = create_post_repository(pool.clone());

    let input = CreatePostInput {
        title: "Test Post".to_string(),
        content: "This is test content".to_string(),
        published: Some(false),
    };

    let post = repo.create(input, author_id).await.expect("Failed to create post");

    assert_eq!(post.title, "Test Post");
    assert_eq!(post.content, "This is test content");
    assert_eq!(post.published, false);
    assert_eq!(post.author_id, author_id);

    pool.close().await;
}

#[tokio::test]
async fn test_find_post_by_id() {
    let (_container, pool) = super::common::setup_test_db().await;
    let author_id = create_test_user(&pool).await;
    let repo = create_post_repository(pool.clone());

    let input = CreatePostInput {
        title: "Test Post".to_string(),
        content: "Content".to_string(),
        published: Some(true),
    };

    let created_post = repo.create(input, author_id).await.unwrap();
    let found_post = repo.find_by_id(created_post.id).await.unwrap();

    assert!(found_post.is_some());
    let found_post = found_post.unwrap();
    assert_eq!(found_post.id, created_post.id);
    assert_eq!(found_post.title, created_post.title);

    pool.close().await;
}

#[tokio::test]
async fn test_find_published_posts() {
    let (_container, pool) = super::common::setup_test_db().await;
    let author_id = create_test_user(&pool).await;
    let repo = create_post_repository(pool.clone());

    // Create published post
    let input1 = CreatePostInput {
        title: "Published Post".to_string(),
        content: "Content".to_string(),
        published: Some(true),
    };
    repo.create(input1, author_id).await.unwrap();

    // Create unpublished post
    let input2 = CreatePostInput {
        title: "Draft Post".to_string(),
        content: "Content".to_string(),
        published: Some(false),
    };
    repo.create(input2, author_id).await.unwrap();

    let published = repo.find_published().await.unwrap();
    assert_eq!(published.len(), 1);
    assert_eq!(published[0].title, "Published Post");

    pool.close().await;
}

#[tokio::test]
async fn test_find_posts_by_author() {
    let (_container, pool) = super::common::setup_test_db().await;
    let author1_id = create_test_user(&pool).await;
    let author2_id = create_test_user(&pool).await;
    let repo = create_post_repository(pool.clone());

    // Create posts for author 1
    for i in 1..=2 {
        let input = CreatePostInput {
            title: format!("Author 1 Post {}", i),
            content: "Content".to_string(),
            published: Some(true),
        };
        repo.create(input, author1_id).await.unwrap();
    }

    // Create post for author 2
    let input = CreatePostInput {
        title: "Author 2 Post".to_string(),
        content: "Content".to_string(),
        published: Some(true),
    };
    repo.create(input, author2_id).await.unwrap();

    let author1_posts = repo.find_by_author(author1_id).await.unwrap();
    assert_eq!(author1_posts.len(), 2);

    pool.close().await;
}

#[tokio::test]
async fn test_update_post() {
    let (_container, pool) = super::common::setup_test_db().await;
    let author_id = create_test_user(&pool).await;
    let repo = create_post_repository(pool.clone());

    let input = CreatePostInput {
        title: "Original Title".to_string(),
        content: "Original Content".to_string(),
        published: Some(false),
    };

    let created_post = repo.create(input, author_id).await.unwrap();

    let update_input = UpdatePostInput {
        title: Some("Updated Title".to_string()),
        content: None,
        published: Some(true),
    };

    let updated_post = repo.update(created_post.id, update_input).await.unwrap();

    assert!(updated_post.is_some());
    let updated_post = updated_post.unwrap();
    assert_eq!(updated_post.title, "Updated Title");
    assert_eq!(updated_post.content, "Original Content");
    assert_eq!(updated_post.published, true);

    pool.close().await;
}

#[tokio::test]
async fn test_delete_post() {
    let (_container, pool) = super::common::setup_test_db().await;
    let author_id = create_test_user(&pool).await;
    let repo = create_post_repository(pool.clone());

    let input = CreatePostInput {
        title: "To Be Deleted".to_string(),
        content: "Content".to_string(),
        published: Some(false),
    };

    let created_post = repo.create(input, author_id).await.unwrap();
    let deleted = repo.delete(created_post.id).await.unwrap();

    assert!(deleted);

    let found = repo.find_by_id(created_post.id).await.unwrap();
    assert!(found.is_none());

    pool.close().await;
}

#[tokio::test]
async fn test_find_all_posts() {
    let (_container, pool) = super::common::setup_test_db().await;
    let author_id = create_test_user(&pool).await;
    let repo = create_post_repository(pool.clone());

    // Create multiple posts
    for i in 1..=3 {
        let input = CreatePostInput {
            title: format!("Post {}", i),
            content: "Content".to_string(),
            published: Some(i % 2 == 0), // Alternate published status
        };
        repo.create(input, author_id).await.unwrap();
    }

    let posts = repo.find_all().await.unwrap();
    assert_eq!(posts.len(), 3);

    pool.close().await;
}

#[tokio::test]
async fn test_post_cascade_delete_with_user() {
    let (_container, pool) = super::common::setup_test_db().await;
    let user_repo = UserRepository::new(pool.clone());
    let post_repo: axum_test_it_is_whay_it_is::modules::posts::repository::PostRepository = create_post_repository(pool.clone());

    // Create user
    let input = CreateUserInput {
        email: "author@example.com".to_string(),
        name: "Author".to_string(),
        password: "password123".to_string(),
    };
    let user = user_repo.create(input).await.unwrap();

    // Create post
    let post_input = CreatePostInput {
        title: "Post".to_string(),
        content: "Content".to_string(),
        published: Some(true),
    };
    let post = post_repo.create(post_input, user.id).await.unwrap();

    // Delete user (should cascade delete posts)
    user_repo.delete(user.id).await.unwrap();

    // Verify post is deleted
    let found_post = post_repo.find_by_id(post.id).await.unwrap();
    assert!(found_post.is_none());

    pool.close().await;
}
