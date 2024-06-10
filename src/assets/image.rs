//! Image assets from disk.
//!
//! Shouldn't be used directly, use [`crate::sprite::Sprite`].

use std::io::Cursor;

use glamour::{Point2, Size2};
use hashbrown::HashMap;
use imgref::ImgVec;
use png::{BitDepth, ColorType, Decoder, Transformations};

use super::Id;

use crate::graphics::command::GpuCommand;

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
    sizes: HashMap<Id, Size2<u16>>,
    /// Collection of all image pixel sources mapped by ID.
    #[cfg(feature = "read-image")]
    sources: HashMap<Id, ImgVec<u32>>,
}

impl ImageManager {
    /// Setup the image manager and upload the empty atlas to the GPU.
    #[inline]
    pub(crate) fn new() -> Self {
        let sizes = HashMap::new();
        #[cfg(feature = "read-image")]
        let sources = HashMap::new();

        Self {
            sizes,
            #[cfg(feature = "read-image")]
            sources,
        }
    }

    /// Create and upload a new image from an array of pixels.
    #[inline]
    pub(crate) fn insert(
        &mut self,
        gpu_command_queue: &mut Vec<GpuCommand>,
        id: Id,
        source: ImgVec<u32>,
    ) {
        let size = Size2::new(source.width() as u16, source.height() as u16);

        // We can just upload as a single diced image to simplify
        self.insert_diced(
            gpu_command_queue,
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
        gpu_command_queue: &mut Vec<GpuCommand>,
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
        self.insert_empty(gpu_command_queue, id.clone(), size);

        // Push the pixels
        self.update_diced(gpu_command_queue, id, source, mappings);
    }

    /// Create and upload a new image from PNG bytes.
    #[inline]
    pub(crate) fn insert_png(
        &mut self,
        gpu_command_queue: &mut Vec<GpuCommand>,
        id: Id,
        png_bytes: Vec<u8>,
    ) {
        // Decode and insert as a regular image with the mappings
        self.insert(gpu_command_queue, id, decode_png(png_bytes))
    }

    /// Create and upload a new image from diced PNG bytes.
    #[inline]
    pub(crate) fn insert_png_diced(
        &mut self,
        gpu_command_queue: &mut Vec<GpuCommand>,
        id: Id,
        diced_png_bytes: Vec<u8>,
        mappings: Vec<DicedImageMapping>,
    ) {
        // Decode and insert as a regular image with the mappings
        self.insert_diced(gpu_command_queue, id, decode_png(diced_png_bytes), mappings)
    }

    /// Create and upload a new empty image.
    #[inline]
    pub(crate) fn insert_empty(
        &mut self,
        gpu_command_queue: &mut Vec<GpuCommand>,
        id: Id,
        size: Size2<u16>,
    ) {
        // Keep track of the image
        self.sizes.insert(id.clone(), size);

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
        gpu_command_queue: &mut Vec<GpuCommand>,
        id: Id,
        source: ImgVec<u32>,
        offset: Point2<u16>,
    ) {
        let size = Size2::new(source.width() as u16, source.height() as u16);

        // We can just upload as a single diced image to simplify
        self.update_diced(
            gpu_command_queue,
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
        gpu_command_queue: &mut Vec<GpuCommand>,
        id: Id,
        source: ImgVec<u32>,
        mappings: Vec<DicedImageMapping>,
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
        gpu_command_queue: &mut Vec<GpuCommand>,
        id: Id,
        source: ImgVec<u32>,
    ) {
        // Resize if the size mismatches
        let size = &self.sizes[&id];
        if size.width as usize != source.width() || size.height as usize != source.height() {
            self.resize(
                gpu_command_queue,
                id.clone(),
                Size2::new(source.width() as u16, source.height() as u16),
            );
        }

        // Write the new pixels
        self.update(gpu_command_queue, id, source, Point2::ZERO);
    }

    /// Remove an image.
    #[inline]
    pub(crate) fn remove(&mut self, gpu_command_queue: &mut Vec<GpuCommand>, id: Id) {
        gpu_command_queue.push(GpuCommand::RemoveImage { id });
    }

    /// Resize the image.
    ///
    /// If the new size is bigger the contents of the resized pixels is undefined and should be filled manually.
    #[inline]
    pub(crate) fn resize(
        &mut self,
        gpu_command_queue: &mut Vec<GpuCommand>,
        id: Id,
        new_size: Size2<u16>,
    ) {
        gpu_command_queue.push(GpuCommand::ResizeImage { id, new_size });
    }

    /// Get the size of an image.
    #[inline]
    pub(crate) fn size(&self, id: Id) -> Size2<u16> {
        self.sizes[&id]
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
