//! 2D map of booleans.
//!
//! This is not an image.
//! Can be used for various effects such as masks for destructible terrain, pixel outlines etc.

use bitvec::vec::BitVec;
use vek::{Extent2, Vec2};

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

    /// Create from a bitvec.
    ///
    /// # Arguments
    ///
    /// * `map` - Vector of booleans to wrap, length must be equal to `size.w * size.h`.
    /// * `size` - Size of the bitmap.
    pub fn from_bitvec(map: BitVec, size: Extent2<usize>) -> Self {
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

    /// Clone the buffer and apply equal padding to each edge.
    ///
    /// # Arguments
    ///
    /// * `padding` - Amount of values to add to each edge.
    pub fn clone_with_padding(&self, padding: usize) -> Self {
        // Create a new empty map in which we will copy each row
        let mut map = Self::empty(self.size + Extent2::new(padding * 2, padding * 2));

        // Copy the non-padded horizontal slices
        for y in 0..self.height() {
            map.copy_slice_from(
                Vec2::new(padding, padding + y),
                self,
                Vec2::new(0, y),
                self.width(),
            )
        }

        debug_assert_eq!(map.map.len(), map.size.product());

        map
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

    /// Copy a range from another source.
    #[inline(always)]
    fn copy_slice_from(
        &mut self,
        start: Vec2<usize>,
        other: &Self,
        other_start: Vec2<usize>,
        amount: usize,
    ) {
        debug_assert!(start.x + amount <= self.width());
        debug_assert!(other_start.x + amount <= other.width());

        let index = start.x + start.y * self.width();
        let other_index = other_start.x + other_start.y * other.width();

        self.map[index..(index + amount)]
            .copy_from_bitslice(&other.map[other_index..(other_index + amount)]);
    }
}

#[cfg(test)]
mod tests {
    use bitvec::prelude::*;
    use vek::Extent2;

    use super::BitMap;

    #[test]
    fn clone_with_padding() {
        #[rustfmt::skip]
        let map  = BitMap::from_bitvec(bits![
            1, 0, 1,
            0, 1, 0,
            1, 0, 1
        ].to_bitvec(), Extent2::new(3, 3));

        #[rustfmt::skip]
        assert_eq!(map.clone_with_padding(1), BitMap::from_bitvec(bits![
            0, 0, 0, 0, 0,
            0, 1, 0, 1, 0,
            0, 0, 1, 0, 0,
            0, 1, 0, 1, 0,
            0, 0, 0, 0, 0
        ].to_bitvec(), Extent2::new(5, 5)));
    }
}
