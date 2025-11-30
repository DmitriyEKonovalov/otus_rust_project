use axum::{extract::State, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::models::CalcInfo;
use crate::api::ApiError;
use crate::calcs::spawn_calc;
use crate::calcs::mass_calc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassCalcParams {
    pub data: Vec<u32>,
    pub iterations: u32,
}

#[derive(Debug, Serialize)]
pub struct RunMassCalcResponse {
    pub calc_id: Uuid,
}

//
// Обработчик запуска mass_calc расчета
pub async fn run_mass_calc(
    State(state): State<AppState>,
    Json(params): Json<MassCalcParams>,
) -> Result<Json<RunMassCalcResponse>, ApiError> {
    let calc_params: MassCalcParams = params.clone();
    if calc_params.iterations == 0 {
        return Err(ApiError::BadParams("iterations must be > 0".into()));
    }
    // создаем новый расчет
    let calc_id = Uuid::new_v4();
    let calc_info = CalcInfo {
        calc_id: calc_id,
        run_dt: Utc::now(),
        end_dt: None,
        params: Some(serde_json::to_value(&params).unwrap()),
        progress: 0,
        result: None,
    };

    // запустить отедльный поток с расчетом
    spawn_calc(mass_calc, calc_info, state.storage);

    Ok(Json(RunMassCalcResponse { calc_id: calc_id }))
}
