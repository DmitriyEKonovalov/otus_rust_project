use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::api::ApiError;
use crate::calcs::spawn_calc;
use crate::calcs::mass_calc;
use tracing::info;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassCalcParams {
    pub user_id: i64,
    pub data: Vec<u32>,
    pub iterations: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunMassCalcResponse {
    pub calc_id: Uuid,
}

// Обработчик запуска mass_calc расчета
pub async fn run_mass_calc(
    State(state): State<AppState>,
    Json(params): Json<MassCalcParams>,
) -> Result<Json<RunMassCalcResponse>, ApiError> {
    info!(
        user_id = params.user_id,
        iterations = params.iterations,
        data_len = params.data.len(),
        "run_mass_calc endpoint called"
    );

    // проверка параметров 
    let calc_params: MassCalcParams = params.clone();
    if calc_params.iterations == 0 {
        return Err(ApiError::BadParams("iterations must be > 0".into()));
    }

    // создаем новый расчет
    let calc_info = state.storage.init_calc(params.user_id, serde_json::to_value(params).unwrap()).await?;
    let calc_id = calc_info.calc_id;
    info!(
        calc_id = %calc_id,
        user_id = calc_info.user_id,
        run_dt = %calc_info.run_dt,
        "mass calculation scheduled"
    );
    
    // запустить отедльный поток с расчетом
    spawn_calc(mass_calc, calc_info, state.storage);
    
    Ok(Json(RunMassCalcResponse { calc_id: calc_id }))
}
