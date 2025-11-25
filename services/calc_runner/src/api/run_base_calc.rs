use std::sync::Arc;

use axum::{extract::State, Json};
use chrono::Utc;
use common::{
    calc_info::CalcInfo,
    redis::{set_calc_info, AppState},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::{errors::ApiError, spawn_calc},
    calcs::base_calc::base_calc,
};

#[derive(Debug, Deserialize)]
pub struct RunCalcRequest {
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct RunCalcResponse {
    pub calc_id: Uuid,
}

pub async fn run_base_calc(
    State(state): State<AppState>,
    Json(payload): Json<RunCalcRequest>,
) -> Result<Json<RunCalcResponse>, ApiError> {
    let calc_id = Uuid::new_v4();
    let now = Utc::now();

    let initial_info = CalcInfo {
        calc_id,
        run_dt: now,
        end_dt: None,
        params: payload.params.clone(),
        progress: 0,
        result: None,
    };

    let mut conn = state.redis_client.get_connection()?;
    set_calc_info(&mut conn, calc_id, &initial_info)?;

    let client_clone = Arc::clone(&state.redis_client);
    let params_clone = payload.params.clone();
    spawn_calc(calc_id, params_clone, client_clone, base_calc);

    Ok(Json(RunCalcResponse { calc_id }))
}
