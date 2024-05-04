# 🐭 Chuột

[![Build Status](https://github.com/tversteeg/chuot/workflows/CI/badge.svg)](https://github.com/tversteeg/chuot/actions?workflow=CI)
[![Crates.io](https://img.shields.io/crates/v/chuot.svg)](https://crates.io/crates/chuot)
[![Documentation](https://docs.rs/chuot/badge.svg)](https://docs.rs/chuot)
[![License: AGPL-3.0](https://img.shields.io/crates/l/chuot.svg)](#license)
[![Dependency Status](https://deps.rs/repo/github/tversteeg/chuot/status.svg)](https://deps.rs/repo/github/tversteeg/chuot)
[![Downloads](https://img.shields.io/crates/d/chuot.svg)](#downloads)

### [Website](https://tversteeg.nl/chuot/)

<!-- cargo-rdme start -->

AGPL licensed and opinionated game engine for 2D pixel-art games.

#### Features

- Pixel-perfect pixel art rendering with built-in rotsprite rotation shader.
- Window creation with independent update and render game loop.
- Hot-reloadable assets, seeing your assets update live in the game when you save them is a great boost in productivity for quickly iterating on ideas.
- Single-binary, all non-texture assets should be embedded directly, and textures should be diced into a single atlas map embedded in the binary when deploying.
- Simple bitmap font drawing.
- Dialogue scripting system.
- OGG audio playback.
- In game CPU & memory profiler GUI.

#### Goals

- Ergonomic API with a focus on quickly creating small games, especially for game jams.
- Reasonable performance, drawing thousands of animated sprites at the same time shouldn't be a problem.
- Proper web support, it should be very easy to bundle as WASM for the web.

#### Non-Goals

- An ECS (Entity component system), although an ECS architecture is great for cache locality and performance, I feel that it's overkill for most small games. Nothing is stopping you to add your own on top of this engine if that's what you want though!
- 3D, this engine is only for 2D pixel art.
- Vector graphics, similar to the above, this engine is focused specifically on pixel art with lower resolutions.
- Reinventing the wheel for everything, when there's a proper crate with good support I prefer to use that instead of creating additional maintainer burden.
- Support all possible file formats, this bloats the engine.

#### Usage

Using this crate is quite simple, there is a single trait [`PixelGame`] with two required functions, [`PixelGame::update`] and [`PixelGame::render`], that need to be implemented for a game state object.

```rust
use chuot::{PixelGame, Context, GameConfig};

struct MyGame;

impl PixelGame for MyGame {
  fn update(&mut self, ctx: Context) {
    // ..
  }

  fn render(&mut self, ctx: Context) {
    // ..
  }
}

// In main

let game = MyGame;

game.run(chuot::load_assets!(), GameConfig::default())?;
```

#### Feature Flags

All major feature flags are enabled by default, I would recommend installing `chuot` with `default-features = false` and adding the required features as needed.

```sh
cargo add chuot --no-default-features
```

##### `embed-assets`

Embed all assets into the binary when building.

_Must_ be enabled when building for the web.
If disabled all assets will be loaded from disk.

This will dice all PNG assets into a single tiny optimized PNG atlas.
On startup this diced atlas will be efficiently uploaded to the GPU as a single bigger atlas, which will be used for all static sprites.

##### `hot-reload-assets`

Hot-reload assets from disk when they are saved.
Has no effect on the web target.
If disabled _all_ assets will be baked into the binary.

##### `dialogue` (default)

A thin wrapper around [Yarn Spinner](https://www.yarnspinner.dev/).
Allows creating hot-reloadable dialogue systems.

###### Requirements

On Linux you need to install `asound2-dev`:

```sh
sudo apt install libasound2-dev
```

##### `in-game-profiler`

A profiler window overlay, implemented with [puffin_egui](https://docs.rs/puffin_egui/latest/puffin_egui/).

Other profiling methods in your game can also be implemented, the [profiling](https://docs.rs/profiling/latest/profiling/) crate is enabled even when this feature flag is disabled.

#### Example

This example will show a window with a counter that's incremented when pressing the left mouse button[^left-mouse].
The counter is rendered as text[^text] loaded from a font in the top-left corner.
When the 'Escape' key is pressed[^escape-key] the game will exit and the window will close.

```rust
use chuot::{PixelGame, Context, GameConfig, MouseButton, KeyCode, glamour::Vector2};

/// Object holding all game state.
struct MyGame {
  /// A simple counter we increment by clicking on the screen.
  counter: u32,
}

impl PixelGame for MyGame {
  fn update(&mut self, ctx: Context) {
    // ^1
    // Increment the counter when we press the left mouse button
    if ctx.mouse_pressed(MouseButton::Left) {
      self.counter += 1;
    }

    // ^3
    // Exit the game if 'Escape' is pressed
    if ctx.key_pressed(KeyCode::Escape) {
      ctx.exit();
    }
  }

  fn render(&mut self, ctx: Context) {
    // ^2
    // Display the counter with a font called 'font' automatically loaded from the `assets/` directory
    // It will be shown in the top-left corner
    ctx.text("font", &format!("Counter: {}", self.counter)).draw();
  }
}

// In main

// Initialize the game state
let game = MyGame { counter: 0 };

// Run the game until exit is requested
game.run(chuot::load_assets!(), GameConfig::default().with_title("My Game"))?;
```

[^left-mouse]: [`Context::mouse_pressed`]
[^text]: [`Context::text`]
[^escape-key]: [`Context::key_pressed`]

<!-- cargo-rdme end -->

#### Rotation Algorithms

In the library it's possible to choose between multiple upscale implementations for the single-pass RotSprite algorithm, see the Rust documentation for more information:

##### Nearest Neighbor

This doesn't apply any extra rotation effects.

![Nearest Neighbor](./img/nearestneighbor.png)

##### cleanEdge

![cleanEdge](./img/cleanedge.png)

##### Scale3x (default)

![Scale3x](./img/scale3x.png)

##### Diag2x

![Diag2x](./img/diag2x.png)

##### Scale2x

![Scale2x](./img/scale2x.png)

#### Credits

- [Torcado](https://torcado.com/cleanEdge/) for the awesome cleanEdge algorithm.
- [gtoknu](https://www.shadertoy.com/view/4l2SRz) for the branchless scale2x shader.
- [@damieng](https://damieng.com/typography/zx-origins/beachball/) for the font behind the `default-font` feature.
- [KenneyNL](https://kenney.nl/assets/ui-audio) for the audio sample in the example.
