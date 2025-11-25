use std::sync::Arc;

use axum::{extract::State, Json};
use chrono::Utc;
use common::{
    calc_info::CalcInfo,
    redis::{set_calc_info, AppState},
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    api::errors::ApiError,
    calcs::mass_calc::{mass_calc, MassCalcParams},
    utils::spawn_calc,
};


#[derive(Debug, Serialize)]
pub struct RunMassCalcResponse {
    pub calc_id: Uuid,
}

pub async fn run_mass_calc(
    State(state): State<AppState>,
    Json(payload): Json<MassCalcParams>,
) -> Result<Json<RunMassCalcResponse>, ApiError> {
    let calc_id = Uuid::new_v4();
    let now = Utc::now();

    let initial_info = CalcInfo {
        calc_id,
        run_dt: now,
        end_dt: None,
        params: Some(serde_json::to_value(&payload)?),
        progress: 0,
        result: None,
    };

    let mut conn = state.redis_client.get_connection()?;
    set_calc_info(&mut conn, calc_id, &initial_info)?;

    let client_clone = Arc::clone(&state.redis_client);
    let params_clone = Some(serde_json::to_value(&payload)?);
    spawn_calc(calc_id, params_clone, client_clone, mass_calc);

    Ok(Json(RunMassCalcResponse { calc_id }))
}
