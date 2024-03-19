//! State for uniform bindings.

use std::marker::PhantomData;

use bytemuck::NoUninit;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages,
    Device, Queue, ShaderStages,
};

/// State data collection for uniforms for a shader.
///
/// Type must be aligned to 16 bytes.
pub(crate) struct UniformState<T: NoUninit> {
    pub(crate) bind_group_layout: BindGroupLayout,
    pub(crate) bind_group: BindGroup,
    buffer: Buffer,
    /// Store the type information.
    _phantom: PhantomData<T>,
}

impl<T: NoUninit> UniformState<T> {
    /// Upload a new uniform.
    pub(crate) fn new(device: &Device, initial_value: &T) -> Self {
        // Convert initial value to bytes
        let contents = bytemuck::bytes_of(initial_value);

        // Ensure that the data has an alignment of 16 bytes, which is needed by WASM
        assert!(
            contents.len() % 16 == 0,
            "Uniform of type '{}' is not aligned to 16 bytes",
            std::any::type_name::<T>(),
        );

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents,
            // Allow us to update this buffer
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create the bind group layout for passing the screen size
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Also already create the bind group, since it will be used without changing the size
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
        });

        Self {
            bind_group,
            buffer,
            bind_group_layout,
            _phantom: PhantomData,
        }
    }

    /// Update the value of the uniform.
    pub(crate) fn update(&mut self, value: &T, queue: &Queue) {
        // Convert value to bytes
        let data = bytemuck::bytes_of(value);

        // PUpdate the buffer and push to queue
        queue.write_buffer(&self.buffer, 0, data);
    }
}
