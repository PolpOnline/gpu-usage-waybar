# GPU USAGE WAYBAR

[![crates.io](https://img.shields.io/crates/v/gpu-usage-waybar.svg)](https://crates.io/crates/gpu-usage-waybar)

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
  "format": "{} {icon}",
  "exec": "gpu-usage-waybar",
  "return-type": "json",
  "format-icons": "󰾲",
  "on-click": "kitty nvtop",
}
```
