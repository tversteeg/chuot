//! Type for exposing instancing functionality in the [`crate::graphics::Render`] trait.

use bytemuck::{Pod, Zeroable};
use glam::Affine2;
use glamour::{Matrix2, Vector2};

use super::texture::TextureRef;

/// WGPU attributes.
const ATTRIBUTES: &[wgpu::VertexAttribute] = &[
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 0,
        // Must be the next one of `TexturedVertex`
        shader_location: 2,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: std::mem::size_of::<[f32; 2]>() as u64,
        shader_location: 3,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: std::mem::offset_of!(Instance, translation) as u64,
        shader_location: 4,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Uint32,
        offset: std::mem::offset_of!(Instance, texture_ref) as u64,
        shader_location: 5,
    },
];

/// Raw representation of the instance type send to the GPU.
#[repr(C, align(16))]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
struct Instance {
    /// Rotation and skewing.
    matrix: Matrix2<f32>,
    /// Translation.
    translation: Vector2,
    /// Texture to render.
    texture_ref: u32,
    /// Alignment padding.
    _padding: u32,
}

impl From<(Affine2, TextureRef)> for Instance {
    fn from((transformation, texture_ref): (Affine2, TextureRef)) -> Self {
        Self {
            matrix: transformation.matrix2.into(),
            translation: transformation.translation.into(),
            texture_ref: texture_ref as u32,
            ..Default::default()
        }
    }
}

/// Raw instance data.
///
/// Shouldn't be used directly outside of the internal rendering code.
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub(crate) struct Instances(Vec<Instance>);

impl Instances {
    /// Push an instance to draw this frame.
    pub(crate) fn push(&mut self, transformation: Affine2, texture_ref: TextureRef) {
        self.0.push((transformation, texture_ref).into());
    }

    /// Push an iterator of instances to draw this frame.
    pub(crate) fn extend(&mut self, items: impl Iterator<Item = (Affine2, TextureRef)>) {
        self.0.extend(items.map(Into::<Instance>::into));
    }

    /// Remove all items.
    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }

    /// Get as raw bytes.
    pub(crate) fn bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.0)
    }

    /// Amount of instances to draw this frame.
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are any.
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// WGPU descriptor.
    pub(crate) fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRIBUTES,
        }
    }
}
