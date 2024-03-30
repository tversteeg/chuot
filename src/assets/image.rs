//! Image assets from disk.
//!
//! Shouldn't be used directly, use [`crate::sprite::Sprite`].

use std::{borrow::Cow, io::Cursor};

use assets_manager::{loader::Loader, Asset, BoxedError};
use glamour::Size2;
use miette::{Context, IntoDiagnostic};
use png::{BitDepth, ColorType, Decoder, Reader, Transformations};

use crate::graphics::texture::Texture;

/// Core of a sprite loaded from disk.
pub enum Image {
    /// Image is raw PNG bytes inside a reader.
    Png {
        /// PNG image reader.
        reader: Box<Reader<Cursor<Vec<u8>>>>,
    },
    /// Image is a raw ARGB byte buffer.
    Raw {
        /// Size of the image in pixels.
        size: Size2<u32>,
        /// Raw pixels.
        data: Vec<u32>,
    },
}

impl Image {
    /// Read the image and split into equal horizontal parts.
    pub(crate) fn into_horizontal_parts(self, part_width: u32) -> Vec<Image> {
        let size = self.size();

        // Ensure that the image can be split into equal parts
        assert!(
            size.width % part_width == 0,
            "Cannot split image into equal horizontal parts of {part_width} pixels"
        );

        // Get the raw pixels, by either reading the PNG or collecting the already raw data
        let raw_bytes = self.into_rgba_image();

        // Loop over each section, recreating the data
        let (width, height) = (size.width as usize, size.height as usize);
        let sub_images = size.width / part_width;
        (0..sub_images)
            .map(|index| {
                // Setup the buffer
                let pixels = (part_width * size.height) as usize;
                let mut data = vec![0u32; pixels];

                // Copy the image slices
                for y in 0..height {
                    let bytes_to_copy = part_width as usize;

                    let src_start = y * width + (index * part_width) as usize;
                    let src_end = src_start + bytes_to_copy;

                    let dst_start = y * (part_width as usize);
                    let dst_end = dst_start + bytes_to_copy;

                    data[dst_start..dst_end].copy_from_slice(&raw_bytes[src_start..src_end]);
                }

                // Create the new image
                Image::Raw {
                    size: Size2::new(part_width, size.height),
                    data,
                }
            })
            .collect()
    }
}

impl Asset for Image {
    // We only support PNG images currently
    const EXTENSION: &'static str = "png";

    type Loader = ImageLoader;
}

impl Texture for Image {
    fn size(&self) -> Size2<u32> {
        match self {
            Image::Png { reader } => {
                let info = reader.info();

                Size2::new(info.width, info.height)
            }
            Image::Raw { size, .. } => *size,
        }
    }

    fn into_rgba_image(self) -> Vec<u32> {
        match self {
            Image::Png { mut reader } => {
                // Allocate the output buffer
                let mut buf = vec![0; reader.output_buffer_size()];

                // Read the bytes into the buffer
                reader
                    .next_frame(&mut buf)
                    .expect("Error reading PNG frame");

                // Convert bytes to ARGB array
                bytemuck::cast_slice(&buf).to_vec()
            }
            Image::Raw { data, .. } => data,
        }
    }
}

/// Image asset loader.
pub struct ImageLoader;

impl Loader<Image> for ImageLoader {
    fn load(content: Cow<[u8]>, _ext: &str) -> Result<Image, BoxedError> {
        log::debug!("Decoding PNG");

        // Copy the bytes into a cursor
        let cursor = Cursor::new(content.to_vec());

        // Decode the PNG
        let mut decoder = Decoder::new(cursor);

        // Discard text chunks
        decoder.set_ignore_text_chunk(true);
        // Make it faster by not checking if it's correct
        decoder.ignore_checksums(true);

        // Convert indexed images to RGBA
        decoder
            .set_transformations(Transformations::normalize_to_color8() | Transformations::ALPHA);

        // Start parsing the PNG
        let reader = Box::new(
            decoder
                .read_info()
                .into_diagnostic()
                .wrap_err("Error reading PNG")?,
        );

        // Ensure we can use the PNG colors
        let (color_type, bits) = reader.output_color_type();

        // Must be 8 bit RGBA or indexed
        if color_type != ColorType::Rgba || bits != BitDepth::Eight {
            Err(miette::miette!(
                "PNG is not 8 bit RGB with an alpha channel"
            ))?;
        }

        Ok(Image::Png { reader })
    }
}
