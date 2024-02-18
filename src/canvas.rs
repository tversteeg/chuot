//! Wrapper around a pixel buffer.

use std::cmp::Ordering;

use line_drawing::{Bresenham, BresenhamCircle};
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

    /// Draw a circle using Bresenham's circle algorithm.
    pub fn draw_circle_outline(&mut self, circle: Disk<f64, f64>, color: u32) {
        // PERF: optimize
        for (x, y) in BresenhamCircle::new(
            circle.center.x as i32,
            circle.center.y as i32,
            circle.radius as i32,
        ) {
            self.set_pixel(Vec2::new(x, y).as_(), color);
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
            let y_index = y * self.size.w;
            self.buffer[(y_index + start_x)..(y_index + end_x)].fill(color);

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
