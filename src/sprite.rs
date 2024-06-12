//! Blittable sprite definitions.

use std::io::Cursor;

use glam::Affine2;
use glamour::{Angle, AsRaw, Point2, Rect, Size2, Transform2, Vector2};
use hashbrown::HashMap;
use imgref::ImgVec;
use miette::Result;
use png::{BitDepth, ColorType, Decoder, Transformations};
use serde::{
    de::{Error, Unexpected},
    Deserialize, Deserializer,
};
use serde_untagged::UntaggedEnumVisitor;

use crate::{
    assets::{loader::toml::TomlLoader, AssetSource, Id, Loadable},
    graphics::{atlas::AtlasRef, command::GpuCommand, data::TexturedVertex, instance::Instances},
};

/// Sprite that can be drawn on the  canvas.
pub(crate) struct Sprite {
    /// Reference to the image in the atlas.
    atlas_ref: AtlasRef,
    /// Sub rectangle of the sprite to draw, can be used to split a sprite sheet.
    sub_rectangle: Rect,
    /// Sprite metadata.
    metadata: SpriteMetadata,
    /// Full original sprite size in pixels.
    size: Size2,
}

impl Sprite {
    /// Split into equal horizontal parts.
    pub(crate) fn horizontal_parts(&self, part_width: f32) -> Vec<Self> {
        // Ensure that the image can be split into equal parts
        assert!(
            self.sub_rectangle.width() % part_width == 0.0,
            "Cannot split image into equal horizontal parts of {part_width} pixels"
        );

        // How many images we need to make
        let sub_images = (self.sub_rectangle.width() / part_width) as usize;

        (0..sub_images)
            .map(|index| {
                // Use the same sub rectangle only changing the position and size
                let mut sub_rectangle = self.sub_rectangle;
                sub_rectangle.origin.x += part_width * index as f32;
                sub_rectangle.size.width = part_width;

                let metadata = self.metadata.clone();
                let size = self.size;
                let atlas_ref = self.atlas_ref;

                Self {
                    atlas_ref,
                    sub_rectangle,
                    metadata,
                    size,
                }
            })
            .collect()
    }

    /// Draw the sprite if the texture is already uploaded.
    #[inline]
    pub(crate) fn draw(&self, position: Vector2, rotation: Angle, instances: &mut Instances) {
        instances.push(
            self.matrix(position, rotation),
            self.sub_rectangle,
            self.atlas_ref,
        );
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

            (transform, self.sub_rectangle, self.atlas_ref)
        }));
    }

    /// Get the size of the sprite in pixels.
    #[inline]
    pub(crate) const fn size(&self) -> Size2 {
        self.size
    }

    /// Read the pixels for this sprite.
    #[inline]
    #[cfg(feature = "read-image")]
    pub(crate) fn pixels(&self) -> Vec<u32> {
        let Rect {
            origin: Point2 { x, y },
            size: Size2 { width, height },
        } = self.sub_rectangle;

        assert!(
            x >= 0.0 && y >= 0.0,
            "Image subrectangle cannot contain negative coordinates"
        );
        assert!(
            width >= 0.0 && height >= 0.0,
            "Image size cannot be negative"
        );

        /*
        let (sub_image, width, height) = self
            .image
            .pixels
            .sub_image(x as usize, y as usize, width as usize, height as usize)
            .to_contiguous_buf();

        // Check we get the correct sub image
        assert_eq!(width, self.size.width as usize);
        assert_eq!(height, self.size.height as usize);

        sub_image.into_owned()
        */

        todo!()
    }

    /// Calculate the transformation matrix.
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
    pub(crate) const fn vertices() -> [TexturedVertex; 4] {
        [
            TexturedVertex::new(Vector2::new(0.0, 0.0), 0.0, Vector2::new(0.0, 0.0)),
            TexturedVertex::new(Vector2::new(1.0, 0.0), 0.0, Vector2::new(1.0, 0.0)),
            TexturedVertex::new(Vector2::new(1.0, 1.0), 0.0, Vector2::new(1.0, 1.0)),
            TexturedVertex::new(Vector2::new(0.0, 1.0), 0.0, Vector2::new(0.0, 1.0)),
        ]
    }

    /// Indices for the instanced sprite quad.
    pub(crate) const fn indices() -> [u16; 6] {
        [0, 1, 3, 3, 1, 2]
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
            Self::Middle => Vector2::new(-sprite_size.width / 2.0, -sprite_size.height / 2.0),
            Self::MiddleTop => Vector2::new(-sprite_size.width / 2.0, 0.0),
            Self::LeftTop => Vector2::ZERO,
            Self::Custom(offset) => -*offset,
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
    #[inline]
    fn load_if_exists(id: &Id, asset_source: &AssetSource) -> Option<Self>
    where
        Self: Sized,
    {
        asset_source.load_if_exists::<TomlLoader, _>(id)
    }
}

