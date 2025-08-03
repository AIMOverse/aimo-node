use http::Method;
use tower_http::cors::{Any, CorsLayer};

use crate::config::ApiOptions;

pub fn cors_layer(_options: &ApiOptions) -> CorsLayer {
    // TODO: Make this configurable

    CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any)
}
