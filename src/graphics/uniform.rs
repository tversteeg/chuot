//! State for uniform bindings.

use std::{marker::PhantomData, ops::Index};

use bytemuck::NoUninit;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferUsages, Device,
    Queue, ShaderStages,
};

/// State data collection for uniforms for a shader.
///
/// Type must be aligned to 16 bytes.
pub(crate) struct UniformState<T: NoUninit> {
    pub(crate) bind_group_layout: BindGroupLayout,
    pub(crate) bind_group: BindGroup,
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
                visibility: ShaderStages::VERTEX,
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
            bind_group_layout,
            _phantom: PhantomData,
        }
    }
}

/// State data collection for uniform arrays for a shader.
///
/// Types must be aligned to 16 bytes.
pub(crate) struct UniformArrayState<T: NoUninit> {
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) bind_group: wgpu::BindGroup,
    /// Buffer on GPU.
    buffer: wgpu::Buffer,
    /// Buffer on CPU.
    local_buffer: Vec<T>,
}

impl<T: NoUninit> UniformArrayState<T> {
    /// Maximum bytes allowed by WebGL2.
    const MAX_BYTES: usize = 16384;

    /// Maximum items in array based on the maximum amount of bytes.
    const MAX_ITEMS: usize = Self::MAX_BYTES / std::mem::size_of::<T>();

    /// Upload a new empty uniform array.
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        // Ensure that the data has an alignment of 16 bytes, which is needed by WASM
        assert!(
            std::mem::size_of::<T>() % 16 == 0,
            "Uniform of type '{}' is not aligned to 16 bytes",
            std::any::type_name::<T>(),
        );

        // Create the empty buffer
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Array Buffer"),
            // Allow us to update this buffer
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            size: Self::MAX_BYTES as u64,
        });

        // Create the bind group layout for passing the screen size
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
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
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
        });

        // Create the local buffer
        let local_buffer = Vec::new();

        Self {
            bind_group,
            buffer,
            bind_group_layout,
            local_buffer,
        }
    }

    /// Create from a vector.
    ///
    /// Still needs to be uploaded.
    pub(crate) fn from_vec(items: Vec<T>, device: &wgpu::Device) -> UniformArrayState<T> {
        // Construct a new empty array
        let mut this = Self::new(device);

        // Upload the whole array
        this.local_buffer = items;

        this
    }

    /// Push the complete current buffer.
    #[inline]
    pub(crate) fn upload(&mut self, queue: &wgpu::Queue) {
        // Push all values
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.local_buffer));
    }

    /// Get the value for a rectangle.
    #[inline]
    pub(crate) fn get(&mut self, index: usize) -> &T {
        self.local_buffer
            .get(index)
            .expect("Index not found in uniform buffer")
    }

    /// Push a value to the array of the uniform.
    ///
    /// # Returns
    ///
    /// - The index of the pushed item.
    pub(crate) fn push(&mut self, value: &T, queue: &Queue) -> u64 {
        assert!(
            self.local_buffer.len() < Self::MAX_ITEMS,
            "Uniform value out ouf range"
        );

        // Convert value to bytes
        let data = bytemuck::bytes_of(value);

        // Calculate the index of the item that got pushed
        let index = self.local_buffer.len();

        // Update the buffer and push to queue
        queue.write_buffer(
            &self.buffer,
            (index * std::mem::size_of::<T>()) as u64,
            data,
        );

        // Update the local buffer
        self.local_buffer.push(*value);

        index as u64
    }
}

impl<T: NoUninit> Index<usize> for UniformArrayState<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.local_buffer
            .get(index)
            .expect("Uniform array value not set")
    }
}
