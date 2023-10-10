# pixel-game-lib

A Rust utility library for creating pixel based games, not a game engine.

[![Build Status](https://github.com/tversteeg/pixel-game-lib/workflows/CI/badge.svg)](https://github.com/tversteeg/pixel-game-lib/actions?workflow=CI)
[![Crates.io](https://img.shields.io/crates/v/pixel-game-lib.svg)](https://crates.io/crates/pixel-game-lib)
[![Documentation](https://docs.rs/pixel-game-lib/badge.svg)](https://docs.rs/pixel-game-lib)
[![License: GPL-3.0](https://img.shields.io/crates/l/pixel-game-lib.svg)](#license)
[![Downloads](https://img.shields.io/crates/d/pixel-game-lib.svg)](#downloads)

### [Documentation](https://docs.rs/pixel-game-lib/)

<!-- cargo-rdme start -->

Utility library for games, not a game engines.

## Features

#### `default-font`

Implements [`Default`] for [`font::Font`] with a font that's embedded into memory.

#### `hot-reloading-assets` (default)

Hot-reload assets from disk when they are saved.
Has no effect on the web target.

#### `embedded-assets` (default on web)

Bake _all_ assets in the `assets/` folder in the binary.
When creating a release binary this feature flag should be enabled.

<!-- cargo-rdme end -->
