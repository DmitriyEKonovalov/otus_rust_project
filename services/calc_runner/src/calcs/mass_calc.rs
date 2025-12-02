use chrono::Utc;
use std::time::Duration;
use tokio::time::sleep;
use crate::models::{CalcInfo};
use crate::storage::SharedStorage;
use crate::api::ApiError;
use crate::api::run_mass_calc::MassCalcParams;

pub async fn mass_calc(
    calc_info: CalcInfo,
    storage: SharedStorage,
) -> Result<(), ApiError> {
    // сохранить запись перед началом расчета в хранилище
    storage.start_calc(&calc_info).await.map_err(ApiError::from)?;

    // расчет 
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


        // обновление прогресса в хранилище
        let progress = (((i + 1) * 100) / calc_params.data.len()) as u32;
        storage.update_progress(&calc_info, progress).await.map_err(ApiError::from)?;

    }

    // cохранение результата в хранилище
    let result = serde_json::json!({"simulations": simulations,});
    storage.set_result(&calc_info, result).await.map_err(ApiError::from)?;

    Ok(())
}
