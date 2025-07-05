use async_oidc_jwt_validator::{OidcConfig, OidcValidator, Validation};
use axum::{body::Body, http::Request, routing::get, Extension, Router};
use axum_jwt_oidc::OidcAuthLayer;
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TestClaims {
    sub: String,
    exp: i64,
}

#[tokio::test]
async fn test_middleware_without_token() {
    // This test verifies that requests without tokens pass through
    // but don't have claims in extensions

    // Mock OIDC validator (this will fail for real tokens)
    let config = OidcConfig::new(
        "https://example.com".to_string(),
        "test-client-id".to_string(),
        "https://example.com/.well-known/jwks.json".to_string(),
    );
    let oidc_validator = OidcValidator::new(config);

    let validation = Validation::default();
    let auth_layer = OidcAuthLayer::<TestClaims>::new(oidc_validator, validation);

    let app = Router::new().route("/test", get(handler)).layer(auth_layer);

    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Check that the response body indicates no authentication
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert_eq!(body_str, "Not authenticated");
}

async fn handler(claims: Option<Extension<TestClaims>>) -> &'static str {
    if claims.is_some() {
        "Authenticated"
    } else {
        "Not authenticated"
    }
}

#[tokio::test]
async fn test_middleware_with_invalid_token() {
    // This test verifies that requests with invalid tokens also pass through
    // but don't have claims in extensions

    let config = OidcConfig::new(
        "https://example.com".to_string(),
        "test-client-id".to_string(),
        "https://example.com/.well-known/jwks.json".to_string(),
    );
    let oidc_validator = OidcValidator::new(config);

    let validation = Validation::default();
    let auth_layer = OidcAuthLayer::<TestClaims>::new(oidc_validator, validation);

    let app = Router::new().route("/test", get(handler)).layer(auth_layer);

    // Send request with an invalid JWT token
    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("Authorization", "Bearer invalid.jwt.token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Check that the response body indicates no authentication
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert_eq!(body_str, "Not authenticated");
}
