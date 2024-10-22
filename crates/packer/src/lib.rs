#![forbid(unsafe_code)]

//! 2D texture packer based on [`texture_packer`](https://docs.rs/texture_packer/latest/texture_packer/).
//!
//! Removes all features for actually creating the textures and allows inserting already defined rectangles.

/// 2D rectangle packer.
#[derive(Debug, Clone)]
pub struct Packer {
    /// Max width of the output rectangle.
    max_width: u16,
    /// Max height of the output rectangle.
    max_height: u16,
    /// Skylines for the skyline packing algorithm.
    skylines: Vec<Skyline>,
}

impl Packer {
    /// Setup a new empty packer with a size.
    ///
    /// # Arguments
    ///
    /// * `(max_width, max_height)` - Tuple of maximum size of the output atlas.
    #[inline]
    #[must_use]
    pub fn new(max_size: impl Into<(u16, u16)>) -> Self {
        let (max_width, max_height) = max_size.into();

        // Start with a single skyline of the full width
        let skyline = Skyline {
            x: 0,
            y: 0,
            width: max_width,
        };
        let skylines = vec![skyline];

        Self {
            max_width,
            max_height,
            skylines,
        }
    }

    /// Fill the packer with already existing rectangles.
    ///
    /// The rectangles should be as close to Y = 0 as much as possible, to efficiently add new items.
    ///
    /// # Arguments
    ///
    /// * `existing_rectangles` - Iterator of pre-set tuple rectangles with `(x, y, width, height)` that should be filled into the positions.
    ///
    /// # Panics
    ///
    /// - When any rectangle is out of bounds.
    #[inline]
    #[must_use]
    pub fn with_existing_rectangles_iter<R>(
        mut self,
        existing_rectangles: impl Iterator<Item = R>,
    ) -> Self
    where
        R: Into<(u16, u16, u16, u16)>,
    {
        for rect in existing_rectangles {
            let (x, y, width, height) = rect.into();

            let y = y + height;

            // Construct the new skyline to check for overlaps and inserts
            let new_skyline = Skyline { x, y, width };

            // Split overlapping skylines
            let mut index = 0;
            let mut index_to_insert = 0;
            while index < self.skylines.len() {
                let skyline = self.skylines[index];

                if skyline.y > new_skyline.y || !skyline.overlaps(new_skyline) {
                    // Only take higher skylines that also overlap
                    continue;
                }

                if skyline.left() >= new_skyline.left() && skyline.right() <= new_skyline.right() {
                    // Skyline is inside the new one
                    self.skylines.remove(index);
                    continue;
                }

                if skyline.left() < new_skyline.left() && skyline.right() > new_skyline.right() {
                    // Old skyline is inside the new one

                    // Insert the slice after
                    self.skylines.insert(index + 1, Skyline {
                        x: new_skyline.right(),
                        y: skyline.y,
                        width: skyline.right() - new_skyline.right(),
                    });

                    // Cut the right part of the old skyline
                    self.skylines[index].width = new_skyline.left() - skyline.left();

                    // Insert between the recently split one
                    index_to_insert = index + 1;
                    break;
                }

                if skyline.left() < new_skyline.left() {
                    // Cut the right part of the old skyline
                    self.skylines[index].width = new_skyline.left() - skyline.left();

                    // Insert after this skyline
                    index_to_insert = index + 1;
                }

                if skyline.right() > new_skyline.right() {
                    // Cut the left part of the old skyline
                    self.skylines[index].width = skyline.right() - new_skyline.right();
                    self.skylines[index].x = new_skyline.right();

                    // Insert before this skyline
                    index_to_insert = index;
                    break;
                }

                index += 1;
            }
            // Insert the skyline here
            self.skylines.insert(index_to_insert, new_skyline);

            // Merge the skylines on the same height
            self.merge();
        }

        self
    }

    /// Insert and pack a rectangle.
    ///
    /// # Arguments
    ///
    /// * `(width, height)` - Size tuple of the rectangle to place in the atlas.
    ///
    /// # Returns
    ///
    /// - `None` when there's not enough space to pack the rectangle.
    /// - Offset tuple `(width, height)` inside the atlas when the rectangle fits.
    #[inline]
    pub fn insert(&mut self, rectangle_size: impl Into<(u16, u16)>) -> Option<(u16, u16)> {
        let (rectangle_width, rectangle_height) = rectangle_size.into();

        // Find the rectangle with the skyline, keep the bottom and width as small as possible
        let mut bottom = u16::MAX;
        let mut width = u16::MAX;
        let mut result = None;

        // Try to find the skyline gap with the smallest Y
        for (index, skyline) in self.skylines.iter().enumerate() {
            if let Some((offset_x, offset_y)) =
                self.can_put(index, rectangle_width, rectangle_height)
            {
                let rect_bottom = offset_y + rectangle_height;
                if rect_bottom < bottom || (rect_bottom == bottom && skyline.width < width) {
                    bottom = rect_bottom;
                    width = skyline.width;
                    result = Some((offset_x, offset_y, index));
                }
            }
        }

        // If no rect is found do nothing
        let (x, y, index) = result?;

        // Insert the skyline
        self.split(index, x, y, rectangle_width, rectangle_height);

        // Merge the skylines on the same height
        self.merge();

        Some((x, y))
    }

