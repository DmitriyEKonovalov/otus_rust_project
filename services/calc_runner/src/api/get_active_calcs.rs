use axum::{
    extract::{State},
    Json,
};
use serde::Serialize;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::api::ApiError;
use crate::models::{CalcInfo};
use crate::models::{USER_CALC_PREFIX};


#[derive(Debug, Serialize)]
pub struct ShortCalcInfo {
    pub calc_id: Uuid,
    pub user_id: i64,
    pub run_dt: chrono::DateTime<chrono::Utc>,
    pub end_dt: Option<chrono::DateTime<chrono::Utc>>,
    pub progress: u32,
}

#[derive(Debug, Serialize)]
pub struct GetActiveCalcsResponse {
    pub calcs: Vec<ShortCalcInfo>,
}

//
// Обработчик запросов на получение всех расчето для всех пользователей
pub async fn get_active_calcs(
    State(state): State<AppState>,
) -> Result<Json<GetActiveCalcsResponse>, ApiError> {
    let storage = state.storage; 
    let calcs_keys = storage.keys(USER_CALC_PREFIX).await.map_err(ApiError::from)?;
    
    let mut calcs: Vec<ShortCalcInfo> = Vec::new();

    for calc_key in calcs_keys {
        let calc_info: CalcInfo = storage.get(&calc_key).await.map_err(ApiError::from)?;
        let calc = ShortCalcInfo {
            calc_id: calc_info.calc_id,
            user_id: calc_info.user_id,
            run_dt: calc_info.run_dt,
            end_dt: calc_info.end_dt,
            progress: calc_info.progress,
        };
        calcs.push(calc);
    }

    Ok(Json(GetActiveCalcsResponse { calcs: calcs,}))
}
