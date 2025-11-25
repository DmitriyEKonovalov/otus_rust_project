use std::{sync::Arc, thread};

use common::redis::set_result;
use uuid::Uuid;

use crate::api::errors::ApiError;

pub fn spawn_calc(
    calc_id: Uuid,
    params: Option<serde_json::Value>,
    client: Arc<redis::Client>,
    calc_fn: fn(Uuid, &mut redis::Connection, Option<serde_json::Value>) -> Result<(), ApiError>,
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
