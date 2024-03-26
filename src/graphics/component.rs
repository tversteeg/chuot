//! Expose render functionality on different types through traits.

use std::borrow::Cow;

use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::{config::RotationAlgorithm, graphics::state::PREFERRED_TEXTURE_FORMAT, sprite::Sprite};

use super::{atlas::Atlas, data::TexturedVertex, instance::Instances};

/// Simple render state holding buffers and instances required for rendering somethging.
pub(crate) struct SpriteRenderState {
    /// Pipeline of the rendering itself.
    render_pipeline: wgpu::RenderPipeline,
    /// GPU buffer reference to the vertices.
    vertex_buffer: wgpu::Buffer,
    /// GPU buffer reference to the indices.
    index_buffer: wgpu::Buffer,
    /// GPU buffer reference to the instances.
    instance_buffer: wgpu::Buffer,
    /// Amount of indices to render.
    indices: u32,
}

impl SpriteRenderState {
    /// Create the state by calling the [`Render`] trait implementations on the type.
    pub(crate) fn new(
        device: &wgpu::Device,
        screen_info_bind_group_layout: &wgpu::BindGroupLayout,
        texture_atlas: &Atlas,
        rotation_algorithm: RotationAlgorithm,
    ) -> Self {
        log::debug!("Creating custom rendering component");

        // Create a new render pipeline first
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Component Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_atlas.bind_group_layout,
                    &texture_atlas.rects.bind_group_layout,
                    screen_info_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Diffuse Texture Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("./shaders/texture.wgsl"))),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Component Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                buffers: &[TexturedVertex::descriptor(), Instances::descriptor()],
                module: &shader,
                entry_point: "vs_main",
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: match rotation_algorithm {
                    RotationAlgorithm::Scale3x => "fs_main_scale3x",
                    RotationAlgorithm::Scale2x => "fs_main_scale2x",
                    RotationAlgorithm::Diag2x => "fs_main_diag2x",
                    RotationAlgorithm::NearestNeighbor => "fs_main_nearest_neighbor",
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: PREFERRED_TEXTURE_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                // Irrelevant since we disable culling
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
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
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE,
        });

        // Create the vertex buffer
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&Sprite::vertices()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // Create the index buffer
        let indices = Sprite::indices();
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        let indices = indices.len() as u32;

        Self {
            indices,
            render_pipeline,
            vertex_buffer,
            instance_buffer,
            index_buffer,
        }
    }

    /// Render all instances from all types with the specified texture.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render(
        &mut self,
        instances: &Instances,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        screen_size_bind_group: &wgpu::BindGroup,
        texture_atlas: &Atlas,
        background_color: wgpu::Color,
    ) {
        if instances.is_empty() {
            // Nothing to render when there's no instances
            return;
        }

        // Get the instances
        let (instances_bytes, instances_len) = {
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
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
            });
        }

        // Start the render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Component Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(background_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Set our pipeline
        render_pass.set_pipeline(&self.render_pipeline);

        // Bind the atlas texture
        render_pass.set_bind_group(0, &texture_atlas.bind_group, &[]);
        // Bind the atlas texture info
        render_pass.set_bind_group(1, &texture_atlas.rects.bind_group, &[]);

        // Bind the screen size
        render_pass.set_bind_group(2, screen_size_bind_group, &[]);

        // Set the target vertices
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // Set the instances
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        // Set the target indices
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // Draw the instances
        render_pass.draw_indexed(0..self.indices, 0, 0..instances_len as u32);
    }
}
