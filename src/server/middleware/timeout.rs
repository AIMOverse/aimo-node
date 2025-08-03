use std::time::Duration;

use tower_http::timeout::TimeoutLayer;

use crate::config::ApiOptions;

pub fn timeout_layer(_options: &ApiOptions) -> TimeoutLayer {
    //TODO: make this configurable
    TimeoutLayer::new(Duration::from_secs(30))
}
