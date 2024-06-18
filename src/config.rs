//! Initial game configuration.

/// Initial game configuration passed to [`chuot::PixelGame::run`].
///
/// There's two ways to initialize the config:
///
/// # Example
///
/// ```rust
/// # use chuot::GameConfig;
/// GameConfig {
///     title: "My Game".to_owned(),
///     ..Default::default()
/// };
/// ```
///
/// # Example
///
/// ```rust
/// # use chuot::GameConfig;
/// GameConfig::default().with_title("My Game");
/// ```
#[derive(Debug, Clone)]
#[allow(clippy::exhaustive_structs)]
pub struct Config {
    /// Amount of horizontal pixels for the canvas.
    ///
    /// Defaults to `320.0`.
    pub buffer_width: f32,
    /// Amount of vertical pixels for the canvas.
    ///
    /// Defaults to `280.0`.
    pub buffer_height: f32,
    /// Factor applied to the buffer size for the requested window size.
    ///
    /// Defaults to `2.0`.
    pub scaling: f32,
    /// Enable vsync on the GPU.
    ///
    /// This will try to lock the framerate with the refreshrate.
    ///
    /// Defaults to `true`.
    pub vsync: bool,
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
    /// Maximum amount a single frame may take in seconds.
    ///
    /// Defaults to `1.0/4.0`.
    pub max_frame_time_secs: f32,
    /// Fixed duration in seconds a single update tick will take.
    ///
    /// Defaults to `1.0/30.0`, AKA 30 update ticks per second.
    pub update_delta_time: f32,
}

impl Config {
    /// Set the amount of horizontal pixels for the canvas.
    #[inline]
    #[must_use]
    pub fn with_buffer_width(mut self, buffer_width: f32) -> Self {
        self.buffer_width = buffer_width;

        self
    }

    /// Set the amount of horizontal pixels for the canvas.
    #[inline]
    #[must_use]
    pub fn with_buffer_height(mut self, buffer_height: f32) -> Self {
        self.buffer_height = buffer_height;

        self
    }

    /// Set the factor applied to the buffer size for the requested window size.
    #[inline]
    #[must_use]
    pub const fn with_scaling(mut self, scaling: f32) -> Self {
        self.scaling = scaling;

        self
    }

    /// Set vsync on the GPU.
    ///
    /// This will try to lock the framerate with the refreshrate.
    #[inline]
    #[must_use]
    pub const fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;

        self
    }

    /// Set the name in the title bar.
    ///
    /// On WASM this will display as a header underneath the rendered content.
    #[inline]
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();

        self
    }

    /// Set the color of the viewport.
    ///
    /// The viewport is the area outside of the buffer when inside a bigger window.
    /// Set the factor applied to the buffer size for the requested window size.
    #[inline]
    #[must_use]
    pub const fn with_viewport_color(mut self, viewport_color: u32) -> Self {
        self.viewport_color = viewport_color;

        self
    }

    /// Set the color of the background of the buffer.
    #[inline]
    #[must_use]
    pub const fn with_background_color(mut self, background_color: u32) -> Self {
        self.background_color = background_color;

        self
    }

    /// Set the shader algorithm to use when rotating sprites.
    ///
    /// Different algorithms have different performance and aesthetic trade offs.
    ///
    /// See [`RotationAlgorithm`] for more information.
    #[inline]
    #[must_use]
    pub const fn with_rotation_algorithm(mut self, rotation_algorithm: RotationAlgorithm) -> Self {
        self.rotation_algorithm = rotation_algorithm;

        self
    }

    /// Set the maximum amount a single frame may take in seconds.
    #[inline]
    #[must_use]
    pub const fn with_max_frame_time_secs(mut self, max_frame_time_secs: f32) -> Self {
        self.max_frame_time_secs = max_frame_time_secs;

        self
    }

    /// Set the duration in seconds a single update tick will take.
    #[inline]
    #[must_use]
    pub const fn with_update_delta_time(mut self, update_delta_time: f32) -> Self {
        self.update_delta_time = update_delta_time;

        self
    }
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self {
            buffer_width: 320.0,
            buffer_height: 280.0,
            scaling: 2.0,
            vsync: true,
            title: "Pixel Game".to_owned(),
            viewport_color: 0xFF76428A,
            background_color: 0xFF9BADB7,
            rotation_algorithm: RotationAlgorithm::default(),
            max_frame_time_secs: 1.0 / 4.0,
            update_delta_time: 1.0 / 30.0,
        }
    }
}

/// What 'upscale' shader algorithm to use for the sprite rotation.
///
/// Defaults to [`RotationAlgorithm::Scale3x`].
///
/// When you don't plan on rotating any sprite use [`RotationAlgorithm::NearestNeighbor`].
///
/// Here are the results from a simple test with 1000 sprites I did:
///
/// | Algorithm | Performance | Visual Quality | Texture Lookups per Pixel |
/// | --- | --- | --- | --- |
/// | [`RotationAlgorithm::CleanEdge`] | ~60fps | Great | 21 |
/// | [`RotationAlgorithm::Scale3x`] (default) | ~300fps | Good | 9 |
/// | [`RotationAlgorithm::Diag2x`] | ~400fps | Average | 9 |
/// | [`RotationAlgorithm::NearestNeighbor`] | ~460fps | Terrible | 1 |
/// | [`RotationAlgorithm::Scale2x`] | ~450fps | Bad | 5 |
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RotationAlgorithm {
    /// 'Upscale' with Torcado's [cleanEdge](https://torcado.com/cleanEdge/) algorithm.
    ///
    /// The performance is the worst of all but it looks very good when rotating pixel-art.
    CleanEdge,
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
