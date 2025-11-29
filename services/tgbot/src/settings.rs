use std::{sync::Arc, time::Duration};

pub const REDIS_USER_PREFIX: &str = "USER:";
pub const REDIS_USER_CALCS_PREFIX: &str = "USERS_CALC::";
pub const DEFAULT_MAX_CALCS: usize = 3;
pub const RESULT_POLL_INTERVAL: Duration = Duration::from_secs(15);
pub const SEND_THROTTLE: Duration = Duration::from_millis(800);

#[derive(Clone)]
pub struct BotConfig {
    pub calc_runner_base: String,
    pub max_active_calcs: usize,
}

#[derive(Clone)]
pub struct BotState {
    pub redis_client: Arc<redis::Client>,
    pub http_client: reqwest::Client,
    pub config: BotConfig,
}
