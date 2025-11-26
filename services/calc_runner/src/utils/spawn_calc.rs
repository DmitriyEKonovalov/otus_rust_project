use std::{sync::Arc, thread};

use common::redis::set_result;
use uuid::Uuid;

use crate::api::errors::ApiError;

// Вспомогательная функция для запуска расчета в отдельном потоке, 
// который запускается в хэндлере, но продолжает работать после выхода из него.   
// Принимает id расчета, функцию, параметры к ней и клиент Redis  
pub fn spawn_calc(
    calc_id: Uuid,
    calc_fn: fn(Uuid, &mut redis::Connection, Option<serde_json::Value>) -> Result<(), ApiError>,
    params: Option<serde_json::Value>,
    client: Arc<redis::Client>,
) {
    thread::spawn(move || match client.get_connection() {
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
    });
}
