use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    routing::Router,
};
use http_body_util::BodyExt;
use sqlx::{sqlite::SqlitePoolOptions};

use rust_web_demo::{models, routes};

#[tokio::test]
async fn list_users_returns_empty_array_initially() {
    let context = TestContext::new().await;

    let response = context
        .request(
            Request::builder()
                .uri("/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = body_bytes(response).await;
    let body: Vec<models::user::User> = serde_json::from_slice(&bytes).unwrap();
    assert!(body.is_empty());
}

#[tokio::test]
async fn create_and_get_user() {
    let context = TestContext::new().await;
    let payload = serde_json::json!({
        "name": "Ada Lovelace",
        "email": "ada@example.com"
    });

    let response = context
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri("/users")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = body_bytes(response).await;
    let user: models::user::User = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(user.name, "Ada Lovelace");
    assert_eq!(user.email, "ada@example.com");

    let response = context
        .request(
            Request::builder()
                .uri(format!("/users/{}", user.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body_bytes(response).await;
    let fetched: models::user::User = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(fetched.id, user.id);
    assert_eq!(fetched.name, "Ada Lovelace");
    assert_eq!(fetched.email, "ada@example.com");
}

#[tokio::test]
async fn update_user_modifies_fields() {
    let context = TestContext::new().await;
    let initial = context
        .create_user("Grace Hopper", "grace@example.com")
        .await;

    let payload = serde_json::json!({
        "name": "Grace B. Hopper",
        "email": "grace.hopper@example.com"
    });

    let response = context
        .request(
            Request::builder()
                .method(http::Method::PUT)
                .uri(format!("/users/{}", initial.id))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body_bytes(response).await;
    let updated: models::user::User = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(updated.id, initial.id);
    assert_eq!(updated.name, "Grace B. Hopper");
    assert_eq!(updated.email, "grace.hopper@example.com");
}

#[tokio::test]
async fn delete_user_removes_row() {
    let context = TestContext::new().await;
    let created = context.create_user("Alan Turing", "alan@example.com").await;

    let response = context
        .request(
            Request::builder()
                .method(http::Method::DELETE)
                .uri(format!("/users/{}", created.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = context
        .request(
            Request::builder()
                .uri(format!("/users/{}", created.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_user_with_invalid_email_returns_validation_error() {
    let context = TestContext::new().await;
    let payload = serde_json::json!({
        "name": "Test User",
        "email": "invalid-email"
    });

    let response = context
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri("/users")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let bytes = body_bytes(response).await;
    let error_response: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(error_response["message"], "Datos de entrada inválidos");
    assert!(error_response["errors"].is_array());
}

#[tokio::test]
async fn create_user_with_empty_name_returns_validation_error() {
    let context = TestContext::new().await;
    let payload = serde_json::json!({
        "name": "",
        "email": "test@example.com"
    });

    let response = context
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri("/users")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_user_with_long_name_returns_validation_error() {
    let context = TestContext::new().await;
    let long_name = "a".repeat(101); // Más de 100 caracteres
    let payload = serde_json::json!({
        "name": long_name,
        "email": "test@example.com"
    });

    let response = context
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri("/users")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn get_nonexistent_user_returns_not_found() {
    let context = TestContext::new().await;
    let fake_id = uuid::Uuid::new_v4();

    let response = context
        .request(
            Request::builder()
                .uri(format!("/users/{}", fake_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let bytes = body_bytes(response).await;
    let error_response: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(error_response["message"], "Recurso no encontrado");
}

#[tokio::test]
async fn update_nonexistent_user_returns_not_found() {
    let context = TestContext::new().await;
    let fake_id = uuid::Uuid::new_v4();
    let payload = serde_json::json!({
        "name": "Updated Name"
    });

    let response = context
        .request(
            Request::builder()
                .method(http::Method::PUT)
                .uri(format!("/users/{}", fake_id))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_nonexistent_user_returns_not_found() {
    let context = TestContext::new().await;
    let fake_id = uuid::Uuid::new_v4();

    let response = context
        .request(
            Request::builder()
                .method(http::Method::DELETE)
                .uri(format!("/users/{}", fake_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_user_with_empty_payload_returns_validation_error() {
    let context = TestContext::new().await;
    let user = context.create_user("Test User", "test@example.com").await;
    let payload = serde_json::json!({});

    let response = context
        .request(
            Request::builder()
                .method(http::Method::PUT)
                .uri(format!("/users/{}", user.id))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn update_user_partially_updates_only_provided_fields() {
    let context = TestContext::new().await;
    let user = context.create_user("Original Name", "original@example.com").await;

    // Solo actualizar el nombre
    let payload = serde_json::json!({
        "name": "Updated Name"
    });

    let response = context
        .request(
            Request::builder()
                .method(http::Method::PUT)
                .uri(format!("/users/{}", user.id))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body_bytes(response).await;
    let updated: models::user::User = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.email, "original@example.com"); // No debería cambiar
    assert_eq!(updated.id, user.id);
    assert_eq!(updated.created_at, user.created_at); // No debería cambiar
}

#[tokio::test]
async fn update_user_with_invalid_email_returns_validation_error() {
    let context = TestContext::new().await;
    let user = context.create_user("Test User", "test@example.com").await;
    let payload = serde_json::json!({
        "email": "invalid-email"
    });

    let response = context
        .request(
            Request::builder()
                .method(http::Method::PUT)
                .uri(format!("/users/{}", user.id))
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_user_sets_created_at_timestamp() {
    let context = TestContext::new().await;
    let before_creation = chrono::Utc::now();
    
    let user = context.create_user("Test User", "test@example.com").await;
    
    let after_creation = chrono::Utc::now();
    
    assert!(user.created_at >= before_creation);
    assert!(user.created_at <= after_creation);
}

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let context = TestContext::new().await;

    let response = context
        .request(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body_bytes(response).await;
    let body = String::from_utf8(bytes).unwrap();
    assert_eq!(body, "OK");
}

#[tokio::test]
async fn root_endpoint_returns_welcome_message() {
    let context = TestContext::new().await;

    let response = context
        .request(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body_bytes(response).await;
    let body = String::from_utf8(bytes).unwrap();
    assert_eq!(body, "Bienvenido a la API en Rust 🚀");
}

#[tokio::test]
async fn create_user_with_whitespace_only_name_returns_validation_error() {
    let context = TestContext::new().await;
    let payload = serde_json::json!({
        "name": "   ",
        "email": "test@example.com"
    });

    let response = context.post_json("/users", payload).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_user_with_whitespace_only_email_returns_validation_error() {
    let context = TestContext::new().await;
    let payload = serde_json::json!({
        "name": "Test User",
        "email": "   "
    });

    let response = context.post_json("/users", payload).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_user_with_mixed_case_email_normalizes_to_lowercase() {
    let context = TestContext::new().await;
    let payload = serde_json::json!({
        "name": "Test User",
        "email": "TEST@EXAMPLE.COM"
    });

    let response = context.post_json("/users", payload).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let bytes = body_bytes(response).await;
    let user: models::user::User = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(user.email, "test@example.com");
}

#[tokio::test]
async fn create_user_with_whitespace_around_fields_trims_whitespace() {
    let context = TestContext::new().await;
    let payload = serde_json::json!({
        "name": "  Test User  ",
        "email": "  test@example.com  "
    });

    let response = context.post_json("/users", payload).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let bytes = body_bytes(response).await;
    let user: models::user::User = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(user.name, "Test User");
    assert_eq!(user.email, "test@example.com");
}

#[tokio::test]
async fn update_user_with_whitespace_only_fields_ignores_them() {
    let context = TestContext::new().await;
    let user = context.create_user("Original Name", "original@example.com").await;
    
    let payload = serde_json::json!({
        "name": "   ",
        "email": "   "
    });

    let response = context.put_json(&format!("/users/{}", user.id), payload).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn list_users_returns_users_in_creation_order() {
    let context = TestContext::new().await;
    
    let user1 = context.create_user("First User", "first@example.com").await;
    let user2 = context.create_user("Second User", "second@example.com").await;
    
    let response = context.get("/users").await;
    assert_eq!(response.status(), StatusCode::OK);
    
    let bytes = body_bytes(response).await;
    let users: Vec<models::user::User> = serde_json::from_slice(&bytes).unwrap();
    
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].id, user1.id);
    assert_eq!(users[1].id, user2.id);
}

#[tokio::test]
async fn create_user_with_maximum_valid_name_length_succeeds() {
    let context = TestContext::new().await;
    let max_name = "a".repeat(100); // Exactamente 100 caracteres
    let payload = serde_json::json!({
        "name": max_name,
        "email": "test@example.com"
    });

    let response = context.post_json("/users", payload).await;
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn create_user_with_various_valid_email_formats_succeeds() {
    let context = TestContext::new().await;
    let valid_emails = vec![
        "user@example.com",
        "user.name@example.com",
        "user+tag@example.co.uk",
        "user123@example-domain.org",
    ];

    for email in valid_emails {
        let payload = serde_json::json!({
            "name": "Test User",
            "email": email
        });

        let response = context.post_json("/users", payload).await;
        assert_eq!(response.status(), StatusCode::CREATED, "Failed for email: {}", email);
    }
}

#[tokio::test]
async fn create_user_with_invalid_email_formats_returns_validation_error() {
    let context = TestContext::new().await;
    let invalid_emails = vec![
        "invalid-email",
        "@example.com",
        "user@",
        "user@.com",
        "user.example.com",
        "user@example",
        "user@@example.com",
    ];

    for email in invalid_emails {
        let payload = serde_json::json!({
            "name": "Test User",
            "email": email
        });

        let response = context.post_json("/users", payload).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY, "Should fail for email: {}", email);
    }
}

struct TestContext {
    app: Router,
}

impl TestContext {

    async fn new() -> Self {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let app = routes::user_routes()
            .merge(routes::health_routes())
            .merge(routes::root_route())
            .with_state(pool);

        Self { app }
    }

    async fn request(&self, request: Request<Body>) -> http::Response<Body> {
        let app = self.app.clone();
        tower::ServiceExt::oneshot(app, request).await.unwrap()
    }

    async fn create_user(&self, name: &str, email: &str) -> models::user::User {
        let payload = serde_json::json!({ "name": name, "email": email });

        let response = self
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/users")
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), StatusCode::CREATED);
        let bytes = body_bytes(response).await;
        serde_json::from_slice(&bytes).unwrap()
    }

    async fn post_json(&self, uri: &str, payload: serde_json::Value) -> http::Response<Body> {
        self.request(
            Request::builder()
                .method(http::Method::POST)
                .uri(uri)
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
    }

    async fn put_json(&self, uri: &str, payload: serde_json::Value) -> http::Response<Body> {
        self.request(
            Request::builder()
                .method(http::Method::PUT)
                .uri(uri)
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
    }

    async fn get(&self, uri: &str) -> http::Response<Body> {
        self.request(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
    }

}

async fn body_bytes(response: http::Response<Body>) -> Vec<u8> {
    response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec()
}
