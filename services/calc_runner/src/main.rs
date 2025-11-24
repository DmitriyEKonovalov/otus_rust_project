use axum::{
    extract::{Path, State},
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use chrono::Utc;
use common::calc_info::CalcInfo;
use common::redis::{get_calc_info, not_found_error, set_calc_info, set_result, AppState, RedisDataError};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, thread};
use tokio::net::TcpListener;
use uuid::Uuid;

mod calcs;
use calcs::{base_calc::base_calc, mass_calc::mass_calc};

// === Структуры данных ===

#[derive(Debug, Deserialize)]
pub struct CalcRequest {
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SubmitResponse {
    pub calc_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// === Ошибки ===
#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error(transparent)]
    Redis(#[from] RedisDataError),
    #[error("Redis error: {0}")]
    RedisClient(#[from] redis::RedisError),
    #[error("Bad params: {0}")]
    BadParams(String),
    #[error("Invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            ApiError::BadParams(_) => StatusCode::BAD_REQUEST,
            ApiError::Json(_) => StatusCode::BAD_REQUEST,
            ApiError::Redis(_) | ApiError::RedisClient(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(ErrorResponse {
            error: self.to_string(),
        });
        (status, body).into_response()
    }
}


fn spawn_calc(
    calc_id: Uuid,
    params: Option<serde_json::Value>,
    client: Arc<redis::Client>,
    calc_fn: fn(Uuid, &mut redis::Connection, Option<serde_json::Value>) -> Result<(), ApiError>,
) {
    thread::spawn(move || {
        match client.get_connection() {
            Ok(mut conn) => {
                if let Err(e) = calc_fn(calc_id, &mut conn, params) {
                    eprintln!("? Calculation failed for {}: {}", calc_id, e);
                    let _ = set_result(&mut conn, calc_id, serde_json::json!({
                        "error": e.to_string()
                    }));
                }
            }
            Err(e) => {
                eprintln!("? Failed to get Redis connection in worker: {}", e);
            }
        }
    });
}

// === Handler: запуск base_calc ===

async fn run_base_calc(
    State(state): State<AppState>,
    Json(payload): Json<CalcRequest>,
) -> Result<Json<SubmitResponse>, ApiError> {
    let calc_id = Uuid::new_v4();
    let now = Utc::now();

    let initial_info = CalcInfo {
        calc_id,
        run_dt: now,
        end_dt: None,
        params: payload.params.clone(),
        progress: 0,
        result: None,
    };

    let mut conn = state.redis_client.get_connection()?;
    set_calc_info(&mut conn, calc_id, &initial_info)?;

    let client_clone = Arc::clone(&state.redis_client);
    let params_clone = payload.params.clone();
    spawn_calc(calc_id, params_clone, client_clone, base_calc);

    Ok(Json(SubmitResponse { calc_id }))
}

// === Handler: запуск mass_calc ===

async fn run_mass_calc(
    State(state): State<AppState>,
    Json(payload): Json<CalcRequest>,
) -> Result<Json<SubmitResponse>, ApiError> {
    let calc_id = Uuid::new_v4();
    let now = Utc::now();

    let initial_info = CalcInfo {
        calc_id,
        run_dt: now,
        end_dt: None,
        params: payload.params.clone(),
        progress: 0,
        result: None,
    };

    let mut conn = state.redis_client.get_connection()?;
    set_calc_info(&mut conn, calc_id, &initial_info)?;

    let client_clone = Arc::clone(&state.redis_client);
    let params_clone = payload.params.clone();
    spawn_calc(calc_id, params_clone, client_clone, mass_calc);

    Ok(Json(SubmitResponse { calc_id }))
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    run_dt: chrono::DateTime<chrono::Utc>,
    progress: u32,
}


// === Handler: ????????? ??????? ===
async fn get_calculation_status(
    State(state): State<AppState>,
    Path(calc_id): Path<Uuid>,
) -> Result<Json<StatusResponse>, ApiError> {
    let mut conn = state.redis_client.get_connection()?;
    let info = get_calc_info(&mut conn, calc_id)?
        .ok_or_else(not_found_error)?;

    Ok(Json(StatusResponse {
        run_dt: info.run_dt,
        progress: info.progress,
    }))
}

#[derive(Debug, Serialize)]
struct ResultResponse {
    calc_id: Uuid,
    end_dt: Option<chrono::DateTime<chrono::Utc>>,
    result: Option<serde_json::Value>,
}

async fn get_calculation_result(
    State(state): State<AppState>,
    Path(calc_id): Path<Uuid>,
) -> Result<Json<ResultResponse>, ApiError> {
    let mut conn = state.redis_client.get_connection()?;
    let info = get_calc_info(&mut conn, calc_id)?
        .ok_or_else(not_found_error)?;

    Ok(Json(ResultResponse {
        calc_id: info.calc_id,
        end_dt: info.end_dt,
        result: info.result,
    }))
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Подключение к Redis (по умолчанию — localhost:6379)
    let redis_client = redis::Client::open("redis://127.0.0.1/").expect("Invalid Redis URL");
    
    // Проверка подключения
    let mut ping_conn = redis_client.get_connection()?;
    let _: String = redis::cmd("PING").query(&mut ping_conn)?;

    println!("✅ Connected to Redis");

    let app_state = AppState {
        redis_client: Arc::new(redis_client),
    };

    let app = Router::new()
        .route("/api/calc/base_calc", post(run_base_calc))
        .route("/api/calc/mass_calc", post(run_mass_calc))
        .route("/api/calc/:id", get(get_calculation_status))
        .route("/api/calc/result/:id", get(get_calculation_result))
        .with_state(app_state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Server running on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
