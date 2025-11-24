use crate::ApiError;
use common::redis::{set_result, update_progress};
use rand::Rng;
use redis::Connection;
use serde::Deserialize;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct MassParams {
    data: Vec<u32>,
    iterations: u32,
}

pub fn mass_calc(
    calc_id: Uuid,
    conn: &mut Connection,
    params: Option<serde_json::Value>,
) -> Result<(), ApiError> {
    let params: MassParams = params
        .map(serde_json::from_value)
        .transpose()?
        .ok_or_else(|| ApiError::BadParams("data and iterations are required".into()))?;

    if params.iterations == 0 {
        return Err(ApiError::BadParams("iterations must be > 0".into()));
    }
    if params.data.is_empty() {
        return Err(ApiError::BadParams("data array must be non-empty".into()));
    }

    let mut rng = rand::thread_rng();
    let mut simulations: HashMap<String, Vec<u32>> = HashMap::new();

    for &n in &params.data {
        let progress = ((n as u64 * 100) / params.data.len() as u64).min(100) as u32;
        let mut series = Vec::with_capacity(params.iterations as usize);

        for _ in 0..params.iterations {
            sleep(Duration::from_secs(5));
            let value = rng.gen_range(0..=n);
            series.push(value);
            update_progress(conn, calc_id, progress)?;
        }

        simulations.insert(n.to_string(), series);
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
