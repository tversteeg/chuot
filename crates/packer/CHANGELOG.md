# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2024-11-17

### Bug Fixes

#### Deps

- Update rust crate [wgpu](https://crates.io/crates/wgpu) to v23 by @renovate[bot]


### Performance

#### Compile-time

- Improve compilation speed by creating inner functions for generics by @tversteeg


### Styling

#### Fmt

- Use 2024 style edition for `cargo fmt` by @tversteeg

[0.2.1]: https://github.com/tversteeg/chuot/compare/0.2.0..0.2.1

<!-- generated by git-cliff -->
## [0.2.0] - 2024-07-05

### Refactor

#### Crate

- [**breaking**] Redesign crate structure ([#83](https://github.com/tversteeg/chuot/pull/83))


### Miscellaneous Tasks

#### Deps

- Update patch versions of crates ([#86](https://github.com/tversteeg/chuot/pull/86))

[0.2.0]: https://github.com///compare/0.1.1..0.2.0

<!-- generated by git-cliff -->
## [0.1.1] - 2024-05-22

### Features

#### Packer

- Add `Packer::with_existing_rectangles_iter` for re-building packed atlasses


### Performance

#### Context

- Decrease asset allocations by only taking owned ID references at the last moment, also decreases monomorphization


### Styling

#### Clippy

- Enforce very pedantic code style

[0.1.1]: https://github.com///compare/0.1.0..0.1.1

<!-- generated by git-cliff -->
