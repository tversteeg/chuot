//! 2D map of booleans.
//!
//! This is not an image.
//! Can be used for various effects such as masks for destructible terrain, pixel outlines etc.

use bitvec::vec::BitVec;
use vek::{Extent2, Vec2};

use crate::{
    canvas::Canvas,
    sprite::{Sprite, SpriteOffset},
};

/// 2D map of boolean values.
///
/// Not an image!
/// Every 'pixel' is a simple `true`/`false` value.
#[derive(Debug, Clone, PartialEq)]
pub struct BitMap {
    /// Amount of pixels in both dimensions.
    size: Extent2<usize>,
    /// All pixel values in a packed vector.
    map: BitVec,
}

impl BitMap {
    /// Create a new empty map.
    ///
    /// # Arguments
    ///
    /// * `size` - Amount of pixels in both dimensions.
    pub fn empty(size: Extent2<usize>) -> Self {
        let map = BitVec::repeat(false, size.product());

        Self { size, map }
    }

    /// Toggle a value at a coordinate.
    ///
    /// If it's `true` it becomes `false` and if it's `false` it becomes `true`.
    ///
    /// # Arguments
    ///
    /// * `position` - Coordinate inside the map to set, if outside nothing is done.
    pub fn toggle(&mut self, position: impl Into<Vec2<usize>>) {
        let position = position.into();
        if position.x >= self.size.w || position.y >= self.size.h {
            return;
        }

        let index = position.x + position.y * self.size.w;
        self.toggle_at_index(index);
    }

    /// Set a value at a coordinate.
    ///
    /// # Arguments
    ///
    /// * `position` - Coordinate inside the map to set, if outside nothing is done.
    /// * `value` - Boolean to set at the coordinate.
    pub fn set(&mut self, position: impl Into<Vec2<usize>>, value: bool) {
        let position = position.into();
        if position.x >= self.size.w || position.y >= self.size.h {
            return;
        }

        let index = position.x + position.y * self.size.w;
        self.set_at_index(index, value);
    }

    /// Perform a floodfill from a position.
    ///
    /// This will walk west, east, north and south with a stack, setting the value to the original value.
    ///
    /// # Arguments
    ///
    /// * `position` - Coordinate inside the map to start the floodfill from.
    /// * `value` - Boolean to fill the area with.
    pub fn floodfill(&mut self, position: impl Into<Vec2<usize>>, value: bool) {
        let position = position.into();

        // Create a stack for pixels that need to be filled
        let mut stack = Vec::with_capacity(16);
        stack.push(position.x + position.y * self.width());

        while let Some(index) = stack.pop() {
            let x = index % self.width();
            let y = index / self.width();
            if x >= self.width() || y >= self.height() || self.at_index(index) == value {
                continue;
            }

            // Fill the value
            self.set_at_index(index, value);

            // Push the neighbors

            // Right
            if x < self.width() - 1 {
                stack.push(index + 1);
            }

            // Left
            if x > 0 {
                stack.push(index.wrapping_sub(1));
            }

            // Up
            if y < self.height() - 1 {
                stack.push(index + self.width());
            }

            // Down
            if y > 0 {
                stack.push(index.wrapping_sub(self.width()));
            }
        }
    }

    /// Convert the value to a image where every `true` value is replaced by the color.
    ///
    /// # Arguments
    ///
    /// * `color` - Draw every boolean `true` value as a colored pixel on the image.
    /// * `offset` - Pixel offset of where the sprite will be drawn.
    pub fn to_sprite(&self, color: u32, offset: SpriteOffset) -> Sprite {
        // Convert each binary value to a pixel
        let pixels = self
            .map
            .iter()
            .map(|bit| if *bit { color } else { 0 })
            .collect::<Vec<_>>();

        // Create a sprite from it
        Sprite::from_buffer(&pixels, self.size, offset)
    }

    /// Get the value of a single pixel.
    pub fn value(&self, position: impl Into<Vec2<usize>>) -> bool {
        let position = position.into();

        let index = position.x + position.y * self.size.w;
        self.at_index(index)
    }

    /// Width of the map.
    pub fn width(&self) -> usize {
        self.size.w
    }

    /// Height of the map.
    pub fn height(&self) -> usize {
        self.size.h
    }

    /// Size of the map.
    pub fn size(&self) -> Extent2<usize> {
        self.size
    }

    /// Get a pixel at index of the map.
    fn at_index(&self, index: usize) -> bool {
        *self.map.get(index).expect("Index out of range")
    }

    /// Set a pixel at index of the map.
    fn set_at_index(&mut self, index: usize, value: bool) {
        self.map.set(index, value);
    }

    /// Toggle a pixel at index of the map.
    fn toggle_at_index(&mut self, index: usize) {
        let pixel = self.at_index(index);

        self.set_at_index(index, !pixel);
    }
}
