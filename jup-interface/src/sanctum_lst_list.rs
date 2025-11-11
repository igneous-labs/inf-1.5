use anyhow::Result;
use backoff::{retry, ExponentialBackoff};
use sanctum_lst_list::{SanctumLst, SanctumLstList};

pub const SANCTUM_API_URL: &str = "https://sanctum-api.ironforge.network/lsts-pub";

fn load_remote_sanctum_lst_list_with_retries() -> Result<Vec<SanctumLst>> {
    let mut response = retry(ExponentialBackoff::default(), || {
        ureq::get(SANCTUM_API_URL).call().map_err(Into::into)
    })?;

    let SanctumLstList { data } = response.body_mut().read_json()?;
    Ok(data)
}

pub fn load_sanctum_lst_list() -> Vec<SanctumLst> {
    match load_remote_sanctum_lst_list_with_retries() {
        Ok(sanctum_lst_list) => sanctum_lst_list,
        Err(_error) => SanctumLstList::load().data,
    }
}
