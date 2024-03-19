//! Expose render functionality on different types through traits.

use std::{borrow::Cow, marker::PhantomData};

use hashbrown::HashMap;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupLayout, BlendState, Buffer, BufferUsages, Color, ColorTargetState,
    ColorWrites, CommandEncoder, Device, FragmentState, FrontFace, IndexFormat, LoadOp,
    MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, StoreOp, TextureFormat, TextureView, VertexState,
};

use super::{
    data::TexturedVertex,
    instance::Instances,
    texture::{TextureInfo, TextureRef, UploadedTextureState},
    uniform::UniformState,
    Render,
};

/// Simple render state holding buffers and instances required for rendering somethging.
pub(crate) struct RenderState<R: Render> {
    /// Pipeline of the rendering itself.
    render_pipeline: RenderPipeline,
    /// GPU buffer reference to the vertices.
    vertex_buffer: Buffer,
    /// GPU buffer reference to the indices.
    index_buffer: Buffer,
    /// GPU buffer reference to the instances.
    instance_buffer: Buffer,
    /// Uniform to hold the texture size.
    texture_size_uniform: UniformState<TextureInfo>,
    /// Hold the type so we can't have mismatching calls.
    _phantom: PhantomData<R>,
}

impl<R: Render> RenderState<R> {
    /// Create the state by calling the [`Render`] trait implementations on the type.
    pub(crate) fn new(
        device: &Device,
        diffuse_texture_bind_group_layout: &BindGroupLayout,
        screen_info_bind_group_layout: &BindGroupLayout,
    ) -> Self
    where
        R: Render,
    {
        log::debug!("Creating custom rendering component");

        // Create the uniform for holding the texture size, will be updated by each component itself
        let texture_size_uniform = UniformState::new(device, &TextureInfo::default());

        // Create a new render pipeline first
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Component Render Pipeline Layout"),
            bind_group_layouts: &[
                diffuse_texture_bind_group_layout,
                screen_info_bind_group_layout,
                &texture_size_uniform.bind_group_layout,
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
                    blend: Some(BlendState::ALPHA_BLENDING),
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

        // Create the initial empty instance buffer, will be resized by the render call
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: &[],
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
            texture_size_uniform,
            _phantom: PhantomData,
        }
    }

    /// Render all instances from the type with the specified texture.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render(
        &mut self,
        target: &mut R,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        queue: &Queue,
        device: &Device,
        screen_size_bind_group: &BindGroup,
        uploaded_textures: &HashMap<TextureRef, UploadedTextureState>,
    ) {
        // Target will be rendered
        target.pre_render();

        // Get the instances
        let (instances_bytes, instances_len) = {
            // We could simplify the scope here by implementing `Render::instances()` but that means implementing another method for each trait implementor

            let instances = target.instances_mut();
            if instances.is_empty() {
                // Nothing to render when there's no instances
                return;
            }

            // Construct the bytes of the instances to upload
            (instances.bytes(), instances.len())
        };

        // Upload the instance buffer
        if instances_bytes.len() as u64 <= self.instance_buffer.size() {
            // We still fit in the buffer, we don't have to resize it
            queue.write_buffer(&self.instance_buffer, 0, instances_bytes);
        } else {
            // We have more instances than the buffer size, recreate the buffer

            log::debug!(
                "Previous instance buffer is too small, rescaling to {} items",
                instances_len
            );

            self.instance_buffer.destroy();
            self.instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: instances_bytes,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
        }

        // Upload the buffers if dirty
        if target.is_dirty() {
            log::debug!(
                "Custom rendering component is dirty, re-uploading vertex and index buffers"
            );

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

            // Set the texture size
            if let Some(texture) = target.texture() {
                // Get the texture so we can update the size
                let uploaded_texture = uploaded_textures
                    .get(texture)
                    .expect("Error getting uploaded texture");

                let size = uploaded_texture.texture.size();

                log::debug!(
                    "Setting texture size uniform to ({}x{})",
                    size.width,
                    size.height
                );

                // Update the texture size uniform
                // The last values are used for padding to align to 16 bytes
                self.texture_size_uniform.update(
                    &TextureInfo {
                        size: [size.width as f32, size.height as f32],
                        ..Default::default()
                    },
                    queue,
                );
            }

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
            let uploaded_texture = uploaded_textures
                .get(texture)
                .expect("Error getting uploaded texture");

            // Bind the texture
            render_pass.set_bind_group(0, &uploaded_texture.bind_group, &[]);
        }

        // Bind the screen size
        render_pass.set_bind_group(1, screen_size_bind_group, &[]);

        // Bind the texture size
        render_pass.set_bind_group(2, &self.texture_size_uniform.bind_group, &[]);

        // Set the target vertices
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // Set the instances
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        // Set the target indices
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

        // Draw the instances
        render_pass.draw_indexed(target.range(), 0, 0..instances_len as u32);
    }
}
