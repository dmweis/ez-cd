use std::path::PathBuf;

use anyhow::Context;
use serde::Deserialize;
use tracing::*;

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    #[serde(default)]
    zenoh: ZenohConfigWrapper,
}

impl AppConfig {
    /// Use default config if no path is provided
    pub fn read_config(config: &Option<PathBuf>) -> Result<AppConfig, anyhow::Error> {
        let mut config_builder = config::Config::builder();

        if let Some(config) = config {
            info!("Using configuration from {:?}", config);
            config_builder = config_builder.add_source(config::File::with_name(
                config.to_str().context("Failed to convert path")?,
            ));
        } else {
            info!("Using dev configuration");
            config_builder = config_builder
                .add_source(config::File::with_name("config/settings"))
                .add_source(config::File::with_name("config/dev_settings").required(false));
        }

        config_builder = config_builder.add_source(config::Environment::with_prefix("APP"));

        let config = config_builder.build()?;

        Ok(config.try_deserialize::<AppConfig>()?)
    }

    pub fn zenoh_config(&self) -> anyhow::Result<zenoh::config::Config> {
        let mut config = if let Some(conf_file) = &self.zenoh.config_path {
            zenoh::config::Config::from_file(conf_file).map_err(crate::ErrorWrapper::ZenohError)?
        } else {
            zenoh::config::Config::default()
        };
        config
            .connect
            .endpoints
            .extend(self.zenoh.connect.iter().cloned());
        config
            .listen
            .endpoints
            .extend(self.zenoh.listen.iter().cloned());
        Ok(config)
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ZenohConfigWrapper {
    #[serde(default)]
    pub connect: Vec<zenoh_config::EndPoint>,
    #[serde(default)]
    pub listen: Vec<zenoh_config::EndPoint>,
    #[serde(default)]
    pub config_path: Option<String>,
}
