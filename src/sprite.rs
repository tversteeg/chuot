//! Blittable sprite definitions.
//!
//! Can be loaded as an asset when the `asset` feature flag is set.

use std::ops::Range;

use assets_manager::{AnyCache, Asset, BoxedError, Compound, SharedString};

use glamour::{Size2, Vector2};
use miette::Result;
use serde::Deserialize;

use crate::{
    assets::image::Image,
    graphics::{
        data::TexturedVertex,
        instance::Instances,
        texture::{Texture, TextureRef},
        Render,
    },
};

/// Two triangles forming a rectangle to draw the sprite on the GPU.
const INDICES: &[u16] = &[0, 1, 3, 3, 1, 2];

/// Sprite that can be drawn on the  canvas.
#[derive(Debug)]
pub(crate) struct Sprite {
    /// Reference of the texture to render.
    image: TextureRef,
    /// Size of the image in pixels.
    size: Size2<u32>,
    /// Sprite metadata.
    metadata: SpriteMetadata,
    /// Instances to draw this frame.
    instances: Instances,
    /// Sprite needs to be updated on the GPU.
    is_dirty: bool,
    /// Graphics information for rendering the sprite.
    ///
    /// Only computed when actually used.
    contents: Option<[TexturedVertex; 4]>,
}

impl Sprite {
    /// Size of the image.
    pub(crate) fn size(&self) -> Size2<u32> {
        self.size
    }

    /// Compute the coordinates and UV for this sprite based on the offset.
    fn set_contents(&mut self) {
        // Only compute when something changed
        if !self.is_dirty && self.contents.is_some() {
            return;
        }

        let width = self.size.width as f32;
        let height = self.size.height as f32;
        let offset = self.metadata.offset.offset(Size2::new(width, height));

        self.contents = Some([
            TexturedVertex::new(Vector2::new(0.0, 0.0) + offset, 0.0, Vector2::new(0.0, 0.0)),
            TexturedVertex::new(
                Vector2::new(width, 0.0) + offset,
                0.0,
                Vector2::new(1.0, 0.0),
            ),
            TexturedVertex::new(
                Vector2::new(width, height) + offset,
                0.0,
                Vector2::new(1.0, 1.0),
            ),
            TexturedVertex::new(
                Vector2::new(0.0, height) + offset,
                0.0,
                Vector2::new(0.0, 1.0),
            ),
        ]);
    }
}

impl Compound for Sprite {
    fn load(cache: AnyCache, id: &SharedString) -> Result<Self, BoxedError> {
        // Load the image
        let image = cache.load_owned::<Image>(id)?;
        // Get the size for our sprite
        let size = image.size();

        // Store the image for uploading to the GPU, and keep the reference
        let image = crate::graphics::texture::upload(id.clone(), image);

        // Load the metadata
        let metadata = match cache.load::<SpriteMetadata>(id) {
            Ok(metadata) => metadata.read().clone(),
            Err(err) => {
                log::warn!("Error loading sprite metadata, using default: {err}");

                SpriteMetadata::default()
            }
        };

        let is_dirty = true;
        let contents = None;
        let instances = Instances::default();

        Ok(Self {
            size,
            image,
            metadata,
            is_dirty,
            contents,
            instances,
        })
    }
}

impl Render for Sprite {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn mark_clean(&mut self) {
        self.is_dirty = false;
    }

    fn instances_mut(&mut self) -> &mut Instances {
        &mut self.instances
    }

    fn range(&self) -> Range<u32> {
        0..INDICES.len() as u32
    }

    fn vertices(&self) -> &[TexturedVertex] {
        self.contents.as_ref().expect("Missing computed content")
    }

    fn indices(&self) -> &[u16] {
        INDICES
    }

    fn texture(&self) -> Option<&TextureRef> {
        Some(&self.image)
    }

    fn pre_render(&mut self) {
        // Calculate the contents if they haven't been set yet
        if self.contents.is_none() {
            self.set_contents();
        }
    }
}

/// Center of the sprite.
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
pub enum SpriteOffset {
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
    pub fn offset(&self, sprite_size: Size2) -> Vector2 {
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
