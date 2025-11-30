use std::future::Future;
use crate::models::CalcInfo;
 use crate::api::ApiError;
use crate::storage::Storage;

// функция для запуска расчета в отдельном потоке
pub fn spawn_calc<F, Fut>(
    calc_fn: F,
    calc_info: CalcInfo,
    client: Storage,
) where
    F: Fn(CalcInfo, Storage) -> Fut
        + Send
        + Sync
        + 'static,
    Fut: Future<Output = Result<(), ApiError>> + Send + 'static,
{
    let calc_info_clone = calc_info.clone();
    let client_clone = client.clone();
    tokio::spawn(async move {
        if let Err(e) = calc_fn(calc_info_clone, client_clone).await {
            eprintln!("Error in calculation {}: {}", calc_info.calc_id, e);
        }
    });
}
