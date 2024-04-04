# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0-alpha.5](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.9.0-alpha.4...pixel-game-lib-v0.9.0-alpha.5) - 2024-04-04

### Added
- *(sprite)* add `SpriteContext::draw_multiple_translated` for a performant way of drawing an iterator of offsets
- *(profiler)* profile heap allocations for rendering and tick call
- *(context)* [**breaking**] use ergonomic zero cost abstraction for text drawing similar to sprite refactor
- *(assets)* export different types that can be used for loading assets

### Other
- *(context)* [**breaking**] move `Context::draw_sprite` and `Context::draw_text` to `Context::sprite|text::draw`
- *(cargo)* only include 'src/' folder for a smaller crate size
- *(sprite)* [**breaking**] use zero-cast abstraction with helper structs for more ergonic sprite drawing
- *(assets)* [**breaking**] move `crate::asset` and `crate::asset_owned` to `Context` as methods

## [0.9.0-alpha.4](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.9.0-alpha.3...pixel-game-lib-v0.9.0-alpha.4) - 2024-03-29

### Added
- *(config)* allow setting vsync in `GameConfig`
- *(config)* add builder methods for `GameConfig` fields
- *(context)* add `Context::sprite_raw_pixels`
- *(context)* add `Context::sprite_size`

### Other
- *(graphics)* refactor structure of communicating with the GPU for future addition of GPU profiler

## [0.9.0-alpha.3](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.9.0-alpha.2...pixel-game-lib-v0.9.0-alpha.3) - 2024-03-27

### Added
- *(graphics)* add `Context::update_sprite_pixels` for updating regions of pixels on an already uploaded sprites
- *(graphics)* allow configuring `RotationAlgorithm` in config

### Fixed
- *(deps)* update rust crates glam and glamour
- *(deps)* update rust crate egui to 0.27.0
- *(deps)* downgrade 'glam' so 'glamour' can upgrade

### Other
- *(readme)* show different RotSprite upscale options in README.md
- *(project)* [**breaking**] rename `WindowConfig` to `GameConfig`
- *(web)* [**breaking**] improve performance on Web builds by using canvas scaling
- *(graphics)* [**breaking**] skip downscale render pass when buffer fits inside window
- *(project)* [**breaking**] make some internal public items private, improve documentation

## [0.9.0-alpha.2](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.9.0-alpha.1...pixel-game-lib-v0.9.0-alpha.2) - 2024-03-24

### Added
- *(font)* [**breaking**] add `Context::draw_text` which loads a bitmap font

### Fixed
- *(deps)* update rust crate glam to 0.27.0
- *(graphics)* repair RotSprite shader
- *(graphics)* properly embed texture atlas info in shader

### Other
- *(context)* improve performance by switching from `Arc<Mutex<..>>` to `Rc<RefCell<..>>`
- *(graphics)* pack all sprites into a single texture atlas

## [0.9.0-alpha.1](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.9.0-alpha...pixel-game-lib-v0.9.0-alpha.1) - 2024-03-21

### Fixed
- *(build)* fix compilation for WASM target
- *(window)* handle exit event

### Other
- *(project)* [**breaking**] replace `PixelGame::update` & `PixelGame::render` with singular `PixelGame::tick`, change vek library to glam and glamour

## [0.9.0-alpha](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.8.0...pixel-game-lib-v0.9.0-alpha) - 2024-03-20

### Added
- *(profile)* Allow `in-game-profiler` feature flag for showing a profiler overlay, use proper color space for background and viewport colors
- *(window)* [**breaking**] add `background_color` and `viewport_color` options to `WindowConfig`
- *(graphics)* update Scale2X to Scale3X with proper sampling
- *(graphics)* use Scale2X for upscaling the sprites
- *(graphics)* create letterbox with in the future integer scaling for the buffer
- *(graphics)* expose `RenderContext::draw_sprite_rotated` which also allows future 2D scaling and skewing
- *(graphics)* make `Sprite` a GPU instanced rendering pipeline
- *(bitmap)* add `from_bitvec` and `clone_with_padding` to `BitMap`
- *(bitmap)* implement floodfill
- *(bitmap)* allow toggling a single value
- *(bitmap)* add `BitMap` for 2D masking operations on buffers

