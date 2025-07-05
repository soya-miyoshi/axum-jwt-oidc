//! Axum middleware for OIDC JWT token validation and claims extraction. This middleware integrates with the `async-oidc-jwt-validator` crate to provide seamless JWT validation in your Axum applications.
//!
//! # Features
//!
//! - Easy integration with Axum applications
//! - Automatic JWT token extraction from Authorization header
//! - Custom claims support with type-safe deserialization
//! - Token validation using OIDC provider discovery
//! - Claims are injected into request extensions for easy access
//!
//! # Usage
//!
//! ```rust,no_run
//! use axum::{Router, routing::get, Extension};
//! use axum_jwt_oidc::OidcAuthLayer;
//! use async_oidc_jwt_validator::{OidcConfig, OidcValidator, Validation};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Deserialize, Serialize)]
//! struct CustomClaims {
//!     sub: String,
//!     email: Option<String>,
//!     // Add your custom claims here
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     // Initialize OIDC validator
//!     let config = OidcConfig::new(
//!         "https://your-oidc-provider.com".to_string(),
//!         "your-client-id".to_string(),
//!         "https://your-oidc-provider.com/.well-known/jwks.json".to_string(),
//!     );
//!     let oidc_validator = OidcValidator::new(config);
//!     
//!     // Configure validation rules
//!     let validation = Validation::default();
//!     
//!     // Create the authentication layer
//!     let auth_layer = OidcAuthLayer::<CustomClaims>::new(oidc_validator, validation);
//!     
//!     // Build your router with the middleware
//!     let app = Router::new()
//!         .route("/protected", get(protected_handler))
//!         .layer(auth_layer);
//!     
//!     // Run your server
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
//!         .await
//!         .unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//!
//! async fn protected_handler(
//!     Extension(claims): Extension<CustomClaims>,
//! ) -> &'static str {
//!     // Access validated claims here
//!     println!("User ID: {}", claims.sub);
//!     "Protected content"
//! }
//! ```
//!
//! # How it works
//!
//! 1. The middleware extracts the JWT token from the `Authorization` header (Bearer token)
//! 2. Validates the token using the configured OIDC validator
//! 3. Deserializes the claims into your custom type
//! 4. Injects the claims into the request extensions
//! 5. Continues to the next handler if validation succeeds
//!
//! If validation fails, the request continues without claims in the extensions. You can implement your own authorization logic based on the presence or absence of claims.
//!
//! # Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! axum-jwt-oidc = "0.1"
//! ```

mod auth;
mod layer;
mod middleware;

// Re-export the public API
pub use layer::OidcAuthLayer;
