use async_oidc_jwt_validator::{OidcValidator, Validation};
use axum::{extract::Request, response::Response};
use futures::future::BoxFuture;
use serde::de::DeserializeOwned;
use std::{
    marker::PhantomData,
    sync::Arc,
    task::{Context, Poll},
};
use tower::Service;

use crate::auth::validate_auth_header;

/// The middleware service that performs JWT validation.
///
/// This service is created by the `OidcAuthLayer` and should not be instantiated directly.
#[derive(Clone)]
pub struct OidcAuthMiddleware<S, T> {
    pub(crate) inner: S,
    pub(crate) oidc_validator: Arc<OidcValidator>,
    pub(crate) validation: Validation,
    pub(crate) _phantom: PhantomData<T>,
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
