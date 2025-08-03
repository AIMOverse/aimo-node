use std::time::Duration;

use tower_http::timeout::TimeoutLayer;

use crate::config::ServerOptions;

pub fn timeout_layer(_options: &ServerOptions) -> TimeoutLayer {
    //TODO: make this configurable
    TimeoutLayer::new(Duration::from_secs(30))
}
