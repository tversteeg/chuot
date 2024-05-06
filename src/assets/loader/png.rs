//! PNG asset loading.

use std::io::Cursor;

use png::{BitDepth, ColorType, Decoder, Reader, Transformations};

use super::Loader;

/// PNG reader type, returned from the loader.
pub type PngReader = Reader<Cursor<Vec<u8>>>;

/// PNG asset loader.
///
/// Doesn't fully parse the PNG but loads a reader.
#[non_exhaustive]
pub struct PngLoader;

impl Loader<PngReader> for PngLoader {
    const EXTENSION: &'static str = "png";

    #[inline]
    fn load(bytes: &[u8]) -> PngReader {
        log::debug!("Decoding PNG");

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
        let reader = decoder.read_info().expect("Error reading PNG");

        // Ensure we can use the PNG colors
        let (color_type, bits) = reader.output_color_type();

        // Must be 8 bit RGBA or indexed
        assert!(
            color_type == ColorType::Rgba && bits == BitDepth::Eight,
            "PNG is not 8 bit RGB with an alpha channel"
        );

        reader
    }
}
