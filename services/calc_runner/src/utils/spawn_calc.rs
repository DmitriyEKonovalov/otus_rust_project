use std::future::Future;
use std::sync::Arc;

use common::CalcInfo;
use serde_json;
use uuid::Uuid;

use crate::api::errors::ApiError;

pub fn spawn_calc<F, Fut>(
    calc_id: Uuid,
    calc_fn: F,
    params: Option<serde_json::Value>,
    client: Arc<redis::Client>,
) where
    F: Fn(
            Uuid,
            &mut redis::aio::MultiplexedConnection,
            Option<serde_json::Value>,
        ) -> Fut
        + Send
        + Sync
        + 'static,
    Fut: Future<Output = Result<(), ApiError>> + Send + 'static,
{
    tokio::spawn(async move {
        match client.get_multiplexed_async_connection().await {
            Ok(mut conn) => {
                if let Err(e) = calc_fn(calc_id, &mut conn, params).await {
                    eprintln!("Calculation failed for {}: {}", calc_id, e);
                    let _ = CalcInfo::set_result(
                        &mut conn,
                        calc_id,
                        serde_json::json!({ "error": e.to_string() }),
                    )
                    .await;
                }
            }
            Err(e) => {
                eprintln!("Failed to get Redis connection in worker: {}", e);
            }
        }
    });
}
