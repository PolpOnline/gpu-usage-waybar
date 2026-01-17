pub mod structs;

use color_eyre::{Result, eyre::eyre};
use etcetera::{BaseStrategy, base_strategy::Xdg};

use crate::config::structs::ConfigFile;

const EXAMPLE_CONFIG: &str = include_str!("../../config.example.toml");

pub fn get_or_init_config() -> Result<ConfigFile> {
    let config_dir = Xdg::new()?.config_dir();

    let config_path = config_dir.join("gpu_usage_waybar.toml");

    if !config_path.exists() {
        std::fs::write(&config_path, EXAMPLE_CONFIG)?;
    }

    let config_str = std::fs::read_to_string(&config_path)?;

    let config: ConfigFile =
        toml::de::from_str(&config_str).map_err(|e| eyre!("Failed to parse config file: {}", e))?;

    Ok(config)
}
