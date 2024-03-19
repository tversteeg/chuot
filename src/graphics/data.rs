//! Data types that can be send to the GPU.

use bytemuck::{Pod, Zeroable};
use vek::Vec2;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

/// Position with a UV coordinate for rendering a vertex with a texture.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TexturedVertex {
    /// XYZ position.
    position: [f32; 3],
    /// UV coordinate.
    uv: [f32; 2],
}

impl TexturedVertex {
    /// Construct a new textured vertex from a 2D position, a Z index and a UV coordinate.
    pub fn new(position: Vec2<f32>, z: f32, uv: Vec2<f32>) -> Self {
        let position = [position.x, position.y, z];
        let uv = uv.into_array();

        Self { position, uv }
    }

    /// WGPU descriptor.
    pub fn descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 3]>() as u64,
                    //offset: std::mem::offset_of!(TexturedVertex, uv) as u64,
                    shader_location: 1,
                },
            ],
        }
    }
}

/// Screen info uniform information.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub(crate) struct ScreenInfo {
    /// Output buffer size.
    pub buffer_size: [f32; 2],
    /// Upscaling factor.
    pub upscale_factor: f32,
    /// Unused data for padding.
    pub _padding: f32,
}
