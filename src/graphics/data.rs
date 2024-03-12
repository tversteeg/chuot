//! Data types that can be send to the GPU.

use std::ops::Range;

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

/// Raw instance data.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Instances(Vec<[f32; 2]>);

impl Instances {
    /// Construct from a slice of positions.
    pub fn from_slice(instances: &[Vec2<f64>]) -> Self {
        Self(
            instances
                .iter()
                .map(|position| position.as_().into_array())
                .collect(),
        )
    }

    /// What part of the vector should be rendered (everything).
    pub fn range(&self) -> Range<u32> {
        0..self.0.len() as u32
    }

    /// WGPU descriptor.
    pub fn descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: 0,
                // Must be the next one of `TexturedVertex`
                shader_location: 2,
            }],
        }
    }
}
