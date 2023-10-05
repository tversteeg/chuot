//! Blittable sprite definitions.
//!
//! Can be loaded as an asset when the `asset` feature flag is set.

use blit::{Blit, BlitBuffer, BlitOptions, ToBlitBuffer};
use vek::{Extent2, Vec2};

/// Sprite that can be drawn on the  canvas.
#[derive(Debug)]
pub struct Sprite {
    /// Pixels to render.
    pub(crate) sprite: BlitBuffer,
    /// Pixel offset to render at.
    pub(crate) offset: Vec2<i32>,
}

impl Sprite {
    /// Create a sprite from a buffer of colors.
    pub fn from_buffer(buffer: &[u32], size: Extent2<usize>, offset: SpriteOffset) -> Self {
        let sprite = BlitBuffer::from_buffer(buffer, size.w, 127);
        let offset = offset.offset(size.as_());

        Self { sprite, offset }
    }

    /// Draw the sprite based on a camera offset.
    pub fn render(&self, offset: Vec2<f64>, canvas: &mut [u32], canvas_size: Extent2<usize>) {
        self.sprite.blit(
            canvas,
            canvas_size.into_tuple().into(),
            &BlitOptions::new_position(offset.x, offset.y),
        );
    }

    /// Whether a pixel on the image is transparent.
    pub fn is_pixel_transparent(&self, pixel: Vec2<u32>) -> bool {
        let offset: Vec2<i32> = pixel.as_() + self.offset;

        let index: i32 = offset.x + offset.y * self.sprite.width() as i32;
        let pixel = self.sprite.pixels()[index as usize];

        pixel == 0
    }

    /// Width of the image.
    pub fn width(&self) -> u32 {
        self.sprite.width()
    }

    /// Height of the image.
    pub fn height(&self) -> u32 {
        self.sprite.height()
    }

    /// Size of the image.
    pub fn size(&self) -> Extent2<u32> {
        Extent2::new(self.width(), self.height())
    }

    /// Raw buffer.
    pub fn into_blit_buffer(self) -> BlitBuffer {
        self.sprite
    }

    /// Get the raw pixels.
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        self.sprite.pixels_mut()
    }
}

/// Center of the sprite.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
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
