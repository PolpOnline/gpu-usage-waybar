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

Add the custom module to the config, use

```jsonc
"custom/gpu-usage": {
  "format": "{} {icon}", /* "{text} {icon}" for nightly users */
  "exec": "gpu-usage-waybar",
  "return-type": "json",
  "format-icons": "󰾲",
  "on-click": "ghostty -e nvtop",
}
```

# Configurations

The crate has a configuration file (located at `$XDG_CONFIG_HOME/gpu_usage_waybar.toml`)
which can be used to set various options about the output.

You can specify the output format in the config file as:

```toml
[text]
format = "{gpu_utilization}%|{mem_utilization}%"

[tooltip]
format = """GPU: {gpu_utilization}%
MEM USED: {mem_used}/{mem_total} MiB ({mem_utilization}%)
MEM R/W: {mem_rw}%
DEC: {decoder_utilization}%
ENC: {encoder_utilization}%
TEMP: {temperature}°C
POWER: {power}W
PSTATE: {p_state}
PLEVEL: {p_level}
FAN SPEED: {fan_speed}%
TX: {tx} MiB/s
RX: {rx} MiB/s"""
```

You can also set them with CLI args.
<details>
<summary>Available fields</summary>

| Field name | Description | Unit | AMD | NVIDIA |
| :--- | :--- | :--- | :---: | :---: |
| `gpu_utilization` | GPU utilization | % | ✅ | ✅ |
| `mem_used` | Memory used in MiB | MiB | ✅ | ✅ |
| `mem_total` | Total memory in MiB | MiB | ✅ | ✅ |
| `mem_rw` | Memory data bus utilization | % | ❌ | ✅ |
| `mem_utilization` | Memory utilization | % | ✅ | ✅ |
| `decoder_utilization` | Decoder utilization | % | ❌ | ✅ |
| `encoder_utilization` | Encoder utilization | % | ❌ | ✅ |
| `temperature` | Temperature in degrees Celsius | °C | ✅ | ✅ |
| `power` | Power usage in Watts | W | ✅ | ✅ |
| `p_state` | (NVIDIA) Performance state | NVIDIA performance state | ❌ | ✅ |
| `p_level` | (AMD) Performance Level | AMD performance level | ✅ | ❌ |
| `fan_speed` | Fan speed in percent | % | ✅ | ✅ |
| `tx` | PCIe TX throughput in MiB/s | MiB/s | ❌ | ✅ |
| `rx` | PCIe RX throughput in MiB/s | MiB/s | ❌ | ✅ |

</details>

Bear in mind that args passed to the command line will override the configuration file
