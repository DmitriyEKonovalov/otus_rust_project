use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::api::ApiError;
use crate::models::CalcInfo;

#[derive(Debug, Serialize)]
pub struct GetCalcStatusResponse {
    pub calc_id: Uuid,
    pub user_id: i64,
    pub run_dt: chrono::DateTime<chrono::Utc>,
    pub progress: u32,
    pub duration: i64,
}

//
// Обработчик запросов на получение статуса расчета
pub async fn get_calc_status(
    State(state): State<AppState>,
    Path(calc_id): Path<Uuid>,
) -> Result<Json<GetCalcStatusResponse>, ApiError> {

    let storage = state.storage; 

    let calc_info:CalcInfo = storage.get(&CalcInfo::to_key(&calc_id)).await.map_err(ApiError::from)?;
    
    let duration: i64 = {
        if calc_info.end_dt.is_none() { 
            (Utc::now() - calc_info.run_dt).num_seconds()
        } else { 
            (calc_info.end_dt.unwrap() - calc_info.run_dt).num_seconds()
        }   
    }; 

    Ok(Json(GetCalcStatusResponse {
        calc_id: calc_info.calc_id,
        user_id: calc_info.user_id,
        run_dt: calc_info.run_dt,
        progress: calc_info.progress,
        duration,
    }))
}
