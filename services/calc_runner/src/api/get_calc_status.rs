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
pub struct GetCalcStatusResponse {
    pub run_dt: chrono::DateTime<chrono::Utc>,
    pub progress: u32,
    pub duration: i64,
}

//
// Обработчик запросов на получение статуса расчета
//
pub async fn get_calc_status(
    State(state): State<AppState>,
    Path(calc_id): Path<Uuid>,
) -> Result<Json<GetCalcStatusResponse>, ApiError> {
    let mut conn = state.redis_client.get_connection()?;
    let calc_info = get_calc_info(&mut conn, calc_id)?;
    let CalcInfo { run_dt, progress, end_dt, .. } = calc_info;
    
    let duration: i64 = {
        if end_dt.is_none() { 
            (Utc::now() - run_dt).num_seconds() 
        } else { 
            (end_dt.unwrap() - run_dt).num_seconds()
        }   
    }; 
    

    Ok(Json(GetCalcStatusResponse {
        run_dt,
        progress,
        duration,
    }))
}
