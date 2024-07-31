//! Camera options.

/// Whether a type is a UI camera.
#[doc(hidden)]
pub trait IsUiCamera {
    /// Is this type a UI camera type?
    fn is_ui_camera() -> bool;
}

/// Item is drawn using the main camera.
#[doc(hidden)]
#[non_exhaustive]
pub struct MainCamera;

impl IsUiCamera for MainCamera {
    #[inline(always)]
    fn is_ui_camera() -> bool {
        false
    }
}

/// Item is drawn using the UI camera.
#[doc(hidden)]
#[non_exhaustive]
pub struct UiCamera;

impl IsUiCamera for UiCamera {
    #[inline(always)]
    fn is_ui_camera() -> bool {
        true
    }
}
