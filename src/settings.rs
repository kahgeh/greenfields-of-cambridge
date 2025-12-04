use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;
use std::io;
use std::sync::{Arc, OnceLock};
use thiserror::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct Metadata {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogSettings {
    pub level: String,
    pub format: String,
}

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("Failed to determine the current directory")]
    GetCurrentDir(#[from] io::Error),
    #[error("Failed to build configuration")]
    ConfigBuild(#[from] ConfigError),
    #[error("Configuration validation failed: {0}")]
    Validation(String),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub log: LogSettings,
    pub metadata: Metadata,
}

// Global settings instance
static SETTINGS: OnceLock<Arc<Settings>> = OnceLock::new();

impl Settings {
    pub fn initialize() -> Result<(), SettingsError> {
        let settings = Self::load()?;

        if SETTINGS.set(Arc::new(settings)).is_err() {
            return Err(SettingsError::Validation("Settings already initialized".to_string()));
        }

        Ok(())
    }

    pub fn get() -> &'static Arc<Settings> {
        SETTINGS.get().expect("Settings not initialized")
    }

    pub fn load() -> Result<Self, SettingsError> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let configuration_directory = std::path::Path::new(manifest_dir).join("config");
        let run_environment = env::var("run_environment").unwrap_or_else(|_| "local".to_string());

        let config = Self::build_config(&configuration_directory, &run_environment)?;
        let settings: Settings = config.try_deserialize().map_err(SettingsError::ConfigBuild)?;

        Ok(settings)
    }

    fn build_config(
        configuration_directory: &std::path::Path,
        run_environment: &str,
    ) -> Result<Config, SettingsError> {
        let mut builder = Config::builder();

        builder = builder
            .set_default("metadata.name", env!("CARGO_PKG_NAME"))?
            .set_default("metadata.version", env!("CARGO_PKG_VERSION"))?;

        builder = builder.add_source(File::from(configuration_directory.join("default.toml")));

        let env_config_path = configuration_directory.join(format!("{}.toml", run_environment));
        builder = builder.add_source(File::from(env_config_path).required(false));

        builder = builder.add_source(
            Environment::with_prefix("app")
                .separator("__")
                .try_parsing(true),
        );

        builder.build().map_err(SettingsError::ConfigBuild)
    }
}