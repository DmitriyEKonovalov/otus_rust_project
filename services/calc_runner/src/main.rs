mod api;
mod calcs;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use common::redis::AppState;
use tokio::net::TcpListener;

use crate::api::{
    get_calculation_result, get_calculation_status, run_base_calc, run_mass_calc,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // �?�?�?��>�?�ؐ�?��� �� Redis (���? �?�?�?�>�ؐ��?��? �?" localhost:6379)
    let redis_client = redis::Client::open("redis://127.0.0.1/").expect("Invalid Redis URL");
    
    // �?�?�?�?��?��� ���?�?��>�?�ؐ�?��?
    let mut ping_conn = redis_client.get_connection()?;
    let _: String = redis::cmd("PING").query(&mut ping_conn)?;

    println!("�?: Connected to Redis");

    let app_state = AppState {
        redis_client: Arc::new(redis_client),
    };

    let app = Router::new()
        .route("/api/calc/base_calc", post(run_base_calc))
        .route("/api/calc/mass_calc", post(run_mass_calc))
        .route("/api/calc/:id", get(get_calculation_status))
        .route("/api/calc/result/:id", get(get_calculation_result))
        .with_state(app_state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("�??? Server running on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
