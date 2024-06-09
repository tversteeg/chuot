//! Image assets from disk.
//!
//! Shouldn't be used directly, use [`crate::sprite::Sprite`].

use std::io::Cursor;

use glamour::{Point2, Size2};
use hashbrown::HashMap;
use imgref::{ImgRef, ImgVec};
use png::{BitDepth, ColorType, Decoder, Transformations};

use crate::graphics::{
    atlas::{Atlas, AtlasRef},
    gpu::Gpu,
};

use super::Id;

/// Core of a sprite loaded from disk.
#[derive(Clone)]
pub(crate) struct Image {
    /// Image atlas ID.
    pub(crate) atlas_id: AtlasRef,

    /// Size of the image in pixels.
    pub(crate) size: Size2<u16>,

    /// Image RGBA pixels.
    #[cfg(feature = "read-image")]
    pub(crate) pixels: imgref::ImgVec<u32>,
}

/// How a diced image source should be translated into a result.
pub(crate) struct DicedImageMapping {
    /// Coordinates of the rectangle on the diced source.
    source: Point2<u16>,
    /// Coordinates of the rectangle on the target output complete image in relative coordinates.
    target: Point2<u16>,
    /// Size of both the rectangle on the source and on the target.
    size: Size2<u16>,
}

/// Handle reading and passing images to the GPU.
///
/// Basically the same as the asset managers but separated because of the complex state the images can be in before being uploaded to the GPU.
pub(crate) struct ImageManager {
    /// Collection of all images mapped by ID.
    images: HashMap<Id, Image>,
    /// Events that should be executed on the GPU for the image.
    gpu_commands: Vec<ImageGpuCommand>,
    /// Collection of all image pixel sources mapped by ID.
    #[cfg(feature = "read-image")]
    image_sources: HashMap<Id, ImgVec<u32>>,
}

impl ImageManager {
    /// Setup the image manager and upload the empty atlas to the GPU.
    #[inline]
    pub(crate) fn new() -> Self {
        let images = HashMap::new();
        let gpu_commands = Vec::new();
        #[cfg(feature = "read-image")]
        let image_sources = HashMap::new();

        Self {
            images,
            gpu_commands,
            #[cfg(feature = "read-image")]
            image_sources,
        }
    }

    /// Create and upload a new image from an array of pixels.
    #[inline]
    pub(crate) fn insert(&mut self, id: Id, source: ImgVec<u32>) {
        let size = Size2::new(source.width() as u16, source.height() as u16);

        // We can just upload as a single diced image to simplify
        self.insert_diced(
            id,
            source,
            vec![DicedImageMapping {
                source: Point2::ZERO,
                target: Point2::ZERO,
                size,
            }],
        )
    }

    /// Create and upload a new image from an array of pixels with diced mappings.
    #[inline]
    pub(crate) fn insert_diced(
        &mut self,
        id: Id,
        source: ImgVec<u32>,
        mappings: Vec<DicedImageMapping>,
    ) {
        // Calculate the total size of the mappings
        let width = mappings
            .iter()
            .map(|mapping| mapping.target.x + mapping.size.width)
            .max()
            .unwrap_or_default();
        let height = mappings
            .iter()
            .map(|mapping| mapping.target.y + mapping.size.height)
            .max()
            .unwrap_or_default();
        let size = Size2::new(width, height);

        // Create the image
        self.insert_empty(id.clone(), size);

        // Push the pixels
        self.update_diced(id, source, mappings);
    }

    /// Create and upload a new image from PNG bytes.
    #[inline]
    pub(crate) fn insert_png(&mut self, id: Id, png_bytes: Vec<u8>) {
        // Decode and insert as a regular image with the mappings
        self.insert(id, decode_png(png_bytes))
    }

    /// Create and upload a new image from diced PNG bytes.
    #[inline]
    pub(crate) fn insert_png_diced(
        &mut self,
        id: Id,
        diced_png_bytes: Vec<u8>,
        mappings: Vec<DicedImageMapping>,
    ) {
        // Decode and insert as a regular image with the mappings
        self.insert_diced(id, decode_png(diced_png_bytes), mappings)
    }

    /// Create and upload a new empty image.
    #[inline]
    pub(crate) fn insert_empty(&mut self, id: Id, size: Size2<u16>) {
        self.gpu_commands.push(ImageGpuCommand::Create { id, size });
    }

    /// Update the pixels of an image in a sub rectangle.
    #[inline]
    pub(crate) fn update(&mut self, id: Id, source: ImgVec<u32>, offset: Point2<u16>) {
        let size = Size2::new(source.width() as u16, source.height() as u16);

        // We can just upload as a single diced image to simplify
        self.update_diced(
            id,
            source,
            vec![DicedImageMapping {
                // Use the full source image
                source: Point2::ZERO,
                // Use the offset as the offset in the target
                target: offset,
                // Update the whole source sice
                size,
            }],
        )
    }

    /// Update the pixels of an image in a sub rectangle with diced mappings.
    #[inline]
    pub(crate) fn update_diced(
        &mut self,
        id: Id,
        source: ImgVec<u32>,
        mappings: Vec<DicedImageMapping>,
    ) {
        self.gpu_commands.push(ImageGpuCommand::Update {
            id,
            source,
            mappings,
        });
    }

    /// Replace the pixels of an image with another image.
    ///
    /// Will resize if sizes don't align.
    #[inline]
    pub(crate) fn replace(&mut self, id: Id, source: ImgVec<u32>) {
        let image = &self.images[&id];

        // Resize if the size mismatches
        if image.size.width as usize != source.width()
            || image.size.height as usize != source.height()
        {
            self.resize(
                id.clone(),
                Size2::new(source.width() as u16, source.height() as u16),
            );
        }

        // Write the new pixels
        self.update(id, source, Point2::ZERO);
    }

    /// Remove an image.
    #[inline]
    pub(crate) fn remove(&mut self, id: Id) {
        self.gpu_commands.push(ImageGpuCommand::Remove { id });
    }

    /// Resize the image.
    ///
    /// If the new size is bigger the contents of the resized pixels is undefined and should be filled manually.
    #[inline]
    pub(crate) fn resize(&mut self, id: Id, new_size: Size2<u16>) {
        self.gpu_commands
            .push(ImageGpuCommand::Resize { id, new_size });
    }

    /// Get the size of an image.
    #[inline]
    pub(crate) fn size(&self, id: Id) -> Size2<u16> {
        self.images[&id].size
    }

    /// Read the pixels of an image.
    #[cfg(feature = "read-image")]
    #[inline]
    pub(crate) fn read(&mut self, id: Id) -> ImgVec<u32> {
        todo!()
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

/// Different commands for image manipulation on the GPU.
pub(crate) enum ImageGpuCommand {
    /// Create a new empty image.
    Create {
        /// Image ID.
        id: Id,
        /// Size of the new image.
        size: Size2<u16>,
    },
    /// Remove the image.
    Remove {
        /// Image ID.
        id: Id,
    },
    /// Resize an existing image.
    Resize {
        /// Image ID.
        id: Id,
        /// Size of the image to resize to.
        new_size: Size2<u16>,
    },
    /// Update a portion of the image.
    Update {
        /// Image ID.
        id: Id,
        /// Source of the image to update.
        source: ImgVec<u32>,
        /// Mappings of each part to update in relative coordinates.
        mappings: Vec<DicedImageMapping>,
    },
}
