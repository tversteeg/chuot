//! Zero-cost abstraction types for configuring the camera systems.

use crate::Context;

/// Specify how the text should be drawn.
///
/// Used by [`crate::Context::main_camera`] and [`crate::Context::ui_camera`].
pub struct CameraContext<'ctx> {
    /// Reference to the context the text will draw in when finished.
    pub(crate) ctx: &'ctx Context,
    /// Whether to use the UI camera for positioning the text, `false` uses the regular game camera.
    pub(crate) is_ui_camera: bool,
}

impl<'ctx> CameraContext<'ctx> {
    /// Center the camera at the middle of the screen.
    ///
    /// This is the default for the main camera.
    #[inline]
    pub fn center(&self) {
        self.ctx.write(|ctx| {
            let width = ctx.config.buffer_width;
            let height = ctx.config.buffer_height;

            ctx.camera_mut(self.is_ui_camera).center(width, height);
        });
    }

    /// Center the camera at the top left corner of the screen.
    ///
    /// This is the default for the UI camera.
    #[inline]
    pub fn top_left(&self) {
        self.ctx.write(|ctx| {
            ctx.camera_mut(self.is_ui_camera).top_left();
        });
    }

    /// Make the camera move towards the location on the horizontal axis only.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal target position in world space to follow.
    #[inline]
    pub fn follow_x(&self, x: f32) {
        self.ctx.write(|ctx| {
            ctx.camera_mut(self.is_ui_camera).set_target_x(x);
        });
    }

    /// Make the camera move towards the location on the vertical axis only.
    ///
    /// # Arguments
    ///
    /// * `y` - Vertical target position in world space to follow.
    #[inline]
    pub fn follow_y(&self, y: f32) {
        self.ctx.write(|ctx| {
            ctx.camera_mut(self.is_ui_camera).set_target_y(y);
        });
    }

    /// Make the camera move towards the location.
    ///
    /// # Arguments
    ///
    /// * `(x, y)` - Tuple of the target position in world space to follow.
    #[inline]
    pub fn follow(&self, target: impl Into<(f32, f32)>) {
        let (x, y) = target.into();

        self.ctx.write(|ctx| {
            let camera = ctx.camera_mut(self.is_ui_camera);
            camera.set_target_x(x);
            camera.set_target_y(y);
        });
    }

    /// Get the relative position if the mouse is inside the viewport frame.
    ///
    /// This is `Some(..`) if the mouse is inside the viewport frame, not the entire window.
    /// The value of the coordinates corresponds to the pixel, when the frame is scaled this also encodes the subpixel in the fractional part.
    ///
    /// # Returns
    ///
    /// - `None` when the mouse is not on the buffer of pixels.
    /// - `Some(..)` with the coordinates of the pixel if the mouse is on the buffer of pixels.
    #[inline]
    #[must_use]
    pub fn mouse(&self) -> Option<(f32, f32)> {
        self.ctx.read(|ctx| {
            ctx.input.mouse().map(|(mouse_x, mouse_y)| {
                let camera = ctx.camera(self.is_ui_camera);

                (mouse_x - camera.offset_x(), mouse_y - camera.offset_y())
            })
        })
    }

    /// Get the relative horizontal position if the mouse is inside the viewport frame.
    ///
    /// This is `Some(..`) if the mouse is inside the viewport frame, not the entire window.
    /// The value of the coordinates corresponds to the pixel, when the frame is scaled this also encodes the subpixel in the fractional part.
    ///
    /// # Returns
    ///
    /// - `None` when the mouse is not on the buffer of pixels.
    /// - `Some(..)` with the X coordinate of the pixel if the mouse is on the buffer of pixels.
    #[inline]
    #[must_use]
    pub fn mouse_x(&self) -> Option<f32> {
        self.ctx.read(|ctx| {
            ctx.input.mouse().map(|(mouse_x, _)| {
                let camera = ctx.camera(self.is_ui_camera);

                mouse_x - camera.offset_x()
            })
        })
    }

    /// Get the relative vertical position if the mouse is inside the viewport frame.
    ///
    /// This is `Some(..`) if the mouse is inside the viewport frame, not the entire window.
    /// The value of the coordinates corresponds to the pixel, when the frame is scaled this also encodes the subpixel in the fractional part.
    ///
    /// # Returns
    ///
    /// - `None` when the mouse is not on the buffer of pixels.
    /// - `Some(..)` with the Y coordinate of the pixel if the mouse is on the buffer of pixels.
    #[inline]
    #[must_use]
    pub fn mouse_y(&self) -> Option<f32> {
        self.ctx.read(|ctx| {
            ctx.input.mouse().map(|(_, mouse_y)| {
                let camera = ctx.camera(self.is_ui_camera);

                mouse_y - camera.offset_y()
            })
        })
    }
}

/// Configuration methods for cameras.
impl Context {
    /// Configure the main game camera.
    ///
    /// This is the default camera that will be used to position all graphical elements on the screen and move them in the game world.
    /// If you want the sprites or text to not move with the game use [`Context::ui_camera`].
    ///
    /// # Returns
    ///
    /// - A helper struct allowing you to configure the camera.
    #[inline(always)]
    #[must_use]
    pub const fn main_camera(&self) -> CameraContext<'_> {
        CameraContext {
            ctx: self,
            is_ui_camera: false,
        }
    }

    /// Configure the camera for drawing user interfaces.
    ///
    /// This is the default camera that will be used to position all graphical elements on the screen specified with [`super::SpriteContext::use_ui_camera`] and [`super::TextContext::use_ui_camera`].
    /// If you want the sprites or text to move with the game use [`Context::main_camera`].
    ///
    /// # Returns
    ///
    /// - A helper struct allowing you to configure the camera.
    #[inline(always)]
    #[must_use]
    pub const fn ui_camera(&self) -> CameraContext<'_> {
        CameraContext {
            ctx: self,
            is_ui_camera: true,
        }
    }
}
