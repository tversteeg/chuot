//! Data types that can be send to the GPU.

use bytemuck::{Pod, Zeroable};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

/// WGPU attributes.
const ATTRIBUTES: &[VertexAttribute] = &[
    VertexAttribute {
        format: VertexFormat::Float32x3,
        offset: 0,
        shader_location: 0,
    },
    VertexAttribute {
        format: VertexFormat::Float32x2,
        offset: std::mem::offset_of!(TexturedVertex, u) as u64,
        shader_location: 1,
    },
];

/// Position with a UV coordinate for rendering a vertex with a texture.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TexturedVertex {
    /// X position.
    x: f32,
    /// Y position.
    y: f32,
    /// Z position.
    z: f32,
    /// U texture coordinate.
    u: f32,
    /// V texture coordinate.
    v: f32,
}

impl TexturedVertex {
    /// Construct a new textured vertex from a 2D position, a Z index and a UV coordinate.
    #[allow(clippy::many_single_char_names)]
    pub const fn new(x: f32, y: f32, z: f32, u: f32, v: f32) -> Self {
        Self { x, y, z, u, v }
    }

    /// WGPU descriptor.
    pub(crate) const fn descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

/// Screen info uniform information.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub(crate) struct ScreenInfo {
    /// Output buffer width.
    pub width: f32,
    /// Output buffer height.
    pub height: f32,
    /// Output buffer width / 2.
    pub half_width: f32,
    /// Output buffer height / 2.
    pub half_height: f32,
}
