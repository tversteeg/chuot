# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.3.0...pixel-game-lib-v0.3.1) - 2023-10-21

### Other
- *(window)* re-export `winit_input_helper::WinitInputHelper` as `window::Input`

## [0.3.0](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.2.0...pixel-game-lib-v0.3.0) - 2023-10-20

### Added
- *(gui)* implement layout system
- *(sprite)* metadata in TOML
- *(assets)* add type that owns an asset or uses a ref
- base structure for gui feature
- implement font & sprite asset loading
- assets features
- default-font feature loading image from memory

### Fixed
- *(window)* WASM build
- *(deps)* update rust crate serde to 1.0.189
- *(assets)* feature flags

### Other
- fix window example
- *(lib.rs)* document features
- *(ci)* fix
- merge main
- *(ci)* generate README.md from lib.rs
- [**breaking**] remove all feature flags
- [**breaking**] remove `assets` feature flag
- [**breaking**] overhaul all feature flags
- [**breaking**] re-export less in the crate root

## [0.2.0](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.1.0...pixel-game-lib-v0.2.0) - 2023-10-02

### Added
- *(window)* expose winit input
- [**breaking**] implement WASM window
- *(window)* hide async implementation

### Other
- ignore run-wasm in release-plz
