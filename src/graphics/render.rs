//! Expose render functionality on different types through traits.

use std::{borrow::Cow, marker::PhantomData, ops::Range};

use vek::Vec2;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupLayout, BlendComponent, BlendState, Buffer, BufferUsages, Color,
    ColorTargetState, ColorWrites, CommandEncoder, Device, FragmentState, FrontFace, IndexFormat,
    LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
    StoreOp, TextureFormat, TextureView, TextureViewDescriptor, VertexState,
};

use super::{
    data::{Instances, TexturedVertex},
    texture::{Texture, TextureRef},
    MainRenderState,
};

/// Allow something to be rendered on the GPU.
pub trait Render {
    /// Definition of the primitive type.
    ///
    /// Defaults to [`wgpu::PrimitiveTopology`].
    fn topology() -> PrimitiveTopology {
        PrimitiveTopology::TriangleList
    }

    /// Whether the mesh needs to be updated on the GPU.
    ///
    /// This is not influenced by instancing.
    fn is_dirty(&self) -> bool;

    /// Tell the object everything is up to date.
    fn mark_clean(&mut self);

    /// All instances of this type to render.
    fn instances(&self) -> &[Vec2<f64>];

    /// All vertices of this type to render.
    fn vertices(&self) -> &[TexturedVertex];

    /// All indices of this type to render.
    fn indices(&self) -> &[u16];

    /// Range of indices to draw.
    fn range(&self) -> Range<u32>;

    /// Texture reference to bind and render.
    ///
    /// If `None` no texture binding will be applied.
    fn texture(&self) -> Option<TextureRef> {
        None
    }

    /// Called just before rendering the objects.
    ///
    /// Can be overwritten to handle some simple logic.
    fn pre_render(&mut self) {}

    /// Called just after rendering the objects.
    ///
    /// Can be overwritten to handle some simple logic.
    fn post_render(&mut self) {}

    /// Maximum amount of instances that can be drawn for this type.
    ///
    /// Defaults to `1024`.
    fn max_instances() -> usize {
        1024
    }
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
    pub fn new(
        device: &Device,
        diffuse_texture_bind_group_layout: &BindGroupLayout,
        screen_size_bind_group_layout: &BindGroupLayout,
    ) -> Self
    where
        R: Render,
    {
        // Create a new render pipeline first
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Component Render Pipeline Layout"),
            bind_group_layouts: &[
                diffuse_texture_bind_group_layout,
                screen_size_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        // Load the shaders from disk
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Diffuse Texture Shader"),
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

        // Create an empty buffer for all possible instances
        let empty_instance_bytes = R::max_instances() * std::mem::size_of::<[f32; 2]>();
        let empty_instance_buffer = vec![0; empty_instance_bytes];

        // Create the initial instance buffer with the maximum drawing size
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: &empty_instance_buffer,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        // Create the vertex buffer
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &[],
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        // Create the index buffer
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: &[],
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            instance_buffer,
            index_buffer,
            _phantom: PhantomData,
        }
    }
}

impl<R: Render> RenderState<R> {
    /// Render all instances from the type with the specified texture.
    pub fn render(
        &mut self,
        target: &mut R,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        queue: &Queue,
        device: &Device,
        screen_size_bind_group: &BindGroup,
    ) {
        // Target will be rendered
        target.pre_render();

        // Construct the instances to upload
        // PERF: don't clone every frame
        let instances = Instances::from_slice(target.instances());

        // Upload the instance buffer
        queue.write_buffer(&self.instance_buffer, 0, instances.bytes());

        // Upload the buffers if dirty
        if target.is_dirty() {
            // Recreate the vertex buffer
            self.vertex_buffer.destroy();
            self.vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(target.vertices()),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });

            // Re upload the indices
            self.index_buffer.destroy();
            self.index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(target.indices()),
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            });

            // Target is not dirty anymore
            target.mark_clean();
        }

        // Allow components to clean up, doesn't matter even though it's technically not rendered yet
        target.post_render();

        // Start the render pass
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Component Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
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

        // Get the texture state from the reference
        if let Some(texture) = target.texture() {
            // Bind the texture
            super::texture::set_bind_group(&texture, &mut render_pass, 0);
        }

        // Bind the screen size
        render_pass.set_bind_group(1, screen_size_bind_group, &[]);

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
