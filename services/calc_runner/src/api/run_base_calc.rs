use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::api::ApiError;
use crate::calcs::spawn_calc;
use crate::calcs::base_calc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseCalcParams {
    pub user_id: i64,
    pub iterations: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunBaseCalcResponse {
    pub calc_id: Uuid,
}

//
// Обработчик запуска base_calc расчета
pub async fn run_base_calc(
    State(state): State<AppState>,
    Json(params): Json<BaseCalcParams>,
) -> Result<Json<RunBaseCalcResponse>, ApiError> {
    
    // проверка параметров 
    let calc_params: BaseCalcParams = params.clone();
    if calc_params.iterations == 0 {
        return Err(ApiError::BadParams("iterations must be > 0".into()));
    }

    // создаем новый расчет
    let calc_info = state.storage.init_calc(params.user_id, serde_json::to_value(params).unwrap()).await?;
    let calc_id = calc_info.calc_id;
    
    // запустить отедльный поток с расчетом
    spawn_calc(base_calc, calc_info, state.storage);

    Ok(Json(RunBaseCalcResponse { calc_id: calc_id }))
}
