//! Game configuration.

use glamour::Size2;

/// Initial game configuration passed to [`crate::PixelGame::run`].
///
/// There's two ways to initialize the config:
///
/// # Example
///
/// ```rust
/// # use pixel_game_lib::GameConfig;
/// GameConfig {
///   title: "My Game".to_owned(),
///   ..Default::default()
/// };
/// ```
///
/// # Example
///
/// ```rust
/// # use pixel_game_lib::GameConfig;
/// GameConfig::default().with_title("My Game");
/// ```
#[derive(Debug, Clone)]
pub struct GameConfig {
    /// Amount of pixels for the canvas.
    ///
    /// Defaults to `(320.0, 280.0)`.
    pub buffer_size: Size2,
    /// Factor applied to the buffer size for the requested window size.
    ///
    /// Defaults to `1.0`.
    pub scaling: f32,
    /// Name in the title bar.
    ///
    /// On WASM this will display as a header underneath the rendered content.
    ///
    /// Defaults to `"Pixel Game"`.
    pub title: String,
    /// Color of the viewport.
    ///
    /// The viewport is the area outside of the buffer when inside a bigger window.
    ///
    /// Defaults to `0xFF76428A` (purple).
    pub viewport_color: u32,
    /// Color of the background of the buffer.
    ///
    /// Defaults to `0xFF9BADB7` (gray).
    pub background_color: u32,
    /// Shader algorithm to use when rotating sprites.
    ///
    /// Different algorithms have different performance and aesthetic trade offs.
    ///
    /// See [`RotationAlgorithm`] for more information.
    ///
    /// Defaults to [`RotationAlgorithm::Scale3x`].
    pub rotation_algorithm: RotationAlgorithm,
}

impl GameConfig {
    /// Set the amount of pixels for the canvas.
    pub fn with_buffer_size(mut self, buffer_size: impl Into<Size2>) -> Self {
        self.buffer_size = buffer_size.into();

        self
    }

    /// Set the factor applied to the buffer size for the requested window size.
    pub fn with_scaling(mut self, scaling: f32) -> Self {
        self.scaling = scaling;

        self
    }

    /// Set the name in the title bar.
    ///
    /// On WASM this will display as a header underneath the rendered content.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();

        self
    }

    /// Set the color of the viewport.
    ///
    /// The viewport is the area outside of the buffer when inside a bigger window.
    /// Set the factor applied to the buffer size for the requested window size.
    pub fn with_viewport_color(mut self, viewport_color: u32) -> Self {
        self.viewport_color = viewport_color;

        self
    }

    /// Set the color of the background of the buffer.
    pub fn with_background_color(mut self, background_color: u32) -> Self {
        self.background_color = background_color;

        self
    }

    /// Set the shader algorithm to use when rotating sprites.
    ///
    /// Different algorithms have different performance and aesthetic trade offs.
    ///
    /// See [`RotationAlgorithm`] for more information.
    pub fn with_rotation_algorithm(mut self, rotation_algorithm: RotationAlgorithm) -> Self {
        self.rotation_algorithm = rotation_algorithm;

        self
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            buffer_size: Size2::new(320.0, 280.0),
            scaling: 1.0,
            title: "Pixel Game".to_string(),
            viewport_color: 0xFF76428A,
            background_color: 0xFF9BADB7,
            rotation_algorithm: RotationAlgorithm::Scale3x,
        }
    }
}

/// What 'upscale' shader algorithm to use for the sprite rotation.
///
/// Defaults to [`RotationAlgorithm::Scale3x`].
///
/// Here are the results from a simple test with 1000 sprites I did:
///
/// | Algorithm | Performance | Visual Quality | Texture Lookups per Pixel |
/// | --- | --- | --- | --- |
/// | [`RotationAlgorithm::Scale3x`] (default) | ~60fps | Great | 9 |
/// | [`RotationAlgorithm::Diag2x`] | ~60fps | Good | 9 |
/// | [`RotationAlgorithm::NearestNeighbor`] | ~160fps | Terrible | 1 |
/// | [`RotationAlgorithm::Scale2x`] | ~80fps | Bad | 5 |
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RotationAlgorithm {
    /// 'Upscale' with the [Scale3x](http://www.scale2x.it/) algorithm.
    ///
    /// The performance is not that great. Rotating pixel-art will look quite good.
    #[default]
    Scale3x,
    /// 'Upscale' with the [Diag2x](https://www.slimesalad.com/forum/viewtopic.php?t=8333) algorithm.
    ///
    /// Very similar but slightly worse performance than [`RotationAlgorithm::Scale2x`], reduces noisy artifacts a tiny bit.
    Diag2x,
    /// 'Upscale' with nearest-neighbor sampling.
    ///
    /// The performance is very good but will result in ugly artifacts when rotating pixel-art.
    ///
    /// Should be used when you don't plan to rotate, skew or stretch any sprites.
    NearestNeighbor,
    /// 'Upscale' with the [Scale2x](http://www.scale2x.it/) algorithm.
    ///
    /// The performance is slightly better than [`RotationAlgorithm::Scale3x`]. Visually it's very noisy. It should probably never be used unless there's a specific aesthetic you're going for.
    Scale2x,
}
