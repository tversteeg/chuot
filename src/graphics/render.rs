//! Expose render functionality on different types through traits.

use std::{borrow::Cow, marker::PhantomData, ops::Range};

use vek::Vec2;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BlendComponent, BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoder, Device, FragmentState, FrontFace, IndexFormat, LoadOp, MultisampleState,
    Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, StoreOp,
    TextureFormat, TextureView, TextureViewDescriptor, VertexState,
};

use super::{
    data::{Instances, TexturedVertex},
    texture::Texture,
};

/// Allow something to be rendered on the GPU.
pub trait Render {
    /// Definition of the vertex buffer.
    fn vertex_buffer_descriptor(&mut self) -> BufferInitDescriptor;

    /// Definition of the index buffer.
    fn index_buffer_descriptor(&mut self) -> BufferInitDescriptor;

    /// Definition of the primitive type.
    ///
    /// Defaults to [`wgpu::PrimitiveTopology`].
    fn topology() -> PrimitiveTopology {
        PrimitiveTopology::TriangleList
    }

    /// Whether the mesh needs to be updated on the GPU.
    fn is_dirty(&self) -> bool;

    /// All instances of this type to render.
    fn instances(&self) -> &[Vec2<f64>];

    /// Range of indices to draw.
    fn range(&self) -> Range<u32>;
}

/// Simple render state holding buffers and instances required for rendering somethging.
pub struct RenderState<R: Render> {
    /// Pipeline of the rendering itself.
    render_pipeline: RenderPipeline,
    /// GPU buffer reference to the vertices.
    vertex_buffer: Buffer,
    /// GPU buffer reference to the indices.
    index_buffer: Buffer,
    /// GPU buffer reference to the instances.
    instance_buffer: Buffer,
    /// Hold the type so we can't have mismatching calls.
    _phantom: PhantomData<R>,
}

impl<R: Render> RenderState<R> {
    /// Create the state by calling the [`Render`] trait implementations on the type.
    pub fn new(device: &Device) -> Self
    where
        R: Render,
    {
        // Create a new render pipeline first
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Component Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Load the shaders from disk
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("./shaders/texture.wgsl"))),
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Component Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                buffers: &[TexturedVertex::descriptor(), Instances::descriptor()],
                module: &shader,
                entry_point: "vs_main",
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba8UnormSrgb,
                    blend: Some(BlendState {
                        color: BlendComponent::REPLACE,
                        alpha: BlendComponent::REPLACE,
                    }),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: R::topology(),
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                // Irrelevant since we disable culling
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                // How many samples the pipeline will use
                count: 1,
                // Use all masks
                mask: !0,
                // Disable anti-aliasing
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create the vertex buffer
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &[],
            usage: BufferUsages::VERTEX,
        });

        // Create the initial instance buffer
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: &[],
            usage: BufferUsages::VERTEX,
        });

        // Create the index buffer
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: &[],
            usage: BufferUsages::INDEX,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            instance_buffer,
            index_buffer,
            _phantom: PhantomData,
        }
    }

    /// Render all instances from the type with the specified texture.
    pub fn render<T>(&mut self, target: &R, texture: &T, encoder: &mut CommandEncoder)
    where
        T: Texture,
    {
        // Construct the instances to upload
        // PERF: don't clone every frame
        let instances = Instances::from_slice(target.instances());

        // Get the texture state from the `Texture` trait
        let texture_state = texture.state().expect("Texture not initialized yet");

        // Start the render pass
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Component Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &texture_state.texture_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::GREEN),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Set our pipeline
        render_pass.set_pipeline(&self.render_pipeline);

        // Bind the texture
        render_pass.set_bind_group(0, &texture_state.bind_group, &[]);

        // Set the target vertices
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // Set the instances
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        // Set the target indices
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

        // Draw the instances
        render_pass.draw_indexed(target.range(), 0, instances.range());
    }
}
