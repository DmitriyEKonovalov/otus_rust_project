use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use uuid::Uuid;
use std::collections::HashSet;
use crate::app_state::AppState;
use crate::api::ApiError;
use crate::models::{UserCalc, USER_CALC_PREFIX};

#[derive(Debug, Serialize)]
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
    let user_calcs_keys = storage.mget(USER_CALC_PREFIX).await.map_err(ApiError::from)?;
    
    let mut calcs: HashSet<Uuid> = HashSet::new();
    for user_calc_key in user_calcs_keys {
        let user_calc: UserCalc = storage.get(&user_calc_key).await.map_err(ApiError::from)?;
        if user_calc.user_id == user_id {
            calcs.insert(user_calc.calc_id);
        }
    }

    Ok(Json(GetUserCalcsResponse {
        user_id: user_id,
        calcs: calcs,
    }))
}
