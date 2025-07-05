use async_oidc_jwt_validator::{OidcValidator, Validation};
use std::{marker::PhantomData, sync::Arc};
use tower::Layer;

use crate::middleware::OidcAuthMiddleware;

/// A Tower layer that adds OIDC JWT authentication to your Axum application.
///
/// This layer will extract JWT tokens from the Authorization header, validate them
/// using the provided OIDC validator, and inject the claims into the request extensions.
#[derive(Clone)]
pub struct OidcAuthLayer<T> {
    pub(crate) oidc_validator: Arc<OidcValidator>,
    pub(crate) validation: Validation,
    pub(crate) _phantom: PhantomData<T>,
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
