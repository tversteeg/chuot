//! Custom shader.

use super::Empty;

/// Allow selecting a custom shader.
pub trait Shader<'path>: Sized {
    /// Get the result struct that can be used to obtain a value.
    fn default_or_id(self) -> ApplyShader<'path>;
}

/// Custom shader.
#[doc(hidden)]
#[derive(Copy, Clone, Default)]
pub struct ApplyShader<'path> {
    /// Shader ID.
    pub(crate) path: Option<&'path str>,
}

impl<'path> ApplyShader<'path> {
    /// Instantiate a new shader.
    #[must_use]
    pub(crate) const fn new(path: &'path str) -> Self {
        Self { path: Some(path) }
    }
}

impl<'path> Shader<'path> for ApplyShader<'path> {
    #[inline]
    #[must_use]
    fn default_or_id(self) -> Self {
        self
    }
}

impl<'path> Shader<'path> for Empty {
    #[inline]
    #[must_use]
    fn default_or_id(self) -> ApplyShader<'path> {
        ApplyShader { path: None }
    }
}
