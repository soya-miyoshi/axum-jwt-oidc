use axum::{routing::get, Extension, Router};
use axum_jwt_oidc::{OidcAuthLayer, OidcConfig, OidcValidator, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CustomClaims {
    sub: String,
    email: Option<String>,
    name: Option<String>,
}

#[tokio::main]
async fn main() {
    // Initialize OIDC validator with your provider
    let config = OidcConfig::new(
        "https://accounts.google.com".to_string(),
        "your-client-id".to_string(),
        "https://www.googleapis.com/oauth2/v3/certs".to_string(),
    );
    let oidc_validator = OidcValidator::new(config);

    // Configure validation rules
    let validation = Validation::default();

    // Create the authentication layer
    let auth_layer = OidcAuthLayer::<CustomClaims>::new(oidc_validator, validation);

    // Build your router with the middleware
    let app = Router::new()
        .route("/", get(public_handler))
        .route("/protected", get(protected_handler))
        .layer(auth_layer);

    println!("Server starting on http://localhost:3000");

    // Run your server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn public_handler() -> &'static str {
    "This is a public endpoint"
}

async fn protected_handler(claims: Option<Extension<CustomClaims>>) -> String {
    if let Some(Extension(claims)) = claims {
        format!(
            "Hello {}! Your email is: {}",
            claims.name.as_deref().unwrap_or("Unknown"),
            claims.email.as_deref().unwrap_or("Not provided")
        )
    } else {
        "Unauthorized: No valid JWT token provided".to_string()
    }
}
