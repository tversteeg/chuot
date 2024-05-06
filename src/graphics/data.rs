//! Data types that can be send to the GPU.

use bytemuck::{Pod, Zeroable};
use glamour::{Size2, Vector2, Vector3};
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
        offset: std::mem::offset_of!(TexturedVertex, uv) as u64,
        shader_location: 1,
    },
];

/// Position with a UV coordinate for rendering a vertex with a texture.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TexturedVertex {
    /// XYZ position.
    position: Vector3,
    /// UV coordinate.
    uv: Vector2,
}

impl TexturedVertex {
    /// Construct a new textured vertex from a 2D position, a Z index and a UV coordinate.
    pub const fn new(position: Vector2, z: f32, uv: Vector2) -> Self {
        let position = Vector3::new(position.x, position.y, z);

        Self { position, uv }
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
    /// Output buffer size.
    pub buffer_size: Size2,
    /// Unused data for padding.
    pub _padding: u64,
}