### Fixed
- *(graphics)* [**breaking**] Use `Into<Rotation>` as argument for `draw_sprite_rotated`
- *(wasm)* build again on WASM with WebGL2 and cleanup
- *(project)* use proper relative mouse coordinates and cleanup small pieces of code and examples
- *(graphics)* use proper alpha blending for components
- *(graphics)* properly render output buffer in viewport
- *(graphics)* render pixel graphics to proper buffer size with upscaling and downscaling preparing for rotsprite like algorithms
- *(deps)* update rust crate winit to 0.29.15
- *(deps)* update rust crate bytemuck to 1.15.0
- *(deps)* update rust crate miette to 7.2.0
- *(deps)* update rust crate blit to 0.8.5

### Other
- *(project)* cleanup and prepare for beta release
- *(ci)* fix test and wasm-build step
- *(project)* [**breaking**] make `window` function private, batch render calls in `render` trait method with `RenderContext`
- *(project)* [**breaking**] remove gui, reorganize most components related to rendering
- *(window)* [**breaking**] start replacing CPU based pixel renderer with GPU based one

## [0.8.0](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.7.0...pixel-game-lib-v0.8.0) - 2024-03-06

### Added
- *(physics)* add square, triangle and circle collider shapes
- *(audio)* `audio` crate feature for playing audio, based on the Kira crate

### Fixed
- *(deps)* update rust crate winit to 0.29.14
- *(deps)* update rust-wasm-bindgen monorepo
- *(ci)* install audio dependency in CI
- *(window)* [**breaking**] re-export proper window types for the input helper
- *(deps)* update rust crate winit_input_helper to 0.16.0
- *(audio)* hide module definition behind feature flag
- *(canvas)* ensure line drawing doesn't go out of bounds
- *(state)* use proper aliased input type

### Other
- *(ci)* only test on Linux and run check on the other platforms

## [0.7.0](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.6.1...pixel-game-lib-v0.7.0) - 2024-03-03

### Added
- *(state)* create a `PixelGame` trait which simplifies setting up a new game with a window

### Fixed
- *(deps)* update rust crate winit to 0.29.13
- *(assets)* [**breaking**] improve ergonomics of `asset` and `asset_owned` by making the path an anonymous generic
- *(deps)* update rust crate assets_manager to 0.11.3

### Other
- *(example)* fix 'physics' import

## [0.6.1](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.6.0...pixel-game-lib-v0.6.1) - 2024-02-28

### Other
- *(window)* perform color conversion on GPU with a custom shader instead of on CPU
- *(window)* use `Rc` instead of `Arc` for the window handler
- *(canvas)* improve `draw_line` performance by switching from 'line_drawing' to the 'clipline' crate

## [0.6.0](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.5.0...pixel-game-lib-v0.6.0) - 2024-02-26

### Added
- *(gui)* [**breaking**] embed default GUI elements behind feature flag `default-gui`
- *(dialogue)* implement dialogue feature based on Yarn Spinner

### Fixed
- *(deps)* update rust crate winit to 0.29.11

### Other
- *(ci)* fix WASM build and test out of space
- *(example)* fix dialogue example buttons
- *(example)* draw options as GUI buttons

## [0.5.0](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.4.7...pixel-game-lib-v0.5.0) - 2024-02-23

### Fixed
- *(sprite)* take offset into account when drawing
- *(deps)* update rust crate image to 0.24.9

### Other
- *(serde)* [**breaking**] add `deny_unknown_fields` to all items implementing `Deserialize`

## [0.4.7](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.4.6...pixel-game-lib-v0.4.7) - 2024-02-20

### Added
- *(canvas)* add `draw_circle`, `draw_scanline` and improve `draw_circle_outline` to/on `Canvas`

### Fixed
- *(deps)* update rust crate serde to 1.0.197

### Other
- *(deps)* update taffy to 0.4

## [0.4.6](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.4.5...pixel-game-lib-v0.4.6) - 2024-02-18

