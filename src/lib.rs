//! # axum-jwt-oidc
//!
//! Axum middleware for OIDC JWT token validation and claims extraction.
//!
//! This crate provides a Tower layer and service that can be used to validate JWT tokens
//! in your Axum applications. It integrates with the `async-oidc-jwt-validator` crate
//! to provide seamless JWT validation using OIDC provider discovery.
//!
//! ## Example
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
//! }
//!
//! # #[tokio::main]
//! # async fn main() {
//! let config = OidcConfig::new(
//!     "https://your-oidc-provider.com".to_string(),
//!     "your-client-id".to_string(),
//!     "https://your-oidc-provider.com/.well-known/jwks.json".to_string(),
//! );
//! let oidc_validator = OidcValidator::new(config);
//! let validation = Validation::default();
//! let auth_layer = OidcAuthLayer::<CustomClaims>::new(oidc_validator, validation);
//!
//! let app: Router = Router::new()
//!     .route("/protected", get(|Extension(claims): Extension<CustomClaims>| async move {
//!         format!("Hello {}", claims.sub)
//!     }))
//!     .layer(auth_layer);
//! # }
//! ```

use async_oidc_jwt_validator::{OidcValidator, Validation};
use axum::{extract::Request, response::Response};
use futures::future::BoxFuture;
use http::HeaderMap;
use serde::de::DeserializeOwned;
use std::{
    marker::PhantomData,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};

/// A Tower layer that adds OIDC JWT authentication to your Axum application.
///
/// This layer will extract JWT tokens from the Authorization header, validate them
/// using the provided OIDC validator, and inject the claims into the request extensions.
#[derive(Clone)]
pub struct OidcAuthLayer<T> {
    oidc_validator: Arc<OidcValidator>,
    validation: Validation,
    _phantom: PhantomData<T>,
}

impl<T> OidcAuthLayer<T> {
    /// Creates a new authentication layer with the provided OIDC validator and validation rules.
    pub fn new(oidc_validator: OidcValidator, validation: Validation) -> Self {
        Self {
            oidc_validator: Arc::new(oidc_validator),
            validation,
            _phantom: PhantomData,
        }
    }
}

impl<S, T> Layer<S> for OidcAuthLayer<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Service = OidcAuthMiddleware<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        OidcAuthMiddleware {
            inner,
            oidc_validator: self.oidc_validator.clone(),
            validation: self.validation.clone(),
            _phantom: PhantomData,
        }
    }
}

/// The middleware service that performs JWT validation.
///
/// This service is created by the `OidcAuthLayer` and should not be instantiated directly.
#[derive(Clone)]
pub struct OidcAuthMiddleware<S, T> {
    inner: S,
    oidc_validator: Arc<OidcValidator>,
    validation: Validation,
    _phantom: PhantomData<T>,
}

impl<S, T> Service<Request> for OidcAuthMiddleware<S, T>
where
    S: Service<Request, Response = Response> + Send + 'static + Clone,
    S::Future: Send + 'static,
    T: DeserializeOwned + Clone + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let not_ready_inner = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, not_ready_inner);
        let oidc_validator = self.oidc_validator.clone();
        let validation = self.validation.clone();

        Box::pin(async move {
            // Extract and validate claims
            if let Some(claims) =
                validate_auth_header::<T>(req.headers(), &oidc_validator, &validation).await
            {
                // Store claims directly in request extensions
                req.extensions_mut().insert(claims);
            }

            // Call the inner service
            inner.call(req).await
        })
    }
}

async fn validate_auth_header<T>(
    headers: &HeaderMap,
    oidc_validator: &OidcValidator,
    validation: &Validation,
) -> Option<T>
where
    T: DeserializeOwned + Clone,
{
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());
    log::debug!("Extracting claims from headers...");

    if let Some(auth_header) = auth_header {
        let token = auth_header.strip_prefix("Bearer ").unwrap_or(auth_header);

        match oidc_validator.validate_custom::<T>(token, validation).await {
            Ok(claims) => {
                log::info!("Successfully authenticated token");
                return Some(claims);
            }
            Err(e) => {
                log::warn!("Authentication failed: {e}");
            }
        }
    }

    None
}
