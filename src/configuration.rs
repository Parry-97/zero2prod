use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub app_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub db_name: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let mut settings = config::Config::default();

    //NOTE: Add configuration values from a file name `configuration`.
    //It will look for any top-level file with an extension what
    //`config` knows how to parse: yaml, json, etc.
    settings.merge(config::File::with_name("configuration"))?;

    //NOTE: Try to convert the configuration values it read into our Settings type
    settings.try_into()
}