    /// Return the rect fitting in a skyline if possible.
    fn can_put(&self, skyline_index: usize, width: u16, height: u16) -> Option<(u16, u16)> {
        // Right side of the rectangle, doesn't change because only the Y position will shift in the next loop
        let x = self.skylines[skyline_index].x;
        let right = x + width;

        let mut y = 0;
        let mut width_left = width;

        // Loop over each skyline to the right starting from the current position to try to find a spot where the rectangle can be put
        self.skylines[skyline_index..].iter().find_map(|skyline| {
            // Get the highest position of each available skyline
            y = y.max(skyline.y);

            // Check if the rectangle still fits in the output
            let bottom = y + height;
            if right > self.max_width || bottom > self.max_height {
                return None;
            }

            if skyline.width >= width_left {
                // Rectangle fits, return it
                Some((x, y))
            } else {
                width_left -= skyline.width;

                None
            }
        })
    }

    /// Split the skylines at the index.
    ///
    /// Will shorten or remove the overlapping skylines to the right.
    fn split(&mut self, skyline_index: usize, x: u16, y: u16, width: u16, height: u16) {
        // Add the new skyline
        let y = y + height;
        self.skylines.insert(skyline_index, Skyline { x, y, width });

        // Shrink all skylines to the right of the inserted skyline
        self.shrink(skyline_index);
    }

    /// Shrink all skylines from the right of the index.
    fn shrink(&mut self, skyline_index: usize) {
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
                    self.skylines[index].x += shrink;
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

            if previous.y == current.y {
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
    /// X position on the rectangle.
    x: u16,
    /// Y position on the rectangle.
    y: u16,
    /// Width of the line.
    width: u16,
}

impl Skyline {
    /// Left split position.
    #[inline(always)]
    pub const fn left(self) -> u16 {
        self.x
    }

    /// Right split position.
    #[inline(always)]
    pub const fn right(self) -> u16 {
        self.x + self.width
    }

    /// Whether it overlaps with another skyline.
    #[inline(always)]
    pub const fn overlaps(self, other: Self) -> bool {
        self.right() >= other.left() && other.right() >= self.left()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packer_fill_squares() {
        // Filling the 32x32 square with 64 equal blocks of 4x4 should fill the box exactly
        let mut packer = Packer::new((32, 32));
        for _ in 0..64 {
            assert!(packer.insert((4, 4)).is_some());
        }

        // Filling the 32x32 square with 128 equal blocks of 4x2 should fill the box exactly
        let mut packer = Packer::new((32, 32));
        for _ in 0..128 {
            assert!(packer.insert((4, 2)).is_some());
        }
    }

    #[test]
    fn packer_fill_squares_overflow() {
        // Filling the 32x32 square with 64 + 1 equal blocks of 4x4 should overflow the box
        let mut packer = Packer::new((32, 32));
        for _ in 0..64 {
            assert!(packer.insert((4, 4)).is_some());
        }
        assert!(packer.insert((4, 4)).is_none());
    }

    #[test]
    fn existing_rects() {
        // Filling the 32x32 square with 63 + 1 predefined + 1 equal blocks of 4x4 should overflow the box
        let mut packer =
            Packer::new((32, 32)).with_existing_rectangles_iter(std::iter::once((0, 0, 4, 4)));
        for _ in 0..63 {
            assert!(packer.insert((4, 4)).is_some());
        }
        assert!(packer.insert((4, 4)).is_none());

        // Filling the 32x32 square with 63 + 1 predefined + 1 equal blocks of 4x4 should overflow the box
        let mut packer =
            Packer::new((32, 32)).with_existing_rectangles_iter(std::iter::once((28, 0, 4, 4)));
        for _ in 0..63 {
            assert!(packer.insert((4, 4)).is_some());
        }
        assert!(packer.insert((4, 4)).is_none());

        // Filling the 32x32 square with 63 + 1 predefined + 1 equal blocks of 4x4 should overflow the box
        let mut packer =
            Packer::new((32, 32)).with_existing_rectangles_iter(std::iter::once((4, 0, 4, 4)));
        for _ in 0..63 {
            assert!(packer.insert((4, 4)).is_some());
        }
        assert!(packer.insert((4, 4)).is_none());
    }
}
