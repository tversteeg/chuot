//! Image assets from disk.
//!
//! Shouldn't be used directly, use [`crate::sprite::Sprite`].

use std::{borrow::Cow, io::Cursor};

use assets_manager::{loader::Loader, Asset, BoxedError};
use miette::{Context, IntoDiagnostic};
use png::{ColorType, Decoder, Reader, Transformations};
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

        // Read the image
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
        decoder.set_transformations(Transformations::normalize_to_color8());
        let reader = decoder
            .read_info()
            .into_diagnostic()
            .wrap_err("Error reading PNG")?;

        // Ensure the image is RGBA, so we can directly copy the pixels into a GPU texture
        let color_type = reader.info().color_type;
        if color_type != ColorType::Rgba && color_type != ColorType::Indexed {
            Err(miette::miette!("PNG is not RGB with an alpha channel"))?;
        }

        Ok(Image { reader })
    }
}
