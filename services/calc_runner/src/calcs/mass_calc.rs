use chrono::Utc;
use std::time::Duration;
use tokio::time::sleep;
use crate::models::{CalcInfo, CALC_INFO_PREFIX, CALC_INFO_TTL_SECONDS};
use crate::storage::Storage;
use crate::api::ApiError;
use crate::api::run_mass_calc::MassCalcParams;

pub async fn mass_calc(
    calc_info: CalcInfo,
    storage: Storage,
) -> Result<(), ApiError> {
    let mut calc_info = calc_info.clone();
    let calc_key: String = format!("{}{}", CALC_INFO_PREFIX, calc_info.calc_id);
    let calc_params: MassCalcParams = serde_json::from_value(
        calc_info.params.clone().ok_or_else(|| ApiError::BadParams("Missing calculation parameters".into()))?
    )?;


    let mut simulations = Vec::with_capacity(calc_params.iterations as usize);

    for (i, &n) in calc_params.data.iter().enumerate() {
        let mut series = Vec::with_capacity(calc_params.iterations as usize);

        for _ in 0..calc_params.iterations {
            sleep(Duration::from_secs(5)).await;
            let value = Utc::now().timestamp_millis() * n as i64; // extract millis from now() like random value
            series.push(value);
        }

        simulations.insert(i, series);

        calc_info.progress = (((i + 1) * 100) / calc_params.data.len()) as u32;

        // обновление прогресса в хранилище
        let value = serde_json::to_string(&calc_info)?;
        storage.set(&calc_key, &value, CALC_INFO_TTL_SECONDS).await.map_err(ApiError::from)?;
    }

    // cохранение результата в хранилище
    let result = serde_json::json!({"simulations": simulations,});
    calc_info.result = Some(result);
    calc_info.end_dt = Some(chrono::Utc::now());
    calc_info.progress = 100;

    let value = serde_json::to_string(&calc_info)?;
    storage.set(&calc_key, &value, CALC_INFO_TTL_SECONDS).await.map_err(ApiError::from)?;

    Ok(())
}
