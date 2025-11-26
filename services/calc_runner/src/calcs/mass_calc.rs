use crate::api::errors::ApiError;
use common::redis::{set_result, update_progress};
use rand::Rng;
use redis::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use uuid::Uuid;

//
// Более сложная функция (mass_calc), имитирующая тяжелый расчет, запускаемая в отдельном потоке. 
// - получет последовательность чисел data и кол-во итераций n
// - для каждого data[i] создает n чисел rand(0..data[i]), с интервалом в 10 сек 
// - возвращает (записывает в redis) в поле с результатом json вида 
// { 
//     "simulations": [
//         [10, 55, -3, ...],
//         [-5, 5, 0, ...],
//         ... 
//     ] 
// } 

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassCalcParams {
    pub data: Vec<u32>,
    pub iterations: u32,
}

pub fn mass_calc(
    calc_id: Uuid,
    conn: &mut Connection,
    params: Option<serde_json::Value>,
) -> Result<(), ApiError> {
    let params: MassCalcParams = params
        .map(serde_json::from_value)
        .transpose()?
        .ok_or_else(|| ApiError::BadParams("data and iterations are required".into()))?;

    if params.iterations == 0 {
        return Err(ApiError::BadParams("iterations must be > 0".into()));
    }
    if params.data.is_empty() {
        return Err(ApiError::BadParams("data array must be non-empty".into()));
    }

    let mut rng = rand::thread_rng();
    let mut simulations: HashMap<String, Vec<u32>> = HashMap::new();

    for (i, &n) in params.data.iter().enumerate() {
        let mut series = Vec::with_capacity(params.iterations as usize);

        for _ in 0..params.iterations {
            sleep(Duration::from_secs(5));
            let value = rng.gen_range(0..n);
            series.push(value);
        }

        simulations.insert(n.to_string(), series);

        // обновить progress
        let progress = ((i + 1) * 100) / params.data.len();
        update_progress(conn, calc_id, progress as u32)?;
    }

    set_result(
        conn,
        calc_id,
        serde_json::json!({
            "simulations": simulations,
        }),
    )?;

    Ok(())
}
