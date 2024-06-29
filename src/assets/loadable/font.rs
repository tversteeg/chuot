//! Font asset.

use nanoserde::DeRon;

use crate::{
    assets::{loader::ron::RonLoader, Id},
    context::ContextInner,
};

use super::{sprite::Sprite, Loadable};

/// Font asset that can be loaded with metadata.
///
/// A font is just a collection of sprites.
pub(crate) struct Font {
    /// Sprites the font is made up of.
    pub(crate) sprites: Vec<Sprite>,
    /// Font metadata.
    pub(crate) metadata: FontMetadata,
}

impl Loadable for Font {
    fn load_if_exists(id: &Id, ctx: &mut ContextInner) -> Option<Self> {
        // Load the base sprite, but don't load the sprite metadata because we can only load one RON file
        let base = Sprite::load_if_exists_without_metadata(id, ctx)?;

        // Load the metadata
        let metadata = FontMetadata::load_if_exists(id, ctx).unwrap_or_default();

        // Split the sprite into multiple sub-sprites for each character
        let sprites = base.horizontal_parts(metadata.glyph_width);

        assert_eq!(
            metadata.last_char - metadata.first_char,
            sprites.len() - 1,
            "Font not properly defined, last char does not match length of parsed glyphs"
        );

        Some(Self { sprites, metadata })
    }
}

/// Font metadata to load from RON.
#[derive(Debug, Clone, Copy, DeRon)]
pub struct FontMetadata {
    /// Width of a single character in pixels.
    pub(crate) glyph_width: f32,
    /// Height of a single character in pixels.
    pub(crate) glyph_height: f32,
    /// First character in the image.
    ///
    /// Uses the ASCII table, the default value is `33` which equals `'!'`.
    #[nserde(default = "'!' as usize")]
    pub(crate) first_char: usize,
    /// Last character in the image.
    ///
    /// Uses the ASCII table, the default value is `127` which equals `'~'`.
    #[nserde(default = "'~' as usize")]
    pub(crate) last_char: usize,
}

impl Loadable for FontMetadata {
    fn load_if_exists(id: &Id, ctx: &mut ContextInner) -> Option<Self> {
        ctx.asset_source.load_if_exists::<RonLoader, Self>(id)
    }
}

impl Default for FontMetadata {
    fn default() -> Self {
        Self {
            glyph_width: 8.0,
            glyph_height: 8.0,
            first_char: '!' as usize,
            last_char: '~' as usize,
        }
    }
}
