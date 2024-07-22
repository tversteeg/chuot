//! The camera system for both UI and the game camera.

use glam::FloatExt;

/// Camera for offsetting sprites.
pub(crate) struct Camera {
    /// Current horizontal position.
    x: f32,
    /// Current vertical position.
    y: f32,
    /// Next horizontal position.
    ///
    /// Set by update and interpolated with using the blending factor.
    next_x: f32,
    /// Next vertical position.
    ///
    /// Set by update and interpolated with using the blending factor.
    next_y: f32,
    /// Interpolated rendering horizontal position.
    render_x: f32,
    /// Interpolated rendering vertical position.
    render_y: f32,
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
    /// Current horizontal shake state.
    shake_x: Shake,
    /// Current vertical shake state.
    shake_y: Shake,
    /// Shake time in total.
    shake_duration: f32,
    /// Shake time left.
    shake_current_duration: f32,
    /// Shake force in pixels.
    shake_amplitude: f32,
    /// Shake frequency in Hertz.
    shake_frequency: f32,
}

impl Camera {
    /// Update the camera.
    #[inline]
    pub(crate) fn update(&mut self, dt: f32) {
        // Keep track of the next movement and the previous to interpolate with in the rendering tick
        self.x = self.next_x;
        self.y = self.next_y;
        self.next_x = self.x.lerp(self.target_x, self.lerp_x);
        self.next_y = self.y.lerp(self.target_y, self.lerp_y);

        // Apply camera shake
        if self.shake_current_duration > 0.0 {
            // Decay the amplitude
            let time_fraction =
                1.0 - (self.shake_duration - self.shake_current_duration) / self.shake_duration;
            let amplitude = self.shake_amplitude * time_fraction;

            self.shake_x.update(
                dt,
                self.shake_current_duration,
                self.shake_frequency,
                amplitude,
            );
            self.shake_y.update(
                dt,
                self.shake_current_duration,
                self.shake_frequency,
                amplitude,
            );

            self.shake_current_duration -= dt;

            // Reset the values
            if self.shake_current_duration <= 0.0 {
                self.shake_x = Shake::default();
                self.shake_y = Shake::default();
            }
        }
    }

    /// Render the camera.
    #[inline]
    pub(crate) fn render(&mut self, blending_factor: f32) {
        // Interpolate between this step and the next one using the blending factor
        self.render_x = self.x.lerp(self.next_x, blending_factor);
        self.render_y = self.y.lerp(self.next_y, blending_factor);

        // Interpolate the shake values
        self.shake_x.render(blending_factor);
        self.shake_y.render(blending_factor);
    }

    /// Set the horizontal lerp.
    #[inline]
    pub(crate) fn set_lerp_x(&mut self, lerp_x: f32) {
        self.lerp_x = lerp_x;
    }

    /// Set the vertical lerp.
    #[inline]
    pub(crate) fn set_lerp_y(&mut self, lerp_y: f32) {
        self.lerp_y = lerp_y;
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
        -self.render_x + self.offset_x + self.shake_x.value()
    }

    /// How much to offset the vertical position of the item to draw.
    #[inline]
    pub(crate) fn offset_y(&self) -> f32 {
        -self.render_y + self.offset_y + self.shake_y.value()
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

    /// Shake the camera.
    pub(crate) fn shake(&mut self, duration: f32, amplitude: f32, frequency: f32) {
        self.shake_duration = duration;
        self.shake_current_duration = duration;
        self.shake_amplitude = amplitude;
        self.shake_frequency = frequency;
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
            shake_x: Shake::default(),
            shake_y: Shake::default(),
            shake_duration: 0.0,
            shake_current_duration: 0.0,
            shake_amplitude: 0.0,
            shake_frequency: 60.0,
            next_x: 0.0,
            next_y: 0.0,
            render_x: 0.0,
            render_y: 0.0,
        }
    }
}

/// Internal 1D shake pattern.
#[derive(Default)]
struct Shake {
    /// Shake interpolation start, interval is the frequency.
    random_point_start: f32,
    /// Shake interpolation end, interval is the frequency.
    random_point_end: f32,
    /// Previous value.
    prev: f32,
    /// Next value.
    next: f32,
    /// Current interpolated render value.
    render: f32,
}

impl Shake {
    /// Update the shake.
    ///
    /// Based on: <https://jonny.morrill.me/en/blog/gamedev-how-to-implement-a-camera-shake-effect/>
    fn update(&mut self, dt: f32, duration: f32, frequency: f32, amplitude: f32) {
        // Check if subtracting the delta time crosses the lerp point
        let current_round = duration % frequency.recip();
        if current_round >= frequency.recip() - dt {
            // We need to calculate a new lerp point to progress
            self.random_point_start = self.random_point_end;
            self.random_point_end = crate::random(-1.0, 1.0);
        }

        // Calculate what to interpolate by finding the distance until the next time unit
        let lerp_offset = 1.0 - current_round / frequency.recip();

        // Interpolate the random value
        let direction = self
            .random_point_start
            .lerp(self.random_point_end, lerp_offset);

        // Update the state
        self.prev = self.next;
        self.next = direction * amplitude;
    }

    /// Update the interpolated render value.
    fn render(&mut self, blending_factor: f32) {
        self.render = self.prev.lerp(self.next, blending_factor);
    }

    /// Get the interpolated render value.
    const fn value(&self) -> f32 {
        self.render
    }
}
