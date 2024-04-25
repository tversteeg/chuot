//! 2D texture packer based on [`texture_packer`](https://docs.rs/texture_packer/latest/texture_packer/).
//!
//! Removes all features for actually creating the textures and allows inserting already defined rectangles.

use glamour::{Point2, Size2};

/// 2D rectangle packer.
#[derive(Debug, Clone)]
pub struct Packer {
    /// Max size of the output rectangle.
    max_size: Size2<u16>,
    /// Skylines for the skyline packing algorithm.
    skylines: Vec<Skyline>,
}

impl Packer {
    /// Setup a new packer with a size.
    ///
    /// # Arguments
    ///
    /// * `max_size` - Maximum size of the output atlas.
    #[inline]
    pub fn new(max_size: Size2<u16>) -> Self {
        // Start with a single skyline of the full width
        let skyline = Skyline {
            position: Point2::ZERO,
            width: max_size.width,
        };
        let skylines = vec![skyline];

        Self { max_size, skylines }
    }

    /// Insert and pack a rectangle.
    ///
    /// # Arguments
    ///
    /// * `rectangle_size` - Size of the rectangle to place in the atlas.
    ///
    /// # Returns
    ///
    /// - `None` when there's not enough space to pack the rectangle.
    /// - The offset inside the atlas when the rectangle fits.
    pub fn insert(&mut self, rectangle_size: Size2<u16>) -> Option<Point2<u16>> {
        // Find the rectangle with the skyline, keep the bottom and width as small as possible
        let mut bottom = std::u16::MAX;
        let mut width = std::u16::MAX;
        let mut result = None;

        // Try to find the skyline gap with the smallest Y
        for (index, skyline) in self.skylines.iter().enumerate() {
            if let Some(offset) = self.can_put(index, rectangle_size) {
                let rect_bottom = offset.y + rectangle_size.height;
                if rect_bottom < bottom || (rect_bottom == bottom && skyline.width < width) {
                    bottom = rect_bottom;
                    width = skyline.width;
                    result = Some((offset, index));
                }
            }
        }

        // If no rect is found do nothing
        let (position, index) = result?;

        // Insert the skyline
        self.split(index, position, rectangle_size);

        // Merge the skylines on the same height
        self.merge();

        Some(position)
    }

    /// Return the rect fitting in a skyline if possible.
    fn can_put(&self, skyline_index: usize, size: Size2<u16>) -> Option<Point2<u16>> {
        // Right side of the rectangle, doesn't change because only the Y position will shift in the next loop
        let x = self.skylines[skyline_index].position.x;
        let right = x + size.width;

        let mut y = 0;
        let mut width_left = size.width;

        // Loop over each skyline to the right starting from the current position to try to find a spot where the rectangle can be put
        self.skylines[skyline_index..].iter().find_map(|skyline| {
            // Get the highest position of each available skyline
            y = y.max(skyline.position.y);

            // Check if the rectangle still fits in the output
            let bottom = y + size.height;
            if right > self.max_size.width || bottom > self.max_size.height {
                return None;
            }

            if skyline.width >= width_left {
                // Rectangle fits, return it
                Some(Point2::new(x, y))
            } else {
                width_left -= skyline.width;

                None
            }
        })
    }

    /// Split the skylines at the index.
    ///
    /// Will shorten or remove the overlapping skylines to the right.
    fn split(&mut self, skyline_index: usize, position: Point2<u16>, size: Size2<u16>) {
        // Add the new skyline
        self.skylines.insert(
            skyline_index,
            Skyline {
                position: Point2::new(position.x, position.y + size.height),
                width: size.width,
            },
        );

        // Shrink all skylines to the right of the inserted skyline
        let index = skyline_index + 1;
        while index < self.skylines.len() {
            let previous = &self.skylines[index - 1];
            let current = &self.skylines[index];

            assert!(previous.left() <= current.left());

            // Check if the previous overlaps the current
            if current.left() <= previous.right() {
                let shrink = previous.right() - current.left();
                if current.width <= shrink {
                    // Skyline is fully overlapped, remove it and move to the next
                    self.skylines.remove(index);
                } else {
                    // Skyline is partially overlapped, shorten it
                    self.skylines[index].position.x += shrink;
                    self.skylines[index].width -= shrink;
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Merge neighbor skylines on the same height.
    fn merge(&mut self) {
        let mut index = 1;
        while index < self.skylines.len() {
            let previous = &self.skylines[index - 1];
            let current = &self.skylines[index];

            if previous.position.y == current.position.y {
                // Merge the skylines
                self.skylines[index - 1].width += current.width;
                self.skylines.remove(index);
                index -= 1;
            }

            index += 1;
        }
    }
}

/// Single skyline with only a width.
#[derive(Debug, Clone, Copy)]
struct Skyline {
    /// Position on the rectangle.
    position: Point2<u16>,
    /// Width of the line.
    width: u16,
}

impl Skyline {
    /// Left split position.
    #[inline(always)]
    pub const fn left(&self) -> u16 {
        self.position.x
    }

    /// Right split position.
    #[inline(always)]
    pub const fn right(&self) -> u16 {
        self.position.x + self.width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packer_fill_squares() {
        // Filling the 32x32 square with 64 equal blocks of 4x4 should fill the box exactly
        let mut packer = Packer::new(Size2::new(32, 32));
        for _ in 0..64 {
            assert!(packer.insert(Size2::new(4, 4)).is_some());
        }

        // Filling the 32x32 square with 128 equal blocks of 4x2 should fill the box exactly
        let mut packer = Packer::new(Size2::new(32, 32));
        for _ in 0..128 {
            assert!(packer.insert(Size2::new(4, 2)).is_some());
        }
    }

    #[test]
    fn packer_fill_squares_overflow() {
        // Filling the 32x32 square with 64 + 1 equal blocks of 4x4 should overflow the box
        let mut packer = Packer::new(Size2::new(32, 32));
        for _ in 0..64 {
            assert!(packer.insert(Size2::new(4, 4)).is_some());
        }
        assert!(packer.insert(Size2::new(4, 4)).is_none());
    }
}
