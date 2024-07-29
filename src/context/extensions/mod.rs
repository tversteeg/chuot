//! Different traits for extending context structs.

pub(crate) mod camera;
pub(crate) mod draw;
pub(crate) mod rotate;
pub(crate) mod scale;
pub(crate) mod translate;

/// Empty struct that can be implemented for generic arguments with a typestate builder.
#[doc(hidden)]
#[non_exhaustive]
#[derive(Default)]
pub struct Empty;
