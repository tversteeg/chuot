//! Type for exposing instancing functionality in the [`crate::graphics::Render`] trait.

use bytemuck::{Pod, Zeroable};
use glam::{Affine2, Mat2, Vec2, Vec4};

use super::atlas::TextureRef;

/// WGPU attributes.
const ATTRIBUTES: &[wgpu::VertexAttribute] = &[
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: 0,
        // Must be the next one of `TexturedVertex`
        shader_location: 2,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: std::mem::offset_of!(Instance, translation) as u64,
        shader_location: 3,
    },
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x4,
        offset: std::mem::offset_of!(Instance, sub_rectangle) as u64,
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
    matrix: Mat2,
    /// Translation.
    translation: Vec2,
    /// Rectangle within the texture to render.
    sub_rectangle: Vec4,
    /// Texture to render.
    texture_ref: u16,
    /// Empty padding.
    _padding: [u8; 6],
}

impl From<(Affine2, Vec4, TextureRef)> for Instance {
    fn from((transformation, sub_rectangle, texture_ref): (Affine2, Vec4, TextureRef)) -> Self {
        Self {
            sub_rectangle,
            texture_ref,
            matrix: transformation.matrix2.into(),
            translation: transformation.translation.into(),
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
    pub(crate) fn push(
        &mut self,
        transformation: Affine2,
        sub_rectangle: (f32, f32, f32, f32),
        atlas_ref: TextureRef,
    ) {
        self.0
            .push((transformation, sub_rectangle.into(), atlas_ref).into());
    }

    /// Push an iterator of instances to draw this frame.
    pub(crate) fn extend(&mut self, items: impl Iterator<Item = (Affine2, Vec4, TextureRef)>) {
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
    pub(crate) const fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRIBUTES,
        }
    }
}
