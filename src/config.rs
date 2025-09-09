use std::default::Default;

#[derive(Default)]
pub struct Config {
    pub cmc_api_key: String,
    pub use_cmc_sandbox_api: bool,
}