use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub redis_client: Arc<redis::Client>,
}
