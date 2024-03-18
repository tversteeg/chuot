//! Type for exposing instancing functionality in the [`crate::graphics::Render`] trait.

use vek::Mat3;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

/// Raw representation of the instance type send to the GPU.
type Instance = [f32; 6];

/// Raw instance data.
///
/// Shouldn't be used directly outside of the internal rendering code.
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct Instances(Vec<Instance>);

impl Instances {
    /// Push an instance to draw this frame.
    pub(crate) fn push(&mut self, transformation: Mat3<f32>) {
        // Convert matrix to column-oriented array
        let array = transformation.into_col_array();

        // Only push the matrix vales we actually need
        // ([2], [5], [8]) is always (0, 0, 1)
        self.0
            .push([array[0], array[1], array[3], array[4], array[6], array[7]]);
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
    pub(crate) fn descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 0,
                    // Must be the next one of `TexturedVertex`
                    shader_location: 2,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 2]>() as u64,
                    shader_location: 3,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 4]>() as u64,
                    shader_location: 4,
                },
            ],
        }
    }
}
