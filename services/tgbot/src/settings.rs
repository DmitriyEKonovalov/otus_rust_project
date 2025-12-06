use std::{env, sync::Arc, time::Duration};

pub const MAX_CALC_FOR_USER: usize = 2;
pub const LIMIT_EXCEEDED_MESSAGE: &str ="Запущено одновременно слишком много расчетов. Дождитесь окончания предыдущих.";

// pub const RESULT_POLL_INTERVAL: Duration = Duration::from_secs(15);
// pub const SEND_THROTTLE: Duration = Duration::from_millis(800);

#[derive(Clone)]
pub struct BotConfig {
    pub calc_runner_url: String,
    pub admin_user_ids: Vec<i64>,
}

#[derive(Clone)]
pub struct BotState {
    pub http_client: reqwest::Client,
    pub config: BotConfig,
}

impl BotConfig {
    pub fn from_env() -> Self {
        let calc_runner_url = env::var("CALC_RUNNER_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:3000/api".into());

        let admin_user_ids = env::var("ADMIN_USER_IDS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|raw| raw.trim().parse::<i64>().ok())
            .collect();

        Self {
            calc_runner_url,
            admin_user_ids,
        }
    }

    pub fn api_url(&self, path: &str) -> String {
        let trimmed_base = self.calc_runner_url.trim_end_matches('/');
        let trimmed_path = path.trim_start_matches('/');
        format!("{}/{}", trimmed_base, trimmed_path)
    }
}
