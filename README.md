# pixel-game-lib

[![Build Status](https://github.com/tversteeg/pixel-game-lib/workflows/CI/badge.svg)](https://github.com/tversteeg/pixel-game-lib/actions?workflow=CI)
[![Crates.io](https://img.shields.io/crates/v/pixel-game-lib.svg)](https://crates.io/crates/pixel-game-lib)
[![Documentation](https://docs.rs/pixel-game-lib/badge.svg)](https://docs.rs/pixel-game-lib)
[![License: GPL-3.0](https://img.shields.io/crates/l/pixel-game-lib.svg)](#license)
[![Downloads](https://img.shields.io/crates/d/pixel-game-lib.svg)](#downloads)

### [Documentation](https://docs.rs/pixel-game-lib/)

<!-- cargo-rdme start -->

Game engine with library features that can be used standalone.

#### Features

- Window creation with game loop and pixel buffer.
- Asset management.
- Bitmap font drawing.
- Sprite loading.
- Simple GUI.
- Physics engine.
- Audio playback.

#### Feature Flags

###### `default-font`

Implements [`Default`] for [`font::Font`] with a font that's embedded into memory.

###### `default-gui`

Implements [`Default`] for different GUI elements with a images embedded into memory.

###### `hot-reloading-assets` (default)

Hot-reload assets from disk when they are saved.
Has no effect on the web target.

###### `embedded-assets` (default on web)

Bake _all_ assets in the `assets/` folder in the binary.
When creating a release binary this feature flag should be enabled.

###### `physics`

Enable the 2D XPBD-based physics engine.

###### `dialogue`

A thin wrapper around [Yarn Spinner](https://www.yarnspinner.dev/).
Allows creating hot-reloadable dialogue systems.

###### `audio`

A thin wrapper around [Kira](https://docs.rs/kira/latest/kira/).
Play sounds and music files which can be hot-reloadable using assets.

To keep the binary and compile-times small only `.ogg` audio files are supported.

####### Requirements

On Linux you need to install `asound2-dev`:

```sh
sudo apt install libasound2-dev
```

<!-- cargo-rdme end -->

#### Credits

- [@damieng](https://damieng.com/typography/zx-origins/beachball/) for the font behind the `default-font` feature.
- [KenneyNL](https://kenney.nl/assets/ui-audio) for the audio sample in the example.
