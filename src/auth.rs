use async_oidc_jwt_validator::{OidcValidator, Validation};
use http::HeaderMap;
use serde::de::DeserializeOwned;

pub(crate) async fn validate_auth_header<T>(
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
