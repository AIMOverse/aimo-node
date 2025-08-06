use crate::server::context::ServiceContext;

#[derive(Clone)]
pub(super) struct ApiState {
    pub ctx: ServiceContext,
}

impl ApiState {
    pub fn new(ctx: ServiceContext) -> Self {
        Self { ctx }
    }
}
