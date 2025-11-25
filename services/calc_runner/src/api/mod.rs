use std::{sync::Arc, thread};

use common::redis::set_result;
use uuid::Uuid;

use errors::ApiError;

pub mod errors;
pub mod get_calc_result;
pub mod get_calc_status;
pub mod run_base_calc;
pub mod run_mass_calc;

pub use errors::{ApiError, ErrorResponse};
pub use get_calc_result::{get_calculation_result, GetCalcResultResponse};
pub use get_calc_status::{get_calculation_status, GetCalcStatusResponse};
pub use run_base_calc::{run_base_calc, RunCalcRequest as RunBaseCalcRequest, RunCalcResponse};
pub use run_mass_calc::{run_mass_calc, RunCalcRequest as RunMassCalcRequest};

pub(crate) fn spawn_calc(
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