/// How a diced image source should be translated into a result.
pub(crate) struct ImageMapping {
    /// Coordinates of the rectangle on the diced source.
    source: Point2,
    /// Coordinates of the rectangle on the target output complete image in relative coordinates.
    target: Point2,
    /// Size of both the rectangle on the source and on the target.
    size: Size2,
}

/// Handle reading and passing sprites to the GPU.
///
/// Basically the same as the asset managers but separated because of the complex state the images can be in before being uploaded to the GPU.
pub(crate) struct SpriteManager {
    /// Collection of all sprites mapped by ID.
    sprites: HashMap<Id, Sprite>,
    /// Collection of all image pixel sources mapped by ID.
    #[cfg(feature = "read-image")]
    sources: HashMap<Id, ImgVec<u32>>,
}

impl SpriteManager {
    /// Setup the image manager and upload the empty atlas to the GPU.
    #[inline]
    pub(crate) fn new() -> Self {
        let sizes = HashMap::new();
        #[cfg(feature = "read-image")]
        let sources = HashMap::new();

        Self {
            sprites: sizes,
            #[cfg(feature = "read-image")]
            sources,
        }
    }

    /// Create and upload a new image from an array of pixels.
    #[inline]
    pub(crate) fn insert(
        &mut self,
        id: Id,
        source: ImgVec<u32>,
        gpu_command_queue: &mut Vec<GpuCommand>,
    ) {
        let size = Size2::new(source.width() as f32, source.height() as f32);

        // We can just upload as a single diced image to simplify
        self.insert_diced(
            id,
            source,
            vec![ImageMapping {
                source: Point2::ZERO,
                target: Point2::ZERO,
                size,
            }],
            gpu_command_queue,
        )
    }

    /// Create and upload a new image from an array of pixels with diced mappings.
    #[inline]
    pub(crate) fn insert_diced(
        &mut self,
        id: Id,
        source: ImgVec<u32>,
        mappings: Vec<ImageMapping>,
        gpu_command_queue: &mut Vec<GpuCommand>,
    ) {
        // Calculate the total size of the mappings
        let width = mappings.iter().fold(0.0_f32, |init, mapping| {
            init.max(mapping.target.x + mapping.size.width)
        });
        let height = mappings.iter().fold(0.0_f32, |init, mapping| {
            init.max(mapping.target.y + mapping.size.height)
        });
        let size = Size2::new(width, height);

        // Create the image
        self.insert_empty(id.clone(), size, gpu_command_queue);

        // Push the pixels
        self.update_diced(id, source, mappings, gpu_command_queue);
    }

    /// Create and upload a new image from PNG bytes.
    #[inline]
    pub(crate) fn insert_png(
        &mut self,
        id: Id,
        png_bytes: Vec<u8>,
        gpu_command_queue: &mut Vec<GpuCommand>,
    ) {
        // Decode and insert as a regular image with the mappings
        self.insert(id, decode_png(png_bytes), gpu_command_queue)
    }

    /// Create and upload a new image from diced PNG bytes.
    #[inline]
    pub(crate) fn insert_png_diced(
        &mut self,
        id: Id,
        diced_png_bytes: Vec<u8>,
        mappings: Vec<ImageMapping>,
        gpu_command_queue: &mut Vec<GpuCommand>,
    ) {
        // Decode and insert as a regular image with the mappings
        self.insert_diced(id, decode_png(diced_png_bytes), mappings, gpu_command_queue)
    }

    /// Create and upload a new empty image.
    #[inline]
    pub(crate) fn insert_empty(
        &mut self,
        id: Id,
        size: Size2,
        gpu_command_queue: &mut Vec<GpuCommand>,
    ) {
        // Keep track of the image
        self.sprites.insert(
            id.clone(),
            Sprite {
                atlas_ref: todo!(),
                sub_rectangle: todo!(),
                metadata: SpriteMetadata::default(),
                size: todo!(),
            },
        );

        // Keep track of the image source if needed
        #[cfg(feature = "read-image")]
        self.sources.insert(
            id.clone(),
            ImgVec::new(
                vec![0_u32; size.width as usize * size.height as usize],
                size.width as usize,
                size.height as usize,
            ),
        );

        // Push to the GPU
        gpu_command_queue.push(GpuCommand::CreateImage { id, size });
    }

