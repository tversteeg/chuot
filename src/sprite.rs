//! Blittable sprite definitions.
//!
//! Can be loaded as an asset when the `asset` feature flag is set.

use std::ops::Range;

use assets_manager::{AnyCache, Asset, BoxedError, Compound, SharedString};

use miette::{Context, IntoDiagnostic, Result};
use serde::Deserialize;
use vek::{Extent2, Vec2};

use crate::{
    assets::image::Image,
    graphics::{
        data::TexturedVertex,
        render::Render,
        texture::{Texture, TextureRef},
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
    size: Extent2<u32>,
    /// Sprite metadata.
    metadata: SpriteMetadata,
    /// Instances of the sprite to render.
    instances: Vec<Vec2<f64>>,
    /// Sprite needs to be updated on the GPU.
    is_dirty: bool,
    /// Graphics information for rendering the sprite.
    ///
    /// Only computed when actually used.
    contents: Option<[TexturedVertex; 4]>,
}

impl Sprite {
    /// Draw the sprite.
    ///
    /// This will add it to the list of instances.
    pub(crate) fn render(&mut self, offset: Vec2<f64>) {
        self.instances.push(offset);
    }

    /// Size of the image.
    pub(crate) fn size(&self) -> Extent2<u32> {
        self.size
    }

    /// Remove all instances.
    pub(crate) fn clear_instances(&mut self) {
        self.instances.clear();
    }

    /// Compute the coordinates and UV for this sprite based on the offset.
    fn set_contents(&mut self) {
        // Only compute when something changed
        if !self.is_dirty && self.contents.is_some() {
            return;
        }

        let offset = self.metadata.offset.offset(self.size().as_()).as_();
        let size = self.size().as_();

        self.contents = Some([
            TexturedVertex::new(Vec2::new(0.0, 0.0) + offset, 0.0, Vec2::new(0.0, 0.0)),
            TexturedVertex::new(Vec2::new(size.w, 0.0) + offset, 0.0, Vec2::new(1.0, 0.0)),
            TexturedVertex::new(Vec2::new(size.w, size.h) + offset, 0.0, Vec2::new(1.0, 1.0)),
            TexturedVertex::new(Vec2::new(0.0, size.h) + offset, 0.0, Vec2::new(0.0, 1.0)),
        ]);
    }
}

impl Compound for Sprite {
    fn load(cache: AnyCache, id: &SharedString) -> Result<Self, BoxedError> {
        // Load the image
        let image = cache
            .load_owned::<Image>(id)
            .into_diagnostic()
            .wrap_err("Error loading image for sprite")?;
        // Get the size for our sprite
        let size = image.size();

        // Store the image for uploading to the GPU, and keep the reference
        let image = crate::graphics::texture::upload(id.clone(), image);

        // Load the metadata
        let metadata = cache
            .load::<SpriteMetadata>(id)
            .into_diagnostic()
            .wrap_err("Error loading metadata for sprite")?
            .read()
            .clone();

        let is_dirty = true;
        let contents = None;
        let instances = Vec::new();

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

    fn instances(&self) -> &[Vec2<f64>] {
        &self.instances
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

    fn post_render(&mut self) {
        // Reset the instances for next frame
        self.instances.clear();
    }
}

/// Center of the sprite.
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
pub enum SpriteOffset {
    /// Middle of the sprite will be rendered at `(0, 0)`.
    #[default]
    Middle,
    /// Horizontal middle and vertical top will be rendered at `(0, 0)`.
    MiddleTop,
    /// Left top of the sprite will be rendered at `(0, 0)`.
    LeftTop,
    /// Sprite will be offset with the custom coordinates counting from the left top.
    Custom(Vec2<f64>),
}

impl SpriteOffset {
    /// Get the offset based on the sprite size.
    pub fn offset(&self, sprite_size: Extent2<f64>) -> Vec2<f64> {
        match self {
            SpriteOffset::Middle => Vec2::new(-sprite_size.w / 2.0, -sprite_size.h / 2.0),
            SpriteOffset::MiddleTop => Vec2::new(-sprite_size.w / 2.0, 0.0),
            SpriteOffset::LeftTop => Vec2::zero(),
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
