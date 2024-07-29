//! Camera options.

/// Allow choosing which camera to use.
pub trait Camera: Sized {
    /// What type the builder will return for the UI camera.
    type IntoUi: Sized;
    /// What type the builder will return for the main game camera.
    type IntoMain: Sized;

    /// Use the UI camera instead of the regular game camera for transforming the drawable object.
    fn use_ui_camera(self) -> Self::IntoUi;

    /// Use the regular game camera instead of the UI camera for transforming the drawable object.
    fn use_main_camera(self) -> Self::IntoMain;
}

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
