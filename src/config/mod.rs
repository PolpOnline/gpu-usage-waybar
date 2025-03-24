pub mod structs;

use std::{path::PathBuf, sync::OnceLock};

use color_eyre::{eyre::eyre, Result};
use etcetera::{base_strategy::Xdg, BaseStrategy};

use crate::config::structs::ConfigFile;

pub static XDG_DIR: OnceLock<Xdg> = OnceLock::new();

fn get_xdg_dir() -> &'static Xdg {
    XDG_DIR.get_or_init(|| Xdg::new().expect("Failed to get XDG directory"))
}

fn get_config_dir() -> PathBuf {
    get_xdg_dir().config_dir()
}

const EXAMPLE_CONFIG: &str = include_str!("../../config.example.toml");

pub fn get_or_init_config() -> Result<ConfigFile> {
    let config_dir = get_config_dir();

    let config_path = config_dir.join("gpu_usage_waybar.toml");

    if !config_path.exists() {
        std::fs::write(&config_path, EXAMPLE_CONFIG)?;
    }

    let config_str = std::fs::read_to_string(&config_path)?;

    let config: ConfigFile =
        toml::de::from_str(&config_str).map_err(|e| eyre!("Failed to parse config file: {}", e))?;

    Ok(config)
}
