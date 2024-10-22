//! State for uniform bindings.

use std::{marker::PhantomData, ops::Index};

use bytemuck::NoUninit;
use wgpu::util::DeviceExt as _;

/// State data collection for uniforms for a shader.
///
/// Type must be aligned to 16 bytes.
pub(crate) struct UniformState<T: NoUninit> {
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) bind_group: wgpu::BindGroup,
    /// Store the type information.
    _phantom: PhantomData<T>,
}

impl<T: NoUninit> UniformState<T> {
    /// Upload a new uniform.
    pub(crate) fn new(device: &wgpu::Device, initial_value: &T) -> Self {
        // Convert initial value to bytes
        let contents = bytemuck::bytes_of(initial_value);

        // Ensure that the data has an alignment of 16 bytes, which is needed by WASM
        assert!(
            contents.len() % 16 == 0,
            "Uniform of type '{}' is not aligned to 16 bytes",
            std::any::type_name::<T>(),
        );

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents,
            // Allow us to update this buffer
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create the bind group layout for passing the screen size
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
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
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
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
pub(crate) struct UniformArrayState<T: NoUninit + Default> {
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) bind_group: wgpu::BindGroup,
    /// Buffer on GPU.
    buffer: wgpu::Buffer,
    /// Buffer on the CPU.
    local_buffer: Vec<T>,
}

impl<T: NoUninit + Default> UniformArrayState<T> {
    /// Maximum bytes allowed by WebGL2.
    const MAX_BYTES: u64 = 0x4000;
    /// Maximum items in array based on the maximum amount of bytes.
    const MAX_ITEMS: u64 = Self::MAX_BYTES / std::mem::size_of::<T>() as u64;

    /// Upload a new empty uniform array.
    pub(crate) fn new(
        preallocated_items: usize,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        // Ensure that the data has an alignment of 16 bytes, which is needed by WASM
        assert!(
            std::mem::size_of::<T>() % 16 == 0,
            "Uniform of type '{}' is not aligned to 16 bytes",
            std::any::type_name::<T>(),
        );
        assert!(preallocated_items < Self::MAX_ITEMS as usize);

        // Create the local CPU buffer
        let local_buffer = vec![T::default(); preallocated_items];

        // Create the GPU buffer
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Array Buffer"),
            // Allow us to update this buffer
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            size: Self::MAX_BYTES,
        });

        // Convert preallocated array to bytes
        let data = bytemuck::cast_slice(&local_buffer);

        // Push the preallocated items to the GPU
        queue.write_buffer(&buffer, data.len() as u64, data);

        // Create the bind group layout for passing the screen size
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
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
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
        });

        Self {
            bind_group_layout,
            bind_group,
            buffer,
            local_buffer,
        }
    }

    /// Push and upload a value to the array of the uniform.
    pub(crate) fn push(&mut self, value: &T, queue: &wgpu::Queue) -> u64 {
        // Get the old index where the item will be pushed
        let index = self.local_buffer.len() as u64;

        assert!(index <= Self::MAX_ITEMS, "Uniform value out ouf range");

        // Push the new value
        self.local_buffer.push(*value);

        // Convert value to bytes
        let data = bytemuck::bytes_of(value);

        // Push the new value to the GPU
        queue.write_buffer(&self.buffer, index * std::mem::size_of::<T>() as u64, data);

        index
    }

    /// Set and upload a value to the array of the uniform.
    #[cfg(feature = "embed-assets")]
    pub(crate) fn set(&mut self, index: usize, value: &T, queue: &wgpu::Queue) {
        // Set the new value
        self.local_buffer[index] = *value;

        // Convert value to bytes
        let data = bytemuck::bytes_of(value);

        // Push the new value to the GPU
        queue.write_buffer(
            &self.buffer,
            (index * std::mem::size_of::<T>()) as u64,
            data,
        );
    }
}

impl<T: NoUninit + Default> Index<usize> for UniformArrayState<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.local_buffer
            .get(index)
            .expect("Uniform array value not set")
    }
}