### Added
- *(canvas)* add `draw_circle_outline` to `Canvas`
- *(canvas)* add `draw_quad` and `draw_triangle` to `Canvas`

## [0.4.5](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.4.4...pixel-game-lib-v0.4.5) - 2024-02-16

### Fixed
- *(deps)* update rust crate miette to 7.1.0
- *(deps)* update rust crate game-loop to 1.1.0
- *(deps)* update rust-wasm-bindgen monorepo
- *(deps)* update rust crate miette to v7
- *(deps)* update rust crate miette to v6
- *(deps)* update rust crate tokio to 1.36.0
- *(deps)* update rust crate serde to 1.0.196

## [0.4.4](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.4.3...pixel-game-lib-v0.4.4) - 2024-01-26

### Fixed
- *(deps)* update rust crate assets_manager to 0.11.2
- *(deps)* update rust crate parry2d-f64 to 0.13.6
- *(deps)* update rust crate puffin to 0.19.0
- *(deps)* update rust crate winit_input_helper to 0.15.3
- *(deps)* update rust crate game-loop to 1.0.1
- *(deps)* update rust crate winit to 0.29.10
- *(deps)* update rust crate image to 0.24.8
- *(deps)* update rust crate winit_input_helper to 0.15.2
- *(deps)* update rust-wasm-bindgen monorepo
- *(deps)* update rust crate serde to 1.0.195
- *(deps)* update rust crate winit to 0.29.9
- *(deps)* update rust crate serde to 1.0.194
- *(deps)* update rust crate winit to 0.29.8
- *(deps)* update rust crate winit to 0.29.7
- *(deps)* update rust crate winit to 0.29.6

### Other
- *(deps)* update swatinem/rust-cache action to v2.7.3
- *(deps)* update swatinem/rust-cache action to v2.7.2

## [0.4.3](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.4.2...pixel-game-lib-v0.4.3) - 2023-12-23

### Fixed
- *(deps)* update rust crate winit to 0.29.5
- *(deps)* update rust crate tokio to 1.35.1
- *(deps)* update rust crate derive-where to 1.2.7

## [0.4.2](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.4.1...pixel-game-lib-v0.4.2) - 2023-12-13

### Added
- *(canvas)* add unoptimized 'draw_line' method

### Fixed
- *(deps)* update rust crate puffin to 0.18.1
- *(deps)* update rust crate tokio to 1.35.0
- *(deps)* update rust crate derive-where to 1.2.6
- *(deps)* update rust-wasm-bindgen monorepo

## [0.4.1](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.4.0...pixel-game-lib-v0.4.1) - 2023-11-26

### Fixed
- *(deps)* update rust crate winit to 0.29.4
- *(deps)* update rust crate puffin to 0.18.0
- *(deps)* update rust crate serde to 1.0.193

### Other
- set MSRV

## [0.4.0](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.3.1...pixel-game-lib-v0.4.0) - 2023-11-14

### Fixed
- *(deps)* update rust crate winit_input_helper to 0.15.1
- *(deps)* update rust crate hecs to 0.10.4
- *(deps)* update rust crate tokio to 1.34.0
- *(deps)* update rust crate serde to 1.0.192
- *(assets)* always embed on web
- *(deps)* update rust-wasm-bindgen monorepo
- *(deps)* update rust crate serde to 1.0.190

### Other
- *(deps)* [**breaking**] update winit to 0.29

## [0.3.1](https://github.com/tversteeg/pixel-game-lib/compare/pixel-game-lib-v0.3.0...pixel-game-lib-v0.3.1) - 2023-10-25

### Added
- *(physics)* add XPBD-based physics engine
- *(gui)* add label widget

### Fixed
- *(math)* conditionally implement From<Isometry2> for Iso
- *(canvas)* set_pixel coordinate calculation
- *(gui)* enforce type soundness with a reference type for each widget

### Other
- *(ci)* test every feature instead of all combinations of features
- *(example)* spawn objects on mouse click in physics example
- *(deps)* update swatinem/rust-cache action to v2.7.1
- *(window)* re-export winit_input_helper::WinitInputHelper as window::Input

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
