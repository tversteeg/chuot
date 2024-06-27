//! Any asset that can be loaded with [`crate::assets::loader::Loader`] implementations.

pub(crate) mod audio;
pub(crate) mod font;
pub(crate) mod sprite;

use downcast_rs::Downcast;

use crate::context::ContextInner;

use super::Id;

/// Any asset that's loadable from any amount of binary files.
///
/// # Example
///
/// ```
/// use chuot::assets::{loader::ron::RonLoader, AssetSource, Id, Loadable};
/// use nanoserde::DeRon;
///
/// /// We define a custom settings object that will be loaded from a '.ron' file.
/// #[derive(DeRon)]
/// struct Settings {
///     property_a: String,
///     property_b: i32,
/// }
///
/// impl Loadable for Settings {
///     fn load_if_exists(id: &Id, assets: &AssetSource) -> Option<Self>
///     where
///         Self: Sized,
///     {
///         // Use the RON loader to load our asset
///         assets.load_if_exists::<RonLoader, _>(id)
///     }
/// }
/// ```
pub trait Loadable: Downcast {
    /// Convert a file object to this type if it exists, if it doesn't return `None`.
    ///
    /// # Panics
    ///
    /// - When parsing binary bytes of asset into type fails.
    fn load_if_exists(id: &Id, ctx: &mut ContextInner) -> Option<Self>
    where
        Self: Sized;

    /// Convert a file object to this type.
    ///
    /// # Panics
    ///
    /// - When parsing binary bytes of asset into type fails.
    /// - When asset does not exist in the source.
    #[inline]
    #[must_use]
    fn load(id: &Id, ctx: &mut ContextInner) -> Self
    where
        Self: Sized,
    {
        Self::load_if_exists(id, ctx).map_or_else(
            || panic!("Error loading asset: '{id}' does not exist"),
            |asset| asset,
        )
    }

    /// Create a new runtime asset from the default value.
    #[inline]
    #[must_use]
    fn new() -> Self
    where
        Self: Default,
    {
        Self::default()
    }
}
downcast_rs::impl_downcast!(Loadable);
