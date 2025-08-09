use std::sync::Arc;

use crate::{db::StateDb, server::context::ServiceContext};

#[derive(Clone)]
pub struct ApiState {
    pub ctx: ServiceContext,
    pub state_db: Arc<StateDb>,
}

impl ApiState {
    pub fn new(ctx: ServiceContext, state_db: Arc<StateDb>) -> Self {
        Self { ctx, state_db }
    }
}
