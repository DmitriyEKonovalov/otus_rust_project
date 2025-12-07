mod api;
mod app_state;
mod calcs;
mod models;
mod storage;

use std::{env, sync::Arc};

use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use deadpool_redis::{Config as RedisConfig, Runtime};
use tracing_subscriber::EnvFilter;

use crate::api::{
    run_base_calc, 
    run_mass_calc,
    get_calc_status,
    get_calc_result,
    get_user_calcs,
    get_active_calcs
};

async fn healthcheck() -> StatusCode {
    StatusCode::OK
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let redis_password = env::var("REDIS_PASSWORD").ok();
    let redis_url = env::var("REDIS_URL")
        .ok()
        .filter(|url| !url.is_empty())
        .or_else(|| {
            let host = env::var("REDIS_HOST").unwrap_or_else(|_| "redis".into());
            let port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".into());
            Some(match &redis_password {
                Some(password) if !password.is_empty() => {
                    format!("redis://:{}@{}:{}/", password, host, port)
                }
                _ => format!("redis://{}:{}/", host, port),
            })
        })
        .unwrap_or_else(|| "redis://127.0.0.1/".into());

    let redis_cfg = RedisConfig::from_url(redis_url);
    let pool = redis_cfg.create_pool(Some(Runtime::Tokio1))?;

    {
        let mut conn = pool.get().await?;
        let _: String = deadpool_redis::redis::cmd("PING").query_async(&mut conn).await?;
    }

    let storage = Arc::new(storage::Storage::new(pool));
    let app_state = app_state::AppState { storage: storage.clone() };

    let app = Router::new()
        .route("/health", get(healthcheck))
        .route("/api/calc/base_calc", post(run_base_calc))
        .route("/api/calc/mass_calc", post(run_mass_calc))
        .route("/api/calc/:id", post(get_calc_status))
        .route("/api/calc/result/:id", post(get_calc_result))
        .route("/api/stats/user/:id", post(get_user_calcs))
        .route("/api/stats/active_calcs", post(get_active_calcs))
        .with_state(app_state);

    let app_host = env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let app_port = env::var("CALC_RUNNER_PORT").unwrap_or_else(|_| "3000".into());
    let listen_addr = format!("{app_host}:{app_port}");
    let listener = TcpListener::bind(&listen_addr).await?;
    tracing::info!("Server running on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
