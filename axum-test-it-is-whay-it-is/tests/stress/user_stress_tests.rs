use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum_test_it_is_whay_it_is::test_helpers::create_test_app;
use serde_json::json;
use tower::util::ServiceExt;

macro_rules! generate_user_registration_test {
    ($name:ident, $email:expr, $username:expr) => {
        #[tokio::test]
        async fn $name() {
            let ctx = crate::common::setup_test_db().await;
            let app = create_test_app(ctx.pool.clone()).await;

            let register_body = json!({
                "email": $email,
                "name": $username,
                "password": "password123"
            });

            let request = Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body).unwrap()))
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::CREATED);

            ctx.cleanup().await;
        }
    };
}

macro_rules! generate_login_test {
    ($name:ident, $email:expr, $username:expr) => {
        #[tokio::test]
        async fn $name() {
            let ctx = crate::common::setup_test_db().await;
            let app = create_test_app(ctx.pool.clone()).await;

            // Register first
            let register_body = json!({
                "email": $email,
                "name": $username,
                "password": "password123"
            });

            let request = Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body).unwrap()))
                .unwrap();

            app.clone().oneshot(request).await.unwrap();

            // Login
            let login_body = json!({
                "email": $email,
                "password": "password123"
            });

            let request = Request::builder()
                .uri("/api/users/login")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            ctx.cleanup().await;
        }
    };
}

macro_rules! generate_profile_test {
    ($name:ident, $email:expr, $username:expr) => {
        #[tokio::test]
        async fn $name() {
            let ctx = crate::common::setup_test_db().await;
            let app = create_test_app(ctx.pool.clone()).await;

            // Register
            let register_body = json!({
                "email": $email,
                "name": $username,
                "password": "password123"
            });

            let request = Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body).unwrap()))
                .unwrap();

            app.clone().oneshot(request).await.unwrap();

            // Login
            let login_body = json!({
                "email": $email,
                "password": "password123"
            });

            let request = Request::builder()
                .uri("/api/users/login")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
            let token = body["data"]["token"].as_str().unwrap();

            // Get profile
            let request = Request::builder()
                .uri("/api/users/profile")
                .method("GET")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(body["data"]["email"], $email);

            ctx.cleanup().await;
        }
    };
}

macro_rules! generate_health_test {
    ($name:ident) => {
        #[tokio::test]
        async fn $name() {
            let ctx = crate::common::setup_test_db().await;
            let app = create_test_app(ctx.pool.clone()).await;

            let request = Request::builder()
                .uri("/health")
                .method("GET")
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            ctx.cleanup().await;
        }
    };
}

// Generate 24 registration tests
generate_user_registration_test!(stress_register_user_01, "user01@stress.test", "Stress User 01");
generate_user_registration_test!(stress_register_user_02, "user02@stress.test", "Stress User 02");
generate_user_registration_test!(stress_register_user_03, "user03@stress.test", "Stress User 03");
generate_user_registration_test!(stress_register_user_04, "user04@stress.test", "Stress User 04");
generate_user_registration_test!(stress_register_user_05, "user05@stress.test", "Stress User 05");
generate_user_registration_test!(stress_register_user_06, "user06@stress.test", "Stress User 06");
generate_user_registration_test!(stress_register_user_07, "user07@stress.test", "Stress User 07");
generate_user_registration_test!(stress_register_user_08, "user08@stress.test", "Stress User 08");
generate_user_registration_test!(stress_register_user_09, "user09@stress.test", "Stress User 09");
generate_user_registration_test!(stress_register_user_10, "user10@stress.test", "Stress User 10");
generate_user_registration_test!(stress_register_user_11, "user11@stress.test", "Stress User 11");
generate_user_registration_test!(stress_register_user_12, "user12@stress.test", "Stress User 12");
generate_user_registration_test!(stress_register_user_13, "user13@stress.test", "Stress User 13");
generate_user_registration_test!(stress_register_user_14, "user14@stress.test", "Stress User 14");
generate_user_registration_test!(stress_register_user_15, "user15@stress.test", "Stress User 15");
generate_user_registration_test!(stress_register_user_16, "user16@stress.test", "Stress User 16");
generate_user_registration_test!(stress_register_user_17, "user17@stress.test", "Stress User 17");
generate_user_registration_test!(stress_register_user_18, "user18@stress.test", "Stress User 18");
generate_user_registration_test!(stress_register_user_19, "user19@stress.test", "Stress User 19");
generate_user_registration_test!(stress_register_user_20, "user20@stress.test", "Stress User 20");
generate_user_registration_test!(stress_register_user_21, "user21@stress.test", "Stress User 21");
generate_user_registration_test!(stress_register_user_22, "user22@stress.test", "Stress User 22");
generate_user_registration_test!(stress_register_user_23, "user23@stress.test", "Stress User 23");
generate_user_registration_test!(stress_register_user_24, "user24@stress.test", "Stress User 24");

