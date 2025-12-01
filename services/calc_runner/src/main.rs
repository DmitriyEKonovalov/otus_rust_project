mod api;
mod app_state;
mod calcs;
mod models;
mod storage;

use std::sync::Arc;

use axum::{
    routing::{post},
    Router,
};
use tokio::net::TcpListener;
use deadpool_redis::{Config as RedisConfig, Runtime};

use crate::api::{
    run_base_calc, 
    run_mass_calc,
    get_calc_status,
    get_calc_result,
    get_user_calcs,
    get_active_calcs
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redis_cfg = RedisConfig::from_url("redis://127.0.0.1/");
    let pool = redis_cfg.create_pool(Some(Runtime::Tokio1))?;

    {
        let mut conn = pool.get().await?;
        let _: String = deadpool_redis::redis::cmd("PING").query_async(&mut conn).await?;
    }

    let storage = Arc::new(storage::Storage::new(pool));
    let app_state = app_state::AppState { storage: storage.clone() };

    let app = Router::new()
        .route("/api/calc/base_calc", post(run_base_calc))
        .route("/api/calc/mass_calc", post(run_mass_calc))
        .route("/api/calc/:id", post(get_calc_status))
        .route("/api/calc/result/:id", post(get_calc_result))
        .route("/api/stats/user/:id", post(get_user_calcs))
        .route("/api/stats/active_calcs", post(get_active_calcs))
        .with_state(app_state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("...Server running on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
