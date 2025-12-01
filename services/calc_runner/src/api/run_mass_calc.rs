use axum::{extract::State, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::models::{CalcInfo, UserCalc};
use crate::models::{CALC_INFO_PREFIX, USER_CALC_PREFIX};
use crate::models::{CALC_INFO_TTL_SECONDS, USER_CALC_TTL_SECONDS};
use crate::api::ApiError;
use crate::calcs::spawn_calc;
use crate::calcs::mass_calc;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassCalcParams {
    pub user_id: i64,
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
    let user_id = params.user_id;

    let calc_info = CalcInfo {
        calc_id: calc_id,
        user_id: user_id,
        run_dt: Utc::now(),
        end_dt: None,
        params: Some(serde_json::to_value(&params).unwrap()),
        progress: 0,
        result: None,
    };

    let user_calc = UserCalc{
        user_id: user_id,
        calc_id: calc_id
    };

    // запустить отедльный поток с расчетом
    spawn_calc(mass_calc, calc_info.clone(), state.storage.clone());
    
    // сохранить инфу о расчете в хранилище
    let calc_info_key: String = format!("{}{}", CALC_INFO_PREFIX, calc_id);
    state.storage.set(&calc_info_key, &calc_info, CALC_INFO_TTL_SECONDS).await.map_err(ApiError::from)?;

    let user_calc_key: String = format!("{}{}", USER_CALC_PREFIX, user_id);
    state.storage.set(&user_calc_key, &user_calc, USER_CALC_TTL_SECONDS).await.map_err(ApiError::from)?;

    Ok(Json(RunMassCalcResponse { calc_id: calc_id }))
}
