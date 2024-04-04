//! Blittable sprite definitions.

use assets_manager::{AnyCache, Asset, BoxedError, Compound, SharedString};
use glam::Affine2;
use glamour::{Angle, AsRaw, Size2, Transform2, Vector2};
use miette::Result;
use serde::Deserialize;

use crate::{
    assets::Image,
    graphics::{
        data::TexturedVertex,
        instance::Instances,
        texture::{PendingOrUploaded, Texture},
    },
};

/// Sprite that can be drawn on the  canvas.
pub(crate) struct Sprite {
    /// Reference of the texture to render.
    pub(crate) image: PendingOrUploaded<Image>,
    /// Size of the image in a form we can calculate with without casting.
    size: Size2,
    /// Sprite metadata.
    metadata: SpriteMetadata,
}

impl Sprite {
    /// Create from an image.
    pub(crate) fn from_image(image: Image, metadata: SpriteMetadata) -> Self {
        // Mark the image as a texture not being uploaded yet
        let image = PendingOrUploaded::new(image);

        let size = image.size();
        let size = Size2::new(size.width as f32, size.height as f32);

        Self {
            image,
            size,
            metadata,
        }
    }

    /// Draw the sprite if the texture is already uploaded.
    #[inline]
    pub(crate) fn draw(&mut self, position: Vector2, rotation: Angle, instances: &mut Instances) {
        let Some(texture_ref) = self.image.try_as_ref() else {
            return;
        };

        instances.push(self.matrix(position, rotation), texture_ref);
    }

    /// Draw the sprites if the texture is already uploaded.
    #[inline]
    pub(crate) fn draw_multiple(
        &mut self,
        base_translation: Vector2,
        base_rotation: Angle,
        translations: impl Iterator<Item = Vector2>,
        instances: &mut Instances,
    ) {
        let Some(texture_ref) = self.image.try_as_ref() else {
            return;
        };

        // Calculate the base transformation
        let transform = self.matrix(base_translation, base_rotation);

        // Transform each instance on top of the base transformation
        instances.extend(translations.map(|translation| {
            let mut transform = transform;
            transform.translation += *translation.as_raw();

            (transform, texture_ref)
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

impl Compound for Sprite {
    fn load(cache: AnyCache, id: &SharedString) -> Result<Self, BoxedError> {
        // Load the image
        let image = cache.load_owned::<Image>(id)?;

        // Load the metadata
        let metadata = match cache.load::<SpriteMetadata>(id) {
            Ok(metadata) => metadata.read().clone(),
            Err(err) => {
                log::warn!("Error loading sprite metadata, using default: {err}");

                SpriteMetadata::default()
            }
        };

        Ok(Self::from_image(image, metadata))
    }
}

/// Center of the sprite.
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
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
            SpriteOffset::Custom(offset) => *offset,
        }
    }
}

/// Sprite metadata to load from TOML.
#[derive(Debug, Clone, Default, Deserialize, Asset)]
#[asset_format = "toml"]
pub(crate) struct SpriteMetadata {
    /// Pixel offset to render at.
    #[serde(default)]
    pub(crate) offset: SpriteOffset,
}
