use std::sync::Arc;

use crate::core::router::Router;

#[derive(Clone)]
pub struct ServiceContext {
    pub(super) router: Arc<dyn Router + Send + Sync>,
}

impl ServiceContext {
    pub fn new(router: Arc<dyn Router + Send + Sync>) -> Self {
        Self { router }
    }
}
