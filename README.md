# pixel-game-lib

[![Build Status](https://github.com/tversteeg/pixel-game-lib/workflows/CI/badge.svg)](https://github.com/tversteeg/pixel-game-lib/actions?workflow=CI)
[![Crates.io](https://img.shields.io/crates/v/pixel-game-lib.svg)](https://crates.io/crates/pixel-game-lib)
[![Documentation](https://docs.rs/pixel-game-lib/badge.svg)](https://docs.rs/pixel-game-lib)
[![License: GPL-3.0](https://img.shields.io/crates/l/pixel-game-lib.svg)](#license)
[![Downloads](https://img.shields.io/crates/d/pixel-game-lib.svg)](#downloads)

### [Documentation](https://docs.rs/pixel-game-lib/)

<!-- cargo-rdme start -->

AGPL licensed and opinionated game engine for pixel-art games.

#### Features

- Pixel-perfect pixel art rendering with built-in rotsprite rotation shader.
- Window creation with independent update and render game loop.
- Hot-reloadable asset management.
- Sprite loading.
- Dialogue scripting system.
- Audio playback.
- In game profiler GUI.
- Simple bitmap font drawing.

#### Usage

Using this crate is quite simple, there is a single trait [`PixelGame`] with a single required function, [`PixelGame::tick`] that needs to be implemented for a state.

```rust
use pixel_game_lib::{PixelGame, Context, WindowConfig};

struct MyGame;

impl PixelGame for MyGame {
  fn tick(&mut self, ctx: Context) {
    // ..
  }
}

// In main
let game = MyGame;

game.run(WindowConfig::default())?;
```

#### Feature Flags

All major feature flags are enabled by default, I would recommend installing `pixel_game_lib` with `default-features = false` and adding the required features as needed.

```sh
cargo add pixel_game_lib --no-default-features
```

##### `hot-reloading-assets` (default)

Hot-reload assets from disk when they are saved.
Has no effect on the web target.

##### `embedded-assets` (default on web)

Bake _all_ assets in the `assets/` folder in the binary.
When creating a release binary this feature flag should be enabled.

##### `dialogue` (default)

A thin wrapper around [Yarn Spinner](https://www.yarnspinner.dev/).
Allows creating hot-reloadable dialogue systems.

##### `audio` (default)

A thin wrapper around [Kira](https://docs.rs/kira/latest/kira/).
Play sounds and music files which can be hot-reloadable using assets.

To keep the binary and compile-times small only `.ogg` audio files are supported.

###### Requirements

On Linux you need to install `asound2-dev`:

```sh
sudo apt install libasound2-dev
```

##### `in-game-profiler` (default)

A profiler window overlay, implemented with [puffin_egui](https://docs.rs/puffin_egui/latest/puffin_egui/).

Other profiling methods in your game can also be implemented, the [profiling](https://docs.rs/profiling/latest/profiling/) crate is enabled even when this feature flag is disabled.

<!-- cargo-rdme end -->

#### Credits

- [gtoknu](https://www.shadertoy.com/view/4l2SRz) for the branchless scale2x shader.
- [@damieng](https://damieng.com/typography/zx-origins/beachball/) for the font behind the `default-font` feature.
- [KenneyNL](https://kenney.nl/assets/ui-audio) for the audio sample in the example.
