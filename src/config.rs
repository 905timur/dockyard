use std::path::PathBuf;
use anyhow::{Context, Result};
use config::{Config, File};
use directories::ProjectDirs;
use std::fs;
use std::io::Write;

use crate::types::AppConfig;

pub fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "dockyard", "dockyard")
        .context("Failed to determine project directories")?;
    let config_dir = proj_dirs.config_dir();
    
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }
    
    Ok(config_dir.join("config.toml"))
}

pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        // Create default config
        let default_config = AppConfig::default();
        save_config(&default_config)?;
        return Ok(default_config);
    }

    let settings = Config::builder()
        .add_source(File::from(config_path))
        .build()?;

    settings.try_deserialize::<AppConfig>().context("Failed to parse configuration")
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_path()?;
    let toml_string = toml::to_string_pretty(config)?;
    
    let mut file = fs::File::create(config_path)?;
    file.write_all(toml_string.as_bytes())?;
    
    Ok(())
}
