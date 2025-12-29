# GPU USAGE WAYBAR

[![Crates.io Version](https://img.shields.io/crates/v/gpu-usage-waybar?style=for-the-badge)](https://crates.io/crates/gpu-usage-waybar)
[![AUR Version](https://img.shields.io/aur/version/gpu-usage-waybar-git?style=for-the-badge)](https://aur.archlinux.org/packages/gpu-usage-waybar-git)
[![GitHub License](https://img.shields.io/github/license/polponline/gpu-usage-waybar?style=for-the-badge)](https://github.com/PolpOnline/gpu-usage-waybar/blob/master/LICENSE)

This is a simple tool I made to add GPU usage to Waybar.

It is compatible with both NVIDIA and AMD cards

# Installation

## Requirements

- For NVIDIA, you need the NVML library installed

- For AMD, the tool just uses the sysfs interface; you shouldn't need to install anything

## Installation

Install with `cargo`

```sh
cargo install gpu-usage-waybar
```

# Usage

Add a custom module to Waybar

Add a module to the top of the config specifying where the module should be positioned

```json
"modules-left": ["...", "custom/gpu-usage", "..."]
```

Add the custom module to the config

```json
"custom/gpu-usage": {
  "format": "{} {icon}", /* "{text} {icon}" for nightly users */
  "exec": "gpu-usage-waybar",
  "return-type": "json",
  "format-icons": "󰾲",
  "on-click": "ghostty -e nvtop",
}
```

The crate also has a configuration file (located at `$XDG_CONFIG_HOME/gpu-usage-waybar/config.toml`)
which can be used to set various options about the output.

Bear in mind that args passed to the command line will override the configuration file
