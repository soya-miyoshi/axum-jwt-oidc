# axum-jwt-oidc

[![Crates.io](https://img.shields.io/crates/v/axum-jwt-oidc.svg)](https://crates.io/crates/axum-jwt-oidc)
[![Docs.rs](https://docs.rs/axum-jwt-oidc/badge.svg)](https://docs.rs/axum-jwt-oidc)
[![CI](https://github.com/soya-miyoshi/axum-jwt-oidc/workflows/CI/badge.svg)](https://github.com/soya-miyoshi/axum-jwt-oidc/actions)

Axum middleware for OIDC JWT token validation and claims extraction. This middleware integrates with the `async-oidc-jwt-validator` crate to provide seamless JWT validation in your Axum applications.

## Features

- Easy integration with Axum applications
- Automatic JWT token extraction from Authorization header
- Custom claims support with type-safe deserialization
- Token validation using OIDC provider discovery
- Claims are injected into request extensions for easy access

## Usage

```rust
use axum::{Router, routing::get, Extension};
use axum_jwt_oidc::{OidcAuthLayer, OidcConfig, OidcValidator, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CustomClaims {
    sub: String,
    email: Option<String>,
    // Add your custom claims here
}

#[tokio::main]
async fn main() {
    // Initialize OIDC validator
    let config = OidcConfig::new(
        "https://your-oidc-provider.com".to_string(),
        "your-client-id".to_string(),
        "https://your-oidc-provider.com/.well-known/jwks.json".to_string(),
    );
    let oidc_validator = OidcValidator::new(config);

    // Configure validation rules
    let validation = Validation::default();

    // Create the authentication layer
    let auth_layer = OidcAuthLayer::<CustomClaims>::new(oidc_validator, validation);

    // Build your router with the middleware
    let app = Router::new()
        .route("/protected", get(protected_handler))
        .layer(auth_layer);

    // Run your server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn protected_handler(
    Extension(claims): Extension<CustomClaims>,
) -> &'static str {
    // Access validated claims here
    println!("User ID: {}", claims.sub);
    "Protected content"
}
```

## How it works

1. The middleware extracts the JWT token from the `Authorization` header (Bearer token)
2. Validates the token using the configured OIDC validator
3. Deserializes the claims into your custom type
4. Injects the claims into the request extensions
5. Continues to the next handler if validation succeeds

If validation fails, the request continues without claims in the extensions. You can implement your own authorization logic based on the presence or absence of claims.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
axum-jwt-oidc = "0.1.0"
```

## License

Licensed under * MIT license [LICENSE-MIT](LICENSE-MIT) 

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
