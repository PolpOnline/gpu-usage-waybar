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

# Configuration

The crate has a configuration file (located at `$XDG_CONFIG_HOME/gpu_usage_waybar.toml`)
which can be used to set various options about the output.

You can specify the output format in the config file as:

```toml
[text]
format = "{gpu_utilization}%|{mem_utilization}%"

[tooltip]
format = """GPU: {gpu_utilization}%
MEM USED: {mem_used:MiB.0}/{mem_total:MiB} MiB ({mem_utilization}%)
MEM R/W: {mem_rw}%
DEC: {decoder_utilization}%
ENC: {encoder_utilization}%
TEMP: {temperature:c}°C
POWER: {power:w}W
PSTATE: {p_state}
PLEVEL: {p_level}
FAN SPEED: {fan_speed}%
TX: {tx:MiB.3} MiB/s
RX: {rx:MiB.3} MiB/s"""
```

- You can specify a unit using `:unit`,
  e.g., `temperature:c` for Celsius or `temperature:f` for Fahrenheit.
  Available units are listed in the table below.
- You can specify decimal places using `.places`, e.g., `temperature:f.2`.
  > [!NOTE]
  > You can only specify decimal places for fields with configurable units.
- The text format defaults to `"{gpu_utilization}%|{mem_utilization}%"`. 
- The tooltip defaults to all fields that are supported by your GPU if not customized. 

You can also set the output format with CLI args using `--text-format` and `--tooltip-format`.
<details>
<summary>Available fields</summary>

| Field name | Description | Unit | AMD | NVIDIA |
| :--- | :--- | :--- | :---: | :---: |
| `gpu_utilization` | GPU utilization | % | ✅ | ✅ |
| `mem_used` | Memory used in MiB | [Memory units](#memory-units) | ✅ | ✅ |
| `mem_total` | Total memory in MiB | [Memory units](#memory-units) | ✅ | ✅ |
| `mem_rw` | Memory data bus utilization | % | ❌ | ✅ |
| `mem_utilization` | Memory utilization | % | ✅ | ✅ |
| `decoder_utilization` | Decoder utilization | % | ❌ | ✅ |
| `encoder_utilization` | Encoder utilization | % | ❌ | ✅ |
| `temperature` | Temperature | c, f, k | ✅ | ✅ |
| `power` | Power usage | w, kw | ✅ | ✅ |
| `p_state` | (NVIDIA) Performance state | NVIDIA performance state | ❌ | ✅ |
| `p_level` | (AMD) Performance Level | AMD performance level | ✅ | ❌ |
| `fan_speed` | Fan speed in percent | % | ✅ | ✅ |
| `tx` | PCIe TX throughput in MiB/s | [Memory units](#memory-units) | ❌ | ✅ |
| `rx` | PCIe RX throughput in MiB/s | [Memory units](#memory-units) | ❌ | ✅ |

</details>

<details id="memory-units">
<summary>Memory units</summary>
Supported units: KiB, MiB, GiB, KB, MB, GB, Kib, Mib, Gib, Kb, Mb, Gb.
</details>

Bear in mind that args passed to the command line will override the configuration file
