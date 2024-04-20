//! Blittable sprite definitions.

use glam::Affine2;
use glamour::{Angle, AsRaw, Size2, Transform2, Vector2};
use miette::Result;
use serde::{
    de::{Error, Unexpected},
    Deserialize, Deserializer,
};
use serde_untagged::UntaggedEnumVisitor;

use crate::{
    assets::{image::Image, loader::toml::TomlLoader, AssetSource, Id, Loadable},
    graphics::{data::TexturedVertex, instance::Instances, texture::Texture},
};

/// Sprite that can be drawn on the  canvas.
pub(crate) struct Sprite {
    /// Reference of the texture to render.
    pub(crate) image: Image,
    /// Sprite metadata.
    metadata: SpriteMetadata,
    /// Size in pixels.
    size: Size2,
}

impl Sprite {
    /// Draw the sprite if the texture is already uploaded.
    #[inline]
    pub(crate) fn draw(&self, position: Vector2, rotation: Angle, instances: &mut Instances) {
        instances.push(self.matrix(position, rotation), self.image.atlas_id);
    }

    /// Draw the sprites if the texture is already uploaded.
    #[inline]
    pub(crate) fn draw_multiple(
        &self,
        base_translation: Vector2,
        base_rotation: Angle,
        translations: impl Iterator<Item = Vector2>,
        instances: &mut Instances,
    ) {
        // Calculate the base transformation
        let transform = self.matrix(base_translation, base_rotation);

        // Transform each instance on top of the base transformation
        instances.extend(translations.map(|translation| {
            let mut transform = transform;
            transform.translation += *translation.as_raw();

            (transform, self.image.atlas_id)
        }));
    }

    /// Get the size of the sprite in pixels.
    #[inline]
    pub(crate) fn size(&self) -> Size2 {
        self.size
    }

    /// Calculate the transformation matrix.
    #[inline]
    fn matrix(&self, translation: Vector2, rotation: Angle) -> Affine2 {
        let sprite_offset = self.metadata.offset.offset(self.size);

        // Draw with a more optimized version if no rotation needs to be applied
        if rotation.radians == 0.0 {
            Affine2::from_translation((sprite_offset + translation).into())
        } else {
            // First translate negatively from the center point
            let transform = Transform2::from_translation(sprite_offset)
                // Then apply the rotation so it rotates around the offset
                .then_rotate(rotation)
                // Finally move it to the correct position
                .then_translate(translation);

            Affine2::from_mat3(transform.matrix.into())
        }
    }

    /// Vertices for the instanced sprite quad.
    pub(crate) fn vertices() -> [TexturedVertex; 4] {
        [
            TexturedVertex::new(Vector2::new(0.0, 0.0), 0.0, Vector2::new(0.0, 0.0)),
            TexturedVertex::new(Vector2::new(1.0, 0.0), 0.0, Vector2::new(1.0, 0.0)),
            TexturedVertex::new(Vector2::new(1.0, 1.0), 0.0, Vector2::new(1.0, 1.0)),
            TexturedVertex::new(Vector2::new(0.0, 1.0), 0.0, Vector2::new(0.0, 1.0)),
        ]
    }

    /// Indices for the instanced sprite quad.
    pub(crate) fn indices() -> [u16; 6] {
        [0, 1, 3, 3, 1, 2]
    }
}

impl Loadable for Sprite {
    fn load_if_exists(id: &Id, asset_source: &AssetSource) -> Option<Self>
    where
        Self: Sized,
    {
        // Load the image
        let image = Image::load(id, asset_source);
        let size = Size2::new(image.size().width as f32, image.size().height as f32);

        // Load the metadata
        let metadata = match SpriteMetadata::load_if_exists(id, asset_source) {
            Some(metadata) => metadata,
            None => {
                log::warn!("Sprite metadata for '{id}' not found, using default");

                SpriteMetadata::default()
            }
        };

        Some(Self {
            image,
            metadata,
            size,
        })
    }
}

/// Center of the sprite.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum SpriteOffset {
    /// Middle of the sprite will be rendered at `(0, 0)`.
    Middle,
    /// Horizontal middle and vertical top will be rendered at `(0, 0)`.
    MiddleTop,
    /// Left top of the sprite will be rendered at `(0, 0)`.
    #[default]
    LeftTop,
    /// Sprite will be offset with the custom coordinates counting from the left top.
    Custom(Vector2),
}

impl SpriteOffset {
    /// Get the offset based on the sprite size.
    #[inline]
    pub(crate) fn offset(&self, sprite_size: Size2) -> Vector2 {
        match self {
            SpriteOffset::Middle => {
                Vector2::new(-sprite_size.width / 2.0, -sprite_size.height / 2.0)
            }
            SpriteOffset::MiddleTop => Vector2::new(-sprite_size.width / 2.0, 0.0),
            SpriteOffset::LeftTop => Vector2::ZERO,
            SpriteOffset::Custom(offset) => -*offset,
        }
    }
}

impl<'de> Deserialize<'de> for SpriteOffset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        UntaggedEnumVisitor::new()
            .string(|string| match string {
                "middle" | "Middle" => Ok(Self::Middle),
                "middle_top" | "Middle_Top" | "MiddleTop" => Ok(Self::MiddleTop),
                "left_top" | "Left_Top" | "LeftTop" => Ok(Self::LeftTop),
                _ => Err(Error::invalid_value(
                    Unexpected::Str(string),
                    &r#""middle" or "middle_top" or "left_top" or { x = .., y = .. }"#,
                )),
            })
            .map(|map| map.deserialize().map(Self::Custom))
            .deserialize(deserializer)
    }
}

/// Sprite metadata to load from TOML.
#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct SpriteMetadata {
    /// Pixel offset to render at.
    #[serde(default)]
    pub(crate) offset: SpriteOffset,
}

impl Loadable for SpriteMetadata {
    fn load_if_exists(id: &Id, asset_source: &AssetSource) -> Option<Self>
    where
        Self: Sized,
    {
        asset_source.load_if_exists::<TomlLoader, _>(id)
    }
}
