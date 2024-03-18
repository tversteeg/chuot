//! Image assets from disk.
//!
//! Shouldn't be used directly, use [`crate::sprite::Sprite`].

use std::{borrow::Cow, io::Cursor};

use assets_manager::{loader::Loader, Asset, BoxedError};
use miette::{Context, IntoDiagnostic};
use png::{BitDepth, ColorType, Decoder, Reader, Transformations};
use vek::Extent2;

use crate::graphics::texture::Texture;

/// Core of a sprite loaded from disk.
pub(crate) struct Image {
    /// PNG image reader.
    reader: Reader<Cursor<Vec<u8>>>,
}

impl Asset for Image {
    // We only support PNG images currently
    const EXTENSION: &'static str = "png";

    type Loader = ImageLoader;
}

impl Texture for Image {
    fn size(&self) -> Extent2<u32> {
        let info = self.reader.info();

        Extent2::new(info.width, info.height)
    }

    fn to_rgba_image(&mut self) -> Vec<u8> {
        log::debug!("Reading PNG frame");

        // Allocate the output buffer
        let mut buf = vec![0; self.reader.output_buffer_size()];

        // Read the bytes into the buffer
        self.reader
            .next_frame(&mut buf)
            .expect("Error reading PNG frame");
        buf
    }
}

/// Image asset loader.
pub(crate) struct ImageLoader;

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
        let reader = decoder
            .read_info()
            .into_diagnostic()
            .wrap_err("Error reading PNG")?;

        // Ensure we can use the PNG colors
        let (color_type, bits) = reader.output_color_type();

        // Must be 8 bit RGBA or indexed
        if color_type != ColorType::Rgba || bits != BitDepth::Eight {
            Err(miette::miette!(
                "PNG is not 8 bit RGB with an alpha channel"
            ))?;
        }

        Ok(Image { reader })
    }
}
