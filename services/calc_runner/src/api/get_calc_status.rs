use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use crate::state::AppState;
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
    let mut conn: redis::aio::MultiplexedConnection = state.redis_client.get_multiplexed_async_connection().await?;
    let calc_info = CalcInfo::get(&mut conn, calc_id).await.map_err(ApiError::from)?;
    
    let duration: i64 = {
        if calc_info.end_dt.is_none() { 
            (Utc::now() - calc_info.run_dt).num_seconds() 
        } else { 
            (calc_info.end_dt.unwrap() - calc_info.run_dt).num_seconds()
        }   
    }; 
    

    Ok(Json(GetCalcStatusResponse {
        run_dt: calc_info.run_dt,
        progress: calc_info.progress,
        duration,
    }))
}
