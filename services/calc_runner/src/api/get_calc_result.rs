
use axum::{
    extract::{Path, State},
    Json,
};
use common::{
    calc_info::{get_calc_info, AppState},
};
use serde::Serialize;
use uuid::Uuid;

use crate::api::errors::ApiError;

#[derive(Debug, Serialize)]
pub struct GetCalcResultResponse {
    pub calc_id: Uuid,
    pub run_dt: chrono::DateTime<chrono::Utc>,
    pub end_dt: Option<chrono::DateTime<chrono::Utc>>,
    pub params: Option<serde_json::Value>,
    pub progress: u32,
    pub result: Option<serde_json::Value>,
    pub duration: Option<i64>,
}

//
// Обработчик запросов на получение результатов расчета
//
pub async fn get_calc_result(
    State(state): State<AppState>,
    Path(calc_id): Path<Uuid>,
) -> Result<Json<GetCalcResultResponse>, ApiError> {
    let mut conn = state.redis_client.get_connection()?;
    let calc_info = get_calc_info(&mut conn, calc_id).map_err(ApiError::from)?;

    if calc_info.end_dt.is_none() {
        return Err(ApiError::CalculationNotCompleted(calc_id));
    } 
    
    Ok(Json(GetCalcResultResponse {
        calc_id: calc_info.calc_id,
        run_dt: calc_info.run_dt,
        end_dt: calc_info.end_dt,
        params: calc_info.params,
        progress: calc_info.progress,
        result: calc_info.result,
        duration: calc_info.end_dt.map(|end_dt| (end_dt - calc_info.run_dt).num_seconds()),
    }))
}
