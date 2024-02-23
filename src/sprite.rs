//! Blittable sprite definitions.
//!
//! Can be loaded as an asset when the `asset` feature flag is set.

use std::borrow::Cow;

use assets_manager::{loader::Loader, AnyCache, Asset, BoxedError, Compound, SharedString};
use blit::{slice::Slice, Blit, BlitBuffer, BlitOptions, ToBlitBuffer};
use image::ImageFormat;
use miette::{Context, IntoDiagnostic};
use serde::Deserialize;
use vek::{Extent2, Vec2};

use crate::canvas::Canvas;

/// Sprite that can be drawn on the  canvas.
#[derive(Debug)]
pub struct Sprite {
    /// Pixels to render.
    sprite: Image,
    /// Sprite metadata.
    metadata: SpriteMetadata,
}

impl Sprite {
    /// Create a sprite from a buffer of colors.
    pub fn from_buffer(buffer: &[u32], size: Extent2<usize>, offset: SpriteOffset) -> Self {
        let sprite = Image(BlitBuffer::from_buffer(buffer, size.w, 127));
        let metadata = SpriteMetadata {
            offset,
            ..Default::default()
        };

        Self { sprite, metadata }
    }

    /// Draw the sprite based on a camera offset.
    pub fn render(&self, offset: Vec2<f64>, canvas: &mut Canvas) {
        let total_offset = self.total_offset(offset);
        let mut options = BlitOptions::new_position(total_offset.x, total_offset.y);
        options.vertical_slice = self.metadata.vertical_slice;
        options.horizontal_slice = self.metadata.horizontal_slice;

        self.sprite
            .0
            .blit(canvas.buffer, canvas.size.into_tuple().into(), &options);
    }

    /// Draw the sprite filling the area.
    ///
    /// The behavior depends on the metadata of the sprite.
    pub fn render_area(&self, offset: Vec2<f64>, area: Extent2<usize>, canvas: &mut Canvas) {
        let total_offset = self.total_offset(offset);
        let mut options =
            BlitOptions::new_position(total_offset.x, total_offset.y).with_area(area.into_tuple());
        options.vertical_slice = self.metadata.vertical_slice;
        options.horizontal_slice = self.metadata.horizontal_slice;

        self.sprite
            .0
            .blit(canvas.buffer, canvas.size.into_tuple().into(), &options);
    }

    /// Draw the sprite with custom blitting options.
    ///
    /// This won't set any of the regular defaults, like the position.
    pub fn render_options(&self, blit_options: &BlitOptions, canvas: &mut Canvas) {
        self.sprite
            .0
            .blit(canvas.buffer, canvas.size.into_tuple().into(), blit_options);
    }

    /// Whether a pixel on the image is transparent.
    pub fn is_pixel_transparent(&self, pixel: Vec2<u32>) -> bool {
        let offset: Vec2<i32> = pixel.as_() + self.metadata.offset.offset(self.size());

        let index: i32 = offset.x + offset.y * self.sprite.0.width() as i32;
        let pixel = self.sprite.0.pixels()[index as usize];

        pixel == 0
    }

    /// Width of the image.
    pub fn width(&self) -> u32 {
        self.sprite.0.width()
    }

    /// Height of the image.
    pub fn height(&self) -> u32 {
        self.sprite.0.height()
    }

    /// Size of the image.
    pub fn size(&self) -> Extent2<u32> {
        Extent2::new(self.width(), self.height())
    }

    /// Raw buffer.
    pub fn into_blit_buffer(self) -> BlitBuffer {
        self.sprite.0
    }

    /// Get the raw pixels.
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        self.sprite.0.pixels_mut()
    }

    /// Calculate the total offset based on offset given.
    fn total_offset(&self, offset: Vec2<f64>) -> Vec2<i32> {
        // Add offset to our own offset
        offset.as_() + self.metadata.offset.offset(self.size())
    }
}

impl Default for Sprite {
    fn default() -> Self {
        let sprite = Image(BlitBuffer::from_buffer(&[0], 1, 0));
        let metadata = SpriteMetadata::default();

        Self { sprite, metadata }
    }
}

impl Compound for Sprite {
    fn load(cache: AnyCache, id: &SharedString) -> Result<Self, BoxedError> {
        // Load the sprite
        let sprite = cache
            .load_owned::<Image>(id)
            .into_diagnostic()
            .wrap_err("Error loading image for sprite")?;

        // Load the metadata
        let metadata = cache
            .load::<SpriteMetadata>(id)
            .into_diagnostic()
            .wrap_err("Error loading metadata for sprite")?
            .read()
            .clone();

        Ok(Self { sprite, metadata })
    }
}

/// Center of the sprite.
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
pub enum SpriteOffset {
    /// Middle of the sprite will be rendered at `(0, 0)`.
    #[default]
    Middle,
    /// Horizontal middle and vertical top will be rendered at `(0, 0)`.
    MiddleTop,
    /// Left top of the sprite will be rendered at `(0, 0)`.
    LeftTop,
    /// Sprite will be offset with the custom coordinates counting from the left top.
    Custom(Vec2<i32>),
}

impl SpriteOffset {
    /// Get the offset based on the sprite size.
    pub fn offset(&self, sprite_size: Extent2<u32>) -> Vec2<i32> {
        match self {
            SpriteOffset::Middle => {
                Vec2::new(-(sprite_size.w as i32) / 2, -(sprite_size.h as i32) / 2)
            }
            SpriteOffset::MiddleTop => Vec2::new(-(sprite_size.w as i32) / 2, 0),
            SpriteOffset::LeftTop => Vec2::zero(),
            SpriteOffset::Custom(offset) => *offset,
        }
    }
}

/// Sprite metadata to load from TOML.
#[derive(Debug, Clone, Default, Deserialize, Asset)]
#[asset_format = "toml"]
pub(crate) struct SpriteMetadata {
    /// Slices to render for scaling the image.
    #[serde(default)]
    pub(crate) vertical_slice: Option<Slice>,
    /// Slices to render for scaling the image.
    #[serde(default)]
    pub(crate) horizontal_slice: Option<Slice>,
    /// Pixel offset to render at.
    #[serde(default)]
    pub(crate) offset: SpriteOffset,
}

/// Core of a sprite loaded from disk.
#[derive(Debug)]
struct Image(BlitBuffer);

impl Asset for Image {
    // We only support PNG images currently
    const EXTENSION: &'static str = "png";

    type Loader = ImageLoader;
}

/// Sprite asset loader.
pub struct ImageLoader;

impl Loader<Image> for ImageLoader {
    fn load(content: Cow<[u8]>, ext: &str) -> Result<Image, BoxedError> {
        assert_eq!(ext.to_lowercase(), "png");

        let sprite = image::load_from_memory_with_format(&content, ImageFormat::Png)?
            .into_rgba8()
            .to_blit_buffer_with_alpha(127);

        Ok(Image(sprite))
    }
}
