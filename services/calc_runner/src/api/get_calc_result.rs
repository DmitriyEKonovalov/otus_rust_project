//
// Обработчик запросов на получение результатов расечта
//

use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use common::{
    calc_info::CalcInfo,
    redis::{get_calc_info, AppState},
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
    pub duration: i64,
}

pub async fn get_calc_result(
    State(state): State<AppState>,
    Path(calc_id): Path<Uuid>,
) -> Result<Json<GetCalcResultResponse>, ApiError> {
    let mut conn = state.redis_client.get_connection()?;
    let calc_info = get_calc_info(&mut conn, calc_id)?.ok_or_else(not_found_error)?;

    Ok(Json(GetCalcResultResponse {
        calc_id: calc_info.calc_id,
        end_dt: calc_info.end_dt,
        result: calc_info.result,
    }))
}