// Generate 24 login tests
generate_login_test!(stress_login_user_01, "login01@stress.test", "Login User 01");
generate_login_test!(stress_login_user_02, "login02@stress.test", "Login User 02");
generate_login_test!(stress_login_user_03, "login03@stress.test", "Login User 03");
generate_login_test!(stress_login_user_04, "login04@stress.test", "Login User 04");
generate_login_test!(stress_login_user_05, "login05@stress.test", "Login User 05");
generate_login_test!(stress_login_user_06, "login06@stress.test", "Login User 06");
generate_login_test!(stress_login_user_07, "login07@stress.test", "Login User 07");
generate_login_test!(stress_login_user_08, "login08@stress.test", "Login User 08");
generate_login_test!(stress_login_user_09, "login09@stress.test", "Login User 09");
generate_login_test!(stress_login_user_10, "login10@stress.test", "Login User 10");
generate_login_test!(stress_login_user_11, "login11@stress.test", "Login User 11");
generate_login_test!(stress_login_user_12, "login12@stress.test", "Login User 12");
generate_login_test!(stress_login_user_13, "login13@stress.test", "Login User 13");
generate_login_test!(stress_login_user_14, "login14@stress.test", "Login User 14");
generate_login_test!(stress_login_user_15, "login15@stress.test", "Login User 15");
generate_login_test!(stress_login_user_16, "login16@stress.test", "Login User 16");
generate_login_test!(stress_login_user_17, "login17@stress.test", "Login User 17");
generate_login_test!(stress_login_user_18, "login18@stress.test", "Login User 18");
generate_login_test!(stress_login_user_19, "login19@stress.test", "Login User 19");
generate_login_test!(stress_login_user_20, "login20@stress.test", "Login User 20");
generate_login_test!(stress_login_user_21, "login21@stress.test", "Login User 21");
generate_login_test!(stress_login_user_22, "login22@stress.test", "Login User 22");
generate_login_test!(stress_login_user_23, "login23@stress.test", "Login User 23");
generate_login_test!(stress_login_user_24, "login24@stress.test", "Login User 24");

// Generate 24 profile tests
generate_profile_test!(stress_profile_user_01, "profile01@stress.test", "Profile User 01");
generate_profile_test!(stress_profile_user_02, "profile02@stress.test", "Profile User 02");
generate_profile_test!(stress_profile_user_03, "profile03@stress.test", "Profile User 03");
generate_profile_test!(stress_profile_user_04, "profile04@stress.test", "Profile User 04");
generate_profile_test!(stress_profile_user_05, "profile05@stress.test", "Profile User 05");
generate_profile_test!(stress_profile_user_06, "profile06@stress.test", "Profile User 06");
generate_profile_test!(stress_profile_user_07, "profile07@stress.test", "Profile User 07");
generate_profile_test!(stress_profile_user_08, "profile08@stress.test", "Profile User 08");
generate_profile_test!(stress_profile_user_09, "profile09@stress.test", "Profile User 09");
generate_profile_test!(stress_profile_user_10, "profile10@stress.test", "Profile User 10");
generate_profile_test!(stress_profile_user_11, "profile11@stress.test", "Profile User 11");
generate_profile_test!(stress_profile_user_12, "profile12@stress.test", "Profile User 12");
generate_profile_test!(stress_profile_user_13, "profile13@stress.test", "Profile User 13");
generate_profile_test!(stress_profile_user_14, "profile14@stress.test", "Profile User 14");
generate_profile_test!(stress_profile_user_15, "profile15@stress.test", "Profile User 15");
generate_profile_test!(stress_profile_user_16, "profile16@stress.test", "Profile User 16");
generate_profile_test!(stress_profile_user_17, "profile17@stress.test", "Profile User 17");
generate_profile_test!(stress_profile_user_18, "profile18@stress.test", "Profile User 18");
generate_profile_test!(stress_profile_user_19, "profile19@stress.test", "Profile User 19");
generate_profile_test!(stress_profile_user_20, "profile20@stress.test", "Profile User 20");
generate_profile_test!(stress_profile_user_21, "profile21@stress.test", "Profile User 21");
generate_profile_test!(stress_profile_user_22, "profile22@stress.test", "Profile User 22");
generate_profile_test!(stress_profile_user_23, "profile23@stress.test", "Profile User 23");
generate_profile_test!(stress_profile_user_24, "profile24@stress.test", "Profile User 24");

// Generate 24 health check tests
generate_health_test!(stress_health_01);
generate_health_test!(stress_health_02);
generate_health_test!(stress_health_03);
generate_health_test!(stress_health_04);
generate_health_test!(stress_health_05);
generate_health_test!(stress_health_06);
generate_health_test!(stress_health_07);
generate_health_test!(stress_health_08);
generate_health_test!(stress_health_09);
generate_health_test!(stress_health_10);
generate_health_test!(stress_health_11);
generate_health_test!(stress_health_12);
generate_health_test!(stress_health_13);
generate_health_test!(stress_health_14);
generate_health_test!(stress_health_15);
generate_health_test!(stress_health_16);
generate_health_test!(stress_health_17);
generate_health_test!(stress_health_18);
generate_health_test!(stress_health_19);
generate_health_test!(stress_health_20);
generate_health_test!(stress_health_21);
generate_health_test!(stress_health_22);
generate_health_test!(stress_health_23);
generate_health_test!(stress_health_24);
