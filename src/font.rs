//! Split a horizontal sprite of equal size text segments into a font.

use assets_manager::{loader::TomlLoader, AnyCache, Asset, BoxedError, Compound, SharedString};
use glamour::{Angle, Size2, Vector2};
use serde::Deserialize;

use crate::{
    assets::Image,
    graphics::instance::Instances,
    sprite::{Sprite, SpriteMetadata},
};

/// A font is just a collection of sprites.
pub(crate) struct Font {
    /// Sprites the font is made up of.
    pub(crate) sprites: Vec<Sprite>,
    /// Font metadata.
    glyph_size: Size2<f32>,
    /// Start char ASCII value.
    first_char: usize,
    /// End char ASCII value.
    last_char: usize,
}

impl Font {
    /// Draw the font on the screen.
    ///
    /// Will create an instance for every glyph sprite.
    pub(crate) fn draw(&mut self, position: Vector2, text: &str, instances: &mut Instances) {
        // Put the start position back 1 glyph since the first action is to move the cursor
        let start_position = position - Vector2::new(self.glyph_size.width, 0.0);
        let mut position = start_position;

        // Draw each character from the string
        text.chars().for_each(|ch| {
            let char_index = ch as usize;

            // Move the cursor
            position.x += self.glyph_size.width;

            // Don't draw characters that are not in the picture
            if char_index < self.first_char || char_index > self.last_char {
                if ch == '\n' {
                    position.x = start_position.x;
                    position.y += self.glyph_size.height;
                } else if ch == '\t' {
                    position.x += self.glyph_size.width * 3.0;
                }
                return;
            }

            // The sub rectangle offset of the character is based on the starting character and counted using the ASCII index
            let char_offset = char_index - self.first_char;

            // Draw the character
            self.sprites[char_offset].draw(position, Angle::from_radians(0.0), instances);
        });
    }
}

impl Compound for Font {
    fn load(cache: AnyCache, id: &SharedString) -> Result<Self, BoxedError> {
        // Load the image
        let image = cache.load_owned::<Image>(id)?;

        // Load the metadata
        let FontMetadata {
            glyph_size,
            first_char,
            last_char,
        } = cache.load_owned::<FontMetadata>(id)?;

        // Convert types used in calculations
        let glyph_size = Size2::new(glyph_size.width as f32, glyph_size.height as f32);
        let first_char = first_char as usize;
        let last_char = last_char as usize;

        // Split the image into multiple sub-images
        let sprites = image
            .into_horizontal_parts(glyph_size.width as u32)
            .into_iter()
            .map(|image| Sprite::from_image(image, SpriteMetadata::default()))
            .collect::<Vec<_>>();

        assert_eq!(
            last_char - first_char,
            sprites.len() - 1,
            "Font not properly defined, last char does not match length of parsed glyphs"
        );

        Ok(Self {
            sprites,
            glyph_size,
            last_char,
            first_char,
        })
    }
}

/// Font metadata to load from TOML.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FontMetadata {
    /// Width and height of a single character.
    pub(crate) glyph_size: Size2<u16>,
    /// First character in the image.
    #[serde(default = "FontMetadata::default_first_char")]
    pub(crate) first_char: char,
    /// Last character in the image.
    #[serde(default = "FontMetadata::default_last_char")]
    pub(crate) last_char: char,
}

impl FontMetadata {
    /// Default for the `first_char` field.
    fn default_first_char() -> char {
        '!'
    }

    /// Default for the `last_char` field.
    fn default_last_char() -> char {
        '~'
    }
}

impl Asset for FontMetadata {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}
