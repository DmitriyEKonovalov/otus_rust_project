use std::future::Future;
use crate::models::CalcInfo;
 use crate::api::ApiError;
use crate::storage::SharedStorage;

// функция для запуска расчета в отдельном потоке
pub fn spawn_calc<F, Fut>(calc_fn: F, calc_info: CalcInfo, storage: SharedStorage)
where
    F: Fn(CalcInfo, SharedStorage) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), ApiError>> + Send + 'static,
{   
    let calc_id = calc_info.calc_id;
    tokio::spawn(async move {
        if let Err(e) = calc_fn(calc_info, storage).await {
            eprintln!("Error in calculation {}: {}", calc_id, e);
        }
    });
}