    /// Update the pixels of an image in a sub rectangle.
    #[inline]
    pub(crate) fn update(
        &mut self,
        id: Id,
        source: ImgVec<u32>,
        offset: Point2,
        gpu_command_queue: &mut Vec<GpuCommand>,
    ) {
        let size = Size2::new(source.width() as f32, source.height() as f32);

        // We can just upload as a single diced image to simplify
        self.update_diced(
            id,
            source,
            vec![ImageMapping {
                // Use the full source image
                source: Point2::ZERO,
                // Use the offset as the offset in the target
                target: offset,
                // Update the whole source sice
                size,
            }],
            gpu_command_queue,
        )
    }

    /// Update the pixels of an image in a sub rectangle with diced mappings.
    #[inline]
    pub(crate) fn update_diced(
        &mut self,
        id: Id,
        source: ImgVec<u32>,
        mappings: Vec<ImageMapping>,
        gpu_command_queue: &mut Vec<GpuCommand>,
    ) {
        // Push to the GPU
        gpu_command_queue.push(GpuCommand::UpdateImage {
            id,
            source,
            mappings,
        });

        // TODO: implement on read-image image
    }

    /// Replace the pixels of an image with another image.
    ///
    /// Will resize if sizes don't align.
    #[inline]
    pub(crate) fn replace(
        &mut self,
        id: Id,
        source: ImgVec<u32>,
        gpu_command_queue: &mut Vec<GpuCommand>,
    ) {
        // Resize if the size mismatches
        let sprite = &self.sprites[&id];
        if sprite.size.width as usize != source.width()
            || sprite.size.height as usize != source.height()
        {
            self.resize(
                id.clone(),
                Size2::new(source.width() as f32, source.height() as f32),
                gpu_command_queue,
            );
        }

        // Write the new pixels
        self.update(id, source, Point2::ZERO, gpu_command_queue);
    }

    /// Remove an image.
    #[inline]
    pub(crate) fn remove(&mut self, id: Id, gpu_command_queue: &mut Vec<GpuCommand>) {
        gpu_command_queue.push(GpuCommand::RemoveImage { id });
    }

    /// Resize the image.
    ///
    /// If the new size is bigger the contents of the resized pixels is undefined and should be filled manually.
    #[inline]
    pub(crate) fn resize(
        &mut self,
        id: Id,
        new_size: Size2,
        gpu_command_queue: &mut Vec<GpuCommand>,
    ) {
        gpu_command_queue.push(GpuCommand::ResizeImage { id, new_size });
    }

    /// Get the size of an image.
    #[inline]
    pub(crate) fn size(&self, id: Id) -> Size2 {
        self.sprites[&id].size
    }

    /// Read the pixels of an image.
    #[cfg(feature = "read-image")]
    #[inline]
    pub(crate) fn read(&mut self, id: Id) -> &'_ ImgVec<u32> {
        &self.sources[&id]
    }
}

/// Decode a PNG.
fn decode_png(bytes: Vec<u8>) -> ImgVec<u32> {
    // Copy the bytes into a cursor
    let cursor = Cursor::new(bytes);

    // Decode the PNG
    let mut decoder = Decoder::new(cursor);

    // Discard text chunks
    decoder.set_ignore_text_chunk(true);
    // Make it faster by not checking if it's correct
    decoder.ignore_checksums(true);

    // Convert indexed images to RGBA
    decoder.set_transformations(Transformations::normalize_to_color8() | Transformations::ALPHA);

    // Start parsing the PNG
    let mut reader = decoder.read_info().expect("Error reading PNG");

    // Ensure we can use the PNG colors
    let (color_type, bits) = reader.output_color_type();

    // Must be 8 bit RGBA or indexed
    assert!(
        color_type == ColorType::Rgba && bits == BitDepth::Eight,
        "PNG is not 8 bit RGB with an alpha channel"
    );

    // Read the PNG
    let mut buf = vec![0_u32; reader.output_buffer_size()];
    let info = reader
        .next_frame(bytemuck::cast_slice_mut(&mut buf))
        .expect("Error reading image");

    // Convert to image
    ImgVec::new(buf, info.width as usize, info.height as usize)
}
