use std::time::Duration;
use tokio::time::sleep;
use chrono::Utc;
use crate::models::{CalcInfo};
use crate::storage::SharedStorage;
use crate::api::ApiError;
use crate::api::run_base_calc::BaseCalcParams;

pub async fn base_calc(
    calc_info: CalcInfo,
    storage: SharedStorage,
) -> Result<(), ApiError> {

    // расчет
    let calc_params: BaseCalcParams = serde_json::from_value(
        calc_info.params.clone().ok_or_else(|| ApiError::BadParams("Missing calculation parameters".into()))?
    )?;
    let mut simulations = Vec::with_capacity(calc_params.iterations as usize);
    for i in 0..calc_params.iterations {
        sleep(Duration::from_secs(10)).await;
        let value = Utc::now().timestamp_millis(); // extract millis from now() like random value
        simulations.push(value);


        // обновление прогресса в хранилище
        let progress = ((i + 1) * 100) / calc_params.iterations;
        storage.update_progress(&calc_info, progress).await.map_err(ApiError::from)?;

    }

    // cохранение результата в хранилище
    let result = serde_json::json!({"simulations": simulations,});
    storage.set_result(&calc_info, result).await.map_err(ApiError::from)?;

    Ok(())
}
