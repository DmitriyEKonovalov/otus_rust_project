use axum::{
    extract::{State},
    Json,
};
use serde::Serialize;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::api::ApiError;
use crate::models::CalcInfo;
use crate::models::CALC_INFO_PREFIX;

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

    let all_calcs: Option<Vec<CalcInfo>> = storage.get_all(&format!("{}:*", CALC_INFO_PREFIX)).await.map_err(ApiError::from)?;
    let mut short_active_calcs: Vec<ShortCalcInfo> = Vec::new();
    if let Some(calcs) = all_calcs {
        for calc in calcs.into_iter().filter(|c| c.end_dt.is_none()) {
            short_active_calcs.push(ShortCalcInfo {
                calc_id: calc.calc_id,
                user_id: calc.user_id,
                run_dt: calc.run_dt,
                end_dt: calc.end_dt,
                progress: calc.progress,
            });
        }
    }
    Ok(Json(GetActiveCalcsResponse { calcs: short_active_calcs,}))
}
