mod config;

pub use config::AppConfig;

use thiserror::Error;
use tracing::*;
use tracing_subscriber::prelude::*;

pub fn setup_tracing(verbosity_level: u8) {
    let filter = match verbosity_level {
        0 => tracing::level_filters::LevelFilter::INFO,
        1 => tracing::level_filters::LevelFilter::DEBUG,
        2 => tracing::level_filters::LevelFilter::TRACE,
        _ => tracing::level_filters::LevelFilter::TRACE,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(filter.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[derive(Error, Debug)]
pub enum ErrorWrapper {
    #[error("Zenoh error {0:?}")]
    ZenohError(#[from] zenoh::Error),
}

pub fn get_simple_install_topic(device_name: &str) -> String {
    format!("ez-cd/simple_install/{}/dpkg", device_name)
}
