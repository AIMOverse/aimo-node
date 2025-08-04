use crate::server::context::ServiceContext;

#[derive(Clone)]
pub(super) struct ApiState {
    ctx: ServiceContext,
}

impl ApiState {
    pub fn new(ctx: ServiceContext) -> Self {
        Self { ctx }
    }
}
