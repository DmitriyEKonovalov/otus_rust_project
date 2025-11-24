use crate::ApiError;
use common::redis::{set_result, update_progress};
use rand::Rng;
use redis::Connection;
use serde::Deserialize;
use std::thread::sleep;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct BaseParams {
    iterations: u32,
}

pub fn base_calc(
    calc_id: Uuid,
    conn: &mut Connection,
    params: Option<serde_json::Value>,
) -> Result<(), ApiError> {
    let params: BaseParams = params
        .map(serde_json::from_value)
        .transpose()?
        .ok_or_else(|| ApiError::BadParams("iterations are required".into()))?;

    if params.iterations == 0 {
        return Err(ApiError::BadParams("iterations must be > 0".into()));
    }

    let mut rng = rand::thread_rng();
    let mut simulations = Vec::with_capacity(params.iterations as usize);

    for i in 0..params.iterations {
        sleep(Duration::from_secs(10));
        let value = rng.gen_range(-100..=100);
        simulations.push(value);

        let progress = ((i + 1) * 100) / params.iterations;
        update_progress(conn, calc_id, progress)?;
    }

    set_result(
        conn,
        calc_id,
        serde_json::json!({
            "simulations": simulations,
        }),
    )?;

    Ok(())
}
