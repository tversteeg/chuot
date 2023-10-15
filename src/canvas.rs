//! Wrapper around a pixel buffer.

use vek::{Extent2, Vec2};

/// Simple wrapper around a pixel buffer that can be passed around to rendering calls.
pub struct Canvas<'a> {
    /// Size of the canvas in pixels.
    pub(crate) size: Extent2<usize>,
    /// Reference to the pixel buffer.
    pub(crate) buffer: &'a mut [u32],
}

impl<'a> Canvas<'a> {
    /// Set a pixel on the buffer at the coordinate passed.
    ///
    /// If the coordinate is out of bounds nothing will be done.
    ///
    /// This is quite a slow operation because it needs to calculate the index of the coordinate, if setting multiple pixels it might be more efficient to create a sprite from them.
    #[inline]
    pub fn set_pixel(&mut self, position: Vec2<usize>, color: u32) {
        let index = position.product();

        if index < self.buffer.len() {
            self.buffer[index] = color;
        }
    }

    /// Get the raw buffer of pixels.
    #[inline]
    pub fn raw_buffer(&mut self) -> &mut [u32] {
        self.buffer
    }

    /// Width in pixels.
    #[inline]
    pub fn width(&self) -> usize {
        self.size.w
    }

    /// Height in pixels.
    #[inline]
    pub fn height(&self) -> usize {
        self.size.h
    }

    /// Size in pixels.
    #[inline]
    pub fn size(&self) -> Extent2<usize> {
        self.size
    }
}
