//! Wrapper around a pixel buffer.

use std::{cmp::Ordering, ops::Range};

use vek::{Disk, Extent2, LineSegment2, Vec2};

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
        if position.x < 0.0 || position.y < 0.0 {
            return;
        }

        let x = position.x.round() as usize;
        let y = position.y.round() as usize;
        if x >= self.size.w || y >= self.size.h {
            return;
        }

        let index = x + y * self.size.w;
        self.buffer[index] = color;
    }

    /// Draw a line using Bresenham's line algorithm.
    #[inline]
    pub fn draw_line(&mut self, start: Vec2<f64>, end: Vec2<f64>, color: u32) {
        let isize_width = self.size.w as isize;

        clipline::clipline(
            (start.as_().into_tuple(), end.as_().into_tuple()),
            ((0, 0), self.size.as_().into_tuple()),
            |x: isize, y: isize| {
                let index = x + y * isize_width;

                self.buffer[index as usize] = color;
            },
        );
    }

    /// Fill a horizontal line, very cheap operation.
    #[inline]
    pub fn draw_scanline(&mut self, y: usize, x: Range<usize>, color: u32) {
        if y >= self.size.h {
            return;
        }

        let y_index = y * self.size.w;

        // Flip if end is later than start
        let x_start = y_index + x.start.min(self.size.w - 1);
        let x_end = y_index + x.end.min(self.size.w);

        self.buffer[x_start..x_end].fill(color);
    }

    /// Draw a circle using Bresenham's circle algorithm.
    pub fn draw_circle_outline(&mut self, circle: Disk<f64, f64>, color: u32) {
        let radius = circle.radius.round() as i32;

        let mut x = 0;
        let mut y = -radius;
        let mut f_m = 1 - radius;
        let mut d_e = 3;
        let mut d_ne = -((radius) << 1) + 5;

        let center = circle.center.round();

        // Draw the missing horizontal line
        let mut draw_pixel_8_sides = |x: f64, y: f64| {
            self.set_pixel(center + Vec2::new(x, y), color);
            self.set_pixel(center + Vec2::new(-x, y), color);
            self.set_pixel(center + Vec2::new(-x, -y), color);
            self.set_pixel(center + Vec2::new(x, -y), color);
            self.set_pixel(center + Vec2::new(y, x), color);
            self.set_pixel(center + Vec2::new(-y, x), color);
            self.set_pixel(center + Vec2::new(-y, -x), color);
            self.set_pixel(center + Vec2::new(y, -x), color);
        };

        // Draw main corners
        draw_pixel_8_sides(circle.radius.round(), 0.0);

        while x < -y {
            if f_m <= 0 {
                f_m += d_e;
            } else {
                f_m += d_ne;
                d_ne += 2;
                y += 1;
            }

            d_e += 2;
            d_ne += 2;
            x += 1;

            draw_pixel_8_sides(x as f64, y as f64);
        }
    }

    /// Fill a circle using Bresenham's circle algorithm.
    ///
    /// Based on: https://funloop.org/post/2021-03-15-bresenham-circle-drawing-algorithm.html
    pub fn draw_circle(&mut self, circle: Disk<f64, f64>, color: u32) {
        let center: Vec2<i32> = circle.center.round().as_();
        let radius = circle.radius.round() as i32;

        let mut x = 0;
        let mut y = -radius;
        let mut f_m = 1 - radius;
        let mut d_e = 3;
        let mut d_ne = -((radius) << 1) + 5;

        // Draw the missing horizontal line
        let left = (center.x - radius).max(0) as usize;
        let right = (center.x + radius).max(0) as usize;
        self.draw_scanline(center.y.max(0) as usize, left..right, color);

        self.set_pixel(circle.center + Vec2::new(0.0, circle.radius), color);
        self.set_pixel(circle.center - Vec2::new(0.0, circle.radius), color);

        while x < -y {
            if f_m <= 0 {
                f_m += d_e;
            } else {
                f_m += d_ne;
                d_ne += 2;
                y += 1;
            }

            d_e += 2;
            d_ne += 2;
            x += 1;

            let mut draw_scanline = |x: i32, y: i32| {
                let negative_y = (center.y - y).max(0) as usize;
                let positive_y = (center.y + y).max(0) as usize;

                let absolute_min_x = (center.x - x).max(0) as usize;
                let absolute_max_x = (center.x + x).max(0) as usize;

                self.draw_scanline(negative_y, absolute_min_x..absolute_max_x, color);
                self.draw_scanline(positive_y, absolute_min_x..absolute_max_x, color);
            };

            draw_scanline(x, y);
            draw_scanline(-y, x);
        }
    }

    /// Fill a triangle.
    ///
    /// Based on: https://joshbeam.com/articles/triangle_rasterization/
    pub fn draw_triangle(&mut self, corners: [Vec2<f64>; 3], color: u32) {
        // Create the 3 edges from the triangle
        let edges = [
            Edge::new(corners[0], corners[1]),
            Edge::new(corners[1], corners[2]),
            Edge::new(corners[2], corners[0]),
        ];

        // Find the longest edge
        let Some((longest_edge_index, longest_edge)) =
            edges.iter().enumerate().max_by(|(_, edge1), (_, edge2)| {
                edge1
                    .y_length()
                    .partial_cmp(&edge2.y_length())
                    .unwrap_or(Ordering::Equal)
            })
        else {
            // Something weird happened, just don't do anything in that case
            return;
        };

        // Find the other two edges
        let short_edge1 = &edges[(longest_edge_index + 1) % 3];
        let short_edge2 = &edges[(longest_edge_index + 2) % 3];

        // Draw the spans between both edges
        self.draw_span_between_edges(longest_edge, short_edge1, color);
        self.draw_span_between_edges(longest_edge, short_edge2, color);
    }

    /// Fill a polygon with 4 corners.
    ///
    ///
    /// If any of the 4 points falls inside the triangle of the other three only a single triangle will be drawn.
    ///
    /// Based on: https://stackoverflow.com/a/2122620
    pub fn draw_quad(&mut self, corners: [Vec2<f64>; 4], color: u32) {
        fn signed_area(a: Vec2<f64>, b: Vec2<f64>, c: Vec2<f64>) -> f64 {
            (a.y - b.y) * c.x + (b.x - a.x) * c.y + (a.x * b.y - b.x * a.y)
        }

        let abc_is_clockwise = signed_area(corners[0], corners[1], corners[2]) > 0.0;

        let abd_is_clockwise = signed_area(corners[0], corners[1], corners[3]) > 0.0;
        let bcd_is_clockwise = signed_area(corners[1], corners[2], corners[3]) > 0.0;
        let cad_is_clockwise = signed_area(corners[2], corners[0], corners[3]) > 0.0;

        // Match by checking the other triangle signs against ABC
        match (
            abd_is_clockwise == abc_is_clockwise,
            bcd_is_clockwise == abc_is_clockwise,
            cad_is_clockwise == abc_is_clockwise,
        ) {
            // ABC ABD
            (false, true, true) => {
                self.draw_triangle([corners[0], corners[1], corners[2]], color);
                self.draw_triangle([corners[0], corners[1], corners[3]], color);
            }
            // ABC BCD
            (true, false, true) => {
                self.draw_triangle([corners[0], corners[1], corners[2]], color);
                self.draw_triangle([corners[1], corners[2], corners[3]], color);
            }
            // ABC CAD
            (true, true, false) => {
                self.draw_triangle([corners[0], corners[1], corners[2]], color);
                self.draw_triangle([corners[2], corners[0], corners[3]], color);
            }

            // D is inside ABC
            (true, true, true) => self.draw_triangle([corners[0], corners[1], corners[2]], color),
            // C is inside ABD
            (true, false, false) => self.draw_triangle([corners[0], corners[1], corners[3]], color),
            // B is inside CAD
            (false, false, true) => self.draw_triangle([corners[0], corners[2], corners[3]], color),
            // A is inside BCD
            (false, true, false) => self.draw_triangle([corners[1], corners[2], corners[3]], color),

            // Shouldn't happen
            _ => (),
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

/// Drawing helpers.
impl<'a> Canvas<'a> {
    /// Draw a span between two edges, this fills everything in between.
    fn draw_span_between_edges(&mut self, long: &Edge, short: &Edge, color: u32) {
        let long_y_length = long.y_length();
        // Ignore horizontal edges
        if long_y_length.abs() < 0.5 {
            return;
        }

        let short_y_length = short.y_length();
        // Ignore horizontal edges
        if short_y_length.abs() < 0.5 {
            return;
        }

        let long_x_diff = long.x_diff();
        let short_x_diff = short.x_diff();

        // Calculate interpolation factors
        let mut long_factor = (short.0.start.y - long.0.start.y) / long_y_length;
        let long_factor_step = long_y_length.recip();
        let mut short_factor = 0.0;
        let short_factor_step = short_y_length.recip();

        // Clamp to the canvas
        let start_y = (short.0.start.y.floor().max(0.0) as usize).min(self.size.h);
        let end_y = (short.0.end.y.ceil().max(0.0) as usize).min(self.size.h);

        for y in start_y..end_y {
            // Calculate the X based on the interpolation by Y
            let long_x = long.0.start.x + long_x_diff * long_factor;
            let short_x = short.0.start.x + short_x_diff * short_factor;

            let (start_x, end_x) = if long_x < short_x {
                (long_x, short_x)
            } else {
                (short_x, long_x)
            };

            // Clamp to the buffer
            let start_x = (start_x.floor().max(0.0) as usize).min(self.size.w);
            let end_x = (end_x.ceil().max(0.0) as usize).min(self.size.w);

            // Draw the pixels
            self.draw_scanline(y, start_x..end_x, color);

            // Increase interpolation factors
            long_factor += long_factor_step;
            short_factor += short_factor_step;
        }
    }
}

/// Edge helper wrapper for line segments.
struct Edge(LineSegment2<f64>);

impl Edge {
    /// Create an edge, sorting the Y coordinates.
    pub fn new(point1: Vec2<f64>, point2: Vec2<f64>) -> Self {
        if point1.y < point2.y {
            Self(LineSegment2 {
                start: point1,
                end: point2,
            })
        } else {
            Self(LineSegment2 {
                start: point2,
                end: point1,
            })
        }
    }

    /// Difference across the X axis, not length because it can be negative.
    fn x_diff(&self) -> f64 {
        self.0.end.x - self.0.start.x
    }

    /// Length across the Y axis.
    fn y_length(&self) -> f64 {
        self.0.end.y - self.0.start.y
    }
}
