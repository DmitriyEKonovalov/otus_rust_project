use axum::{
    extract::{Path, State},
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use redis::{Commands, Connection, RedisResult};
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    thread,
    time::Duration,
};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// === Структуры данных ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcInfo {
    pub calc_id: Uuid,
    pub run_dt: DateTime<Utc>,
    pub end_dt: Option<DateTime<Utc>>,
    pub params: Option<serde_json::Value>,
    pub progress: u32, // 0..100
    pub result: Option<serde_json::Value>,
}

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
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            ApiError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Json(_) => StatusCode::BAD_REQUEST,
        };
        let body = Json(ErrorResponse {
            error: self.to_string(),
        });
        (status, body).into_response()
    }
}

// === Состояние сервера ===

#[derive(Clone)]
struct AppState {
    redis_client: Arc<redis::Client>,
}

// === Вспомогательные функции Redis ===

fn get_calc_info(conn: &mut Connection, calc_id: Uuid) -> RedisResult<Option<CalcInfo>> {
    let key = format!("calc:{}", calc_id);
    let json: Option<String> = conn.get(&key)?;
    match json {
        Some(s) => {
            let info = serde_json::from_str(&s)?;
            Ok(Some(info))
        }
        None => Ok(None),
    }
}

fn set_calc_info(conn: &mut Connection, calc_id: Uuid, info: &CalcInfo) -> RedisResult<()> {
    let key = format!("calc:{}", calc_id);
    let json = serde_json::to_string(info)?;
    // TTL = 1 час
    conn.set_ex(&key, &json, 3600)?;
    Ok(())
}

fn update_progress(
    conn: &mut Connection,
    calc_id: Uuid,
    progress: u32,
) -> RedisResult<()> {
    let mut info = get_calc_info(conn, calc_id)?
        .ok_or_else(|| redis::RedisError::from((redis::ErrorKind::NotFound, "Task not found")))?;

    info.progress = progress;
    set_calc_info(conn, calc_id, &info)
}

fn set_result(
    conn: &mut Connection,
    calc_id: Uuid,
    result: serde_json::Value,
) -> RedisResult<()> {
    let mut info = get_calc_info(conn, calc_id)?
        .ok_or_else(|| redis::RedisError::from((redis::ErrorKind::NotFound, "Task not found")))?;

    info.end_dt = Some(Utc::now());
    info.result = Some(result);
    info.progress = 100;
    set_calc_info(conn, calc_id, &info)
}

// === Функция расчёта (пример) ===

/// Тип функции расчёта: принимает параметры и доступ к Redis
type CalcFn = Box<dyn FnOnce(Uuid, &mut Connection, Option<serde_json::Value>) + Send>;

fn example_calculation(
    calc_id: Uuid,
    conn: &mut Connection,
    params: Option<serde_json::Value>,
) -> Result<(), ApiError> {
    // Имитация долгой работы: 100 шагов
    for step in 0..=100 {
        thread::sleep(Duration::from_millis(30));

        // Обновляем прогресс в Redis
        update_progress(conn, calc_id, step)?;

        // Можно добавить early stop: если progress == 0 → отмена
    }

    // Формируем результат
    let result_value = serde_json::json!({
        "input_params": params,
        "status": "completed",
        "result_value": 42.0,
        "steps_done": 100
    });

    // Сохраняем результат
    set_result(conn, calc_id, result_value)?;

    Ok(())
}

// === Handler: создание расчёта ===

async fn submit_calculation(
    State(state): State<AppState>,
    Json(payload): Json<CalcRequest>,
) -> Result<Json<SubmitResponse>, ApiError> {
    let calc_id = Uuid::new_v4();
    let now = Utc::now();

    // Создаём начальную запись
    let initial_info = CalcInfo {
        calc_id,
        run_dt: now,
        end_dt: None,
        params: payload.params.clone(),
        progress: 0,
        result: None,
    };

    // Сохраняем в Redis
    let mut conn = state.redis_client.get_connection()?;
    set_calc_info(&mut conn, calc_id, &initial_info)?;

    // === Запуск расчёта в отдельном потоке ===
    let client_clone = Arc::clone(&state.redis_client);
    let params_clone = payload.params.clone();

    // Запускаем блокирующий расчёт в std::thread
    thread::spawn(move || {
        match client_clone.get_connection() {
            Ok(mut conn) => {
                if let Err(e) = example_calculation(calc_id, &mut conn, params_clone) {
                    eprintln!("❌ Calculation failed for {}: {}", calc_id, e);
                    // Можно записать ошибку в Redis:
                    let _ = set_result(&mut conn, calc_id, serde_json::json!({
                        "error": e.to_string()
                    }));
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to get Redis connection in worker: {}", e);
            }
        }
    });

    Ok(Json(SubmitResponse { calc_id }))
}

// === Handler: получение статуса ===

async fn get_calculation_status(
    State(state): State<AppState>,
    Path(calc_id): Path<Uuid>,
) -> Result<Json<CalcInfo>, ApiError> {
    let mut conn = state.redis_client.get_connection()?;
    let info = get_calc_info(&mut conn, calc_id)?
        .ok_or_else(|| redis::RedisError::from((redis::ErrorKind::NotFound, "Task not found")))?;

    Ok(Json(info))
}

// === Запуск сервера ===

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Подключение к Redis (по умолчанию — localhost:6379)
    let redis_client = redis::Client::open("redis://127.0.0.1/").expect("Invalid Redis URL");
    
    // Проверка подключения
    let _: String = redis_client
        .get_connection()?
        .ping()?;

    println!("✅ Connected to Redis");

    let app_state = AppState {
        redis_client: Arc::new(redis_client),
    };

    let app = Router::new()
        .route("/calc", post(submit_calculation))
        .route("/calc/:id", get(get_calculation_status))
        .with_state(app_state);

    println!("🚀 Server running on http://0.0.0.0:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}