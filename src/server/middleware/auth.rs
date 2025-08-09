use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

use crate::{core::keys::SecretKeyV1, server::api::state::ApiState};

/// Validate a secret key and forward secret key payload to axum's extension extractor
pub async fn auth_layer(
    State(ApiState { state_db, .. }): State<ApiState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let sk = bearer.token();

    let (scope, payload) = SecretKeyV1::decode(sk).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            "Failed to decode secret key payload".to_string(),
        )
    })?;

    if state_db.is_key_revoked(&payload).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to check key revocation: {err}"),
        )
    })? {
        return Err((StatusCode::UNAUTHORIZED, "Key already revoked".to_string()));
    }

    if scope != "dev" {
        return Err((
            StatusCode::UNAUTHORIZED,
            format!("Scope {scope} not supported"),
        ));
    }

    if let Err(err) = payload.verify_signature() {
        return Err((
            StatusCode::UNAUTHORIZED,
            format!("Failed to verify secret key: {err}"),
        ));
    }

    // Secret key is valid
    req.extensions_mut().insert(payload);

    Ok(next.run(req).await)
}
