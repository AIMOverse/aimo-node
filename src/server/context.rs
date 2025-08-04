use std::sync::Arc;

use crate::router::RouterDyn;

#[derive(Clone)]
pub struct ServiceContext {
    pub(super) router: Arc<dyn RouterDyn + Send + Sync>,
}

impl ServiceContext {
    pub fn new(router: Arc<dyn RouterDyn + Send + Sync>) -> Self {
        Self { router }
    }
}
