use crate::api::errors::ApiError;
use rand::Rng;
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassCalcParams {
    pub data: Vec<u32>,
    pub iterations: u32,
}

pub async fn mass_calc(
    calc_id: Uuid,
    conn: &mut MultiplexedConnection,
    params: Option<serde_json::Value>,
) -> Result<(), ApiError> {
    let params: MassCalcParams = params
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

    for (i, &n) in params.data.iter().enumerate() {
        let mut series = Vec::with_capacity(params.iterations as usize);

        for _ in 0..params.iterations {
            sleep(Duration::from_secs(5)).await;
            let value = rng.gen_range(0..n);
            series.push(value);
        }

        simulations.insert(n.to_string(), series);

        let progress = ((i + 1) * 100) / params.data.len();
        CalcInfo::update_progress(conn, calc_id, progress as u32).await?;
    }

    CalcInfo::set_result(
        conn,
        calc_id,
        serde_json::json!({
            "simulations": simulations,
        }),
    )
    .await?;

    Ok(())
}
