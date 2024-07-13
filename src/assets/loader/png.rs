//! PNG asset loading.

use std::io::Cursor;

use png::{BitDepth, ColorType, Decoder, Transformations};
use rgb::RGBA8;

use super::Loader;
use crate::assets::Id;

/// PNG asset loader.
///
/// Loader type returned is `(width, height, pixels)`, where `pixels` is an array of RGBA pixels.
#[non_exhaustive]
pub struct PngLoader;

impl Loader<(u32, u32, Vec<RGBA8>)> for PngLoader {
    const EXTENSION: &'static str = "png";

    #[inline]
    fn load(bytes: &[u8], id: &Id) -> (u32, u32, Vec<RGBA8>) {
        // Copy the bytes into a cursor
        let cursor = Cursor::new(bytes.to_vec());

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
        let mut reader = decoder.read_info().unwrap();

        // Ensure we can use the PNG colors
        let (color_type, bits) = reader.output_color_type();

        // Must be 8 bit RGBA or indexed
        assert!(
            color_type == ColorType::Rgba && bits == BitDepth::Eight,
            "PNG of asset with ID '{id}' is not 8 bit RGB with an alpha channel"
        );

        // Read the PNG
        let mut pixels = vec![RGBA8::default(); reader.output_buffer_size()];
        let info = reader
            .next_frame(bytemuck::cast_slice_mut(&mut pixels))
            .expect("Error reading image");

        (info.width, info.height, pixels)
    }
}
