//! Wrapper around a pixel buffer.

use line_drawing::Bresenham;
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
    pub fn set_pixel(&mut self, position: Vec2<f64>, color: u32) {
        if position.x < 0.0
            || position.y < 0.0
            || position.x >= self.size.w as f64
            || position.y >= self.size.h as f64
        {
            return;
        }

        let index = position.x as usize + position.y as usize * self.size.w;
        if index < self.buffer.len() {
            self.buffer[index] = color;
        }
    }

    /// Draw a line using Bresenham's line algorithm.
    #[inline]
    pub fn draw_line(&mut self, start: Vec2<f64>, end: Vec2<f64>, color: u32) {
        // PERF: optimize
        for (x, y) in Bresenham::new(
            (start.x as i32, start.y as i32),
            (end.x as i32, end.y as i32),
        ) {
            self.set_pixel(Vec2::new(x, y).as_(), color);
        }
    }

    /// Fill the canvas with a single color.
    #[inline]
    pub fn fill(&mut self, color: u32) {
        self.buffer.fill(color);
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
