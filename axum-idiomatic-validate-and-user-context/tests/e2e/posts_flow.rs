use crate::common;

#[tokio::test]
async fn full_posts_crud_flow() {
    let (url, _pool) = common::spawn_app().await;
    let client = reqwest::Client::new();

    // Register and get token
    let res = client
        .post(format!("{url}/users/register"))
        .json(&serde_json::json!({
            "username": "cruder",
            "email": "cruder@test.com",
            "password": "pass123"
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = res.json().await.unwrap();
    let token = body["token"].as_str().unwrap().to_string();

    // Create post
    let res = client
        .post(format!("{url}/posts"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({
            "title": "My Post",
            "content": "Hello world"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 201);
    let post: serde_json::Value = res.json().await.unwrap();
    let post_id = post["id"].as_str().unwrap();
    assert_eq!(post["title"], "My Post");

    // List posts
    let res = client
        .get(format!("{url}/posts"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let posts: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(posts.len(), 1);

    // Get by ID
    let res = client
        .get(format!("{url}/posts/{post_id}"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let fetched: serde_json::Value = res.json().await.unwrap();
    assert_eq!(fetched["title"], "My Post");

    // Update
    let res = client
        .put(format!("{url}/posts/{post_id}"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({
            "title": "Updated Post"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let updated: serde_json::Value = res.json().await.unwrap();
    assert_eq!(updated["title"], "Updated Post");
    assert_eq!(updated["content"], "Hello world");

    // Delete
    let res = client
        .delete(format!("{url}/posts/{post_id}"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 204);

    // Get deleted → 404
    let res = client
        .get(format!("{url}/posts/{post_id}"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn cross_user_isolation() {
    let (url, _pool) = common::spawn_app().await;
    let client = reqwest::Client::new();

    // Register user A
    let res = client
        .post(format!("{url}/users/register"))
        .json(&serde_json::json!({
            "username": "user_a_e2e",
            "email": "a_e2e@test.com",
            "password": "pass"
        }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let token_a = body["token"].as_str().unwrap().to_string();

    // Register user B
    let res = client
        .post(format!("{url}/users/register"))
        .json(&serde_json::json!({
            "username": "user_b_e2e",
            "email": "b_e2e@test.com",
            "password": "pass"
        }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let token_b = body["token"].as_str().unwrap().to_string();

    // User A creates a post
    let res = client
        .post(format!("{url}/posts"))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&serde_json::json!({
            "title": "A's Post",
            "content": "Private to A"
        }))
        .send()
        .await
        .unwrap();
    let post: serde_json::Value = res.json().await.unwrap();
    let post_id = post["id"].as_str().unwrap();

    // User B cannot see A's post
    let res = client
        .get(format!("{url}/posts/{post_id}"))
        .header("Authorization", format!("Bearer {token_b}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 404);

    // User B's list is empty
    let res = client
        .get(format!("{url}/posts"))
        .header("Authorization", format!("Bearer {token_b}"))
        .send()
        .await
        .unwrap();

    let posts: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(posts.is_empty());
}

#[tokio::test]
async fn create_post_with_empty_fields_returns_400() {
    let (url, _pool) = common::spawn_app().await;
    let client = reqwest::Client::new();

    // Register and get token
    let res = client
        .post(format!("{url}/users/register"))
        .json(&serde_json::json!({
            "username": "validator_test",
            "email": "validator@test.com",
            "password": "pass123"
        }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let token = body["token"].as_str().unwrap().to_string();

    let res = client
        .post(format!("{url}/posts"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({
            "title": "",
            "content": ""
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("must not be empty"));
}

#[tokio::test]
async fn admin_can_read_all_posts() {
    let (url, pool) = common::spawn_app().await;
    let client = reqwest::Client::new();

    // Create a regular user with a post
    let (_, user_token) = common::insert_test_user(&pool, "regular_user").await;

    let res = client
        .post(format!("{url}/posts"))
        .header("Authorization", format!("Bearer {user_token}"))
        .json(&serde_json::json!({
            "title": "User's Post",
            "content": "Private content"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let post: serde_json::Value = res.json().await.unwrap();
    let post_id = post["id"].as_str().unwrap();

    // Create an admin
    let (_, admin_token) = common::insert_test_admin(&pool, "admin_reader").await;

    // Admin can list all posts
    let res = client
        .get(format!("{url}/posts"))
        .header("Authorization", format!("Bearer {admin_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let posts: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0]["title"], "User's Post");

    // Admin can get specific post by ID
    let res = client
        .get(format!("{url}/posts/{post_id}"))
        .header("Authorization", format!("Bearer {admin_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let fetched: serde_json::Value = res.json().await.unwrap();
    assert_eq!(fetched["title"], "User's Post");
}

#[tokio::test]
async fn admin_can_update_any_post() {
    let (url, pool) = common::spawn_app().await;
    let client = reqwest::Client::new();

    // Create a regular user with a post
    let (_, user_token) = common::insert_test_user(&pool, "owner_user").await;

    let res = client
        .post(format!("{url}/posts"))
        .header("Authorization", format!("Bearer {user_token}"))
        .json(&serde_json::json!({
            "title": "Original Title",
            "content": "Original content"
        }))
        .send()
        .await
        .unwrap();
    let post: serde_json::Value = res.json().await.unwrap();
    let post_id = post["id"].as_str().unwrap();

    // Create an admin
    let (_, admin_token) = common::insert_test_admin(&pool, "admin_updater").await;

    // Admin updates the post
    let res = client
        .put(format!("{url}/posts/{post_id}"))
        .header("Authorization", format!("Bearer {admin_token}"))
        .json(&serde_json::json!({
            "title": "Admin Updated"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let updated: serde_json::Value = res.json().await.unwrap();
    assert_eq!(updated["title"], "Admin Updated");
    assert_eq!(updated["content"], "Original content");
}

#[tokio::test]
async fn admin_can_delete_any_post() {
    let (url, pool) = common::spawn_app().await;
    let client = reqwest::Client::new();

    // Create a regular user with a post
    let (_, user_token) = common::insert_test_user(&pool, "delete_target").await;

    let res = client
        .post(format!("{url}/posts"))
        .header("Authorization", format!("Bearer {user_token}"))
        .json(&serde_json::json!({
            "title": "To Be Deleted",
            "content": "Will be removed by admin"
        }))
        .send()
        .await
        .unwrap();
    let post: serde_json::Value = res.json().await.unwrap();
    let post_id = post["id"].as_str().unwrap();

    // Create an admin
    let (_, admin_token) = common::insert_test_admin(&pool, "admin_deleter").await;

    // Admin deletes the post
    let res = client
        .delete(format!("{url}/posts/{post_id}"))
        .header("Authorization", format!("Bearer {admin_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // Verify it's gone
    let res = client
        .get(format!("{url}/posts/{post_id}"))
        .header("Authorization", format!("Bearer {admin_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}
