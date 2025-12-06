pub use calc_runner::{
    BaseCalcParams, GetActiveCalcsResponse, GetCalcResultResponse, GetCalcStatusResponse,
    GetUserCalcsResponse, MassCalcParams, RunBaseCalcResponse, RunMassCalcResponse,
};
use reqwest::{Client, StatusCode};
use uuid::Uuid;

use crate::{exceptions::BotError, settings::BotConfig};

pub const RUN_BASE_CALC_URL: &str = "/calc/base_calc";
pub const RUN_MASS_CALC_URL: &str = "/calc/mass_calc";
pub const CALC_STATUS_URL: &str = "/calc";
pub const CALC_RESULT_URL: &str = "/calc/result";
pub const USER_CALCS_URL: &str = "/stats/user";
pub const ACTIVE_CALCS_URL: &str = "/stats/active_calcs";


pub async fn run_base_calc(
    client: &Client,
    config: &BotConfig,
    params: &BaseCalcParams,
) -> Result<RunBaseCalcResponse, BotError> {
    let url = config.api_url(RUN_BASE_CALC_URL);
    let response = client.post(url).json(params).send().await?;
    let response = response.error_for_status()?;
    Ok(response.json::<RunBaseCalcResponse>().await?)
}

pub async fn run_mass_calc(
    client: &Client,
    config: &BotConfig,
    params: &MassCalcParams,
) -> Result<RunMassCalcResponse, BotError> {
    let url = config.api_url(RUN_MASS_CALC_URL);
    let response = client.post(url).json(params).send().await?;
    let response = response.error_for_status()?;
    Ok(response.json::<RunMassCalcResponse>().await?)
}

pub async fn get_calc_status(
    client: &Client,
    config: &BotConfig,
    calc_id: Uuid,
) -> Result<GetCalcStatusResponse, BotError> {
    let url = config.api_url(&format!("{}/{}", CALC_STATUS_URL, calc_id));
    let response = client.post(url).send().await?;
    let response = response.error_for_status()?;
    Ok(response.json::<GetCalcStatusResponse>().await?)
}

pub async fn get_calc_result(
    client: &Client,
    config: &BotConfig,
    calc_id: Uuid,
) -> Result<GetCalcResultResponse, BotError> {
    let url = config.api_url(&format!("{}/{}", CALC_RESULT_URL, calc_id));
    let response = client.post(url).send().await?;
    let response = response.error_for_status()?;
    Ok(response.json::<GetCalcResultResponse>().await?)
}

pub async fn get_user_calcs(
    client: &Client,
    config: &BotConfig,
    user_id: i64,
) -> Result<GetUserCalcsResponse, BotError> {
    let url = config.api_url(&format!("{}/{}", USER_CALCS_URL, user_id));
    let response = client.post(url).send().await?;

    if response.status() == StatusCode::NOT_FOUND {
        return Ok(GetUserCalcsResponse {
            user_id,
            calcs: Default::default(),
        });
    }

    let response = response.error_for_status()?;
    Ok(response.json::<GetUserCalcsResponse>().await?)
}

pub async fn get_active_calcs(
    client: &Client,
    config: &BotConfig,
) -> Result<GetActiveCalcsResponse, BotError> {
    let url = config.api_url(ACTIVE_CALCS_URL);
    let response = client.post(url).send().await?;
    let response = response.error_for_status()?;
    Ok(response.json::<GetActiveCalcsResponse>().await?)
}

