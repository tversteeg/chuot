//! Types and helpers for drawing on the GPU.

pub(crate) mod atlas;
pub(crate) mod component;
pub(crate) mod data;
pub(crate) mod gpu;
pub(crate) mod instance;
pub(crate) mod post_processing;
pub(crate) mod state;
pub(crate) mod texture;
pub(crate) mod uniform;

pub use texture::Texture;

use state::PREFERRED_TEXTURE_FORMAT;
use wgpu::Color;

/// Convert an `u32` color to a WGPU [`wgpu::Color`] taking in account sRGB.
fn u32_to_wgpu_color(argb: u32) -> Color {
    let a = ((argb & 0xFF000000) >> 24) as f64 / 255.0;
    let r = ((argb & 0x00FF0000) >> 16) as f64 / 255.0;
    let g = ((argb & 0x0000FF00) >> 8) as f64 / 255.0;
    let b = (argb & 0x000000FF) as f64 / 255.0;

    if PREFERRED_TEXTURE_FORMAT.is_srgb() {
        // Convert to sRGB space
        let a = a.powf(2.2);
        let r = r.powf(2.2);
        let g = g.powf(2.2);
        let b = b.powf(2.2);

        Color { a, r, g, b }
    } else {
        Color { a, r, g, b }
    }
}
