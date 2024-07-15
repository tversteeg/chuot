//! The main camera.

use glam::FloatExt;

/// Camera for offsetting sprites.
pub(crate) struct Camera {
    /// Current horizontal position.
    x: f32,
    /// Current vertical position.
    y: f32,
    /// Screen horizontal offset.
    offset_x: f32,
    /// Screen vertical offset.
    offset_y: f32,
    /// Target horizontal position.
    target_x: f32,
    /// Target vertical position.
    target_y: f32,
    /// How fast to interpolate between the horizontal positions.
    lerp_x: f32,
    /// How fast to interpolate between the vertical positions.
    lerp_y: f32,
}

impl Camera {
    /// Update the camera.
    #[inline]
    pub(crate) fn update(&mut self) {
        self.x = self.x.lerp(self.target_x, self.lerp_x);
        self.y = self.y.lerp(self.target_y, self.lerp_y);
    }

    /// Set the target horizontal position.
    #[inline]
    pub(crate) fn set_target_x(&mut self, x: f32) {
        self.target_x = x;
    }

    /// Set the target vertical position.
    #[inline]
    pub(crate) fn set_target_y(&mut self, y: f32) {
        self.target_y = y;
    }

    /// How much to offset the horizontal position of the item to draw.
    #[inline]
    pub(crate) fn offset_x(&self) -> f32 {
        -self.x + self.offset_x
    }

    /// How much to offset the vertical position of the item to draw.
    #[inline]
    pub(crate) fn offset_y(&self) -> f32 {
        -self.y + self.offset_y
    }

    /// Center the camera at the middle of the screen.
    #[inline]
    pub(crate) fn center(&mut self, buffer_width: f32, buffer_height: f32) {
        self.offset_x = buffer_width / 2.0;
        self.offset_y = buffer_height / 2.0;
    }

    /// Center the camera at the top left corner of the screen.
    #[inline]
    pub(crate) fn top_left(&mut self) {
        self.offset_x = 0.0;
        self.offset_y = 0.0;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            target_x: 0.0,
            target_y: 0.0,
            lerp_x: 0.3,
            lerp_y: 0.3,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}
