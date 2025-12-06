use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashSet;
use crate::app_state::AppState;
use crate::api::ApiError;
use crate::models::UserCalcs;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserCalcsResponse {
    pub user_id: i64,
    pub calcs: HashSet<Uuid>,
}

//
// Обработчик запросов на получение незавершенных расчетов пользователя
pub async fn get_user_calcs(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<GetUserCalcsResponse>, ApiError> {
    let storage = state.storage; 

    let user_calcs: UserCalcs = storage.get(&UserCalcs::to_key(&user_id)).await.map_err(ApiError::from)?;
    
    Ok(Json(GetUserCalcsResponse {
        user_id: user_id,
        calcs: user_calcs.calcs,
    }))
}
