//! Shader asset loading.

use super::Loadable;
use crate::{
    assets::{Id, loader::Loader},
    context::ContextInner,
};

/// WGSL shader asset loader.
#[non_exhaustive]
pub struct ShaderLoader;

impl Loader<Shader> for ShaderLoader {
    const EXTENSION: &'static str = "wgsl";

    #[inline]
    fn load(bytes: &[u8], _id: &Id) -> Shader {
        // Convert raw bytes to a valid UTF-8 string
        let shader_source = String::from_utf8_lossy(bytes);

        Shader(shader_source.to_string())
    }
}

/// Shader asset that can be loaded with metadata.
#[derive(Clone)]
pub struct Shader(String);

impl Loadable for Shader {
    fn load_if_exists(id: &Id, ctx: &mut ContextInner) -> Option<Self>
    where
        Self: Sized,
    {
        // Load the source code from the loader
        let shader = ctx.asset_source.load_if_exists::<ShaderLoader, _>(id)?;

        // Upload it
        ctx.graphics.upload_shader(id, shader.0.clone());

        Some(shader)
    }
}
