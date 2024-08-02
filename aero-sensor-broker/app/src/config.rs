use config::{Config, File};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfigSettings {
    pub influxdb: InfluxDBConfig,
    pub arduino: ArduinoConfig,
}

#[derive(Deserialize)]
pub struct InfluxDBConfig {
    pub url: String,
    pub bucket: String,
    pub org: String,
    pub auth_token: String,
}

#[derive(Deserialize)]
pub struct ArduinoConfig {
    pub baud_rate: u32,
    pub timeout: u64,
    pub device_name: String,
}

pub fn load_settings() -> Result<ConfigSettings, config::ConfigError> {
    Config::builder()
        .add_source(File::with_name("settings/Settings.toml"))
        .build()?
        .try_deserialize::<ConfigSettings>()
}
