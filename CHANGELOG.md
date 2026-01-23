# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.26] - 2026-01-19

### Fixed
- [bbca4bf] Specify minimum rust version in Cargo.toml as in #123

## [0.1.25] - 2026-01-19

### Added

- Pattern-based output formatting with `--text-format` and `--tooltip-format` CLI options and TOML config ([#118](https://github.com/PolpOnline/gpu-usage-waybar/pull/118)) by [@Bowen951209](https://github.com/Bowen951209)
- Two-stage GPU presence detection for NVIDIA to avoid unnecessary wake-ups ([#113](https://github.com/PolpOnline/gpu-usage-waybar/pull/113)) by [@Bowen951209](https://github.com/Bowen951209)

### Changed

- **BREAKING**: Removed legacy formatting options (`text_no_memory` flag) in favor of pattern-based config ([#118](https://github.com/PolpOnline/gpu-usage-waybar/pull/118)) by [@Bowen951209](https://github.com/Bowen951209)

    If you were using `text_no_memory = true`, replace it with custom formats. For example:

    Old config:

    ```toml
    text_no_memory = true
    ```

    New config:

    ```toml
    [text]
    format = "{gpu_utilization}%"
    ```

### Fixed

- NVIDIA GPU being woken up unnecessarily during status checks ([#113](https://github.com/PolpOnline/gpu-usage-waybar/pull/113)) by [@Bowen951209](https://github.com/Bowen951209)

### Documentation

- Added Waybar nightly format specifier instructions ([#112](https://github.com/PolpOnline/gpu-usage-waybar/pull/112)) by [@nouritsu](https://github.com/nouritsu)

## [0.1.24] - 2025-07-28

Previous release.
