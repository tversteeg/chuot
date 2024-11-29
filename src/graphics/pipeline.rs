//! Abstraction for rendering instances with a shader.

use std::borrow::Cow;

use glam::Affine2;
use wgpu::util::DeviceExt as _;

use super::{
    Instances, PREFERRED_TEXTURE_FORMAT, ScreenInfo, TextureRef, UniformState, atlas::Atlas,
    data::TexturedVertex,
};

/// The flow for rendering instances with a shader.
pub(crate) struct Pipeline {
    /// All instances to render.
    instances: Instances,
    /// Pipeline of the rendering itself.
    render: wgpu::RenderPipeline,
    /// GPU buffer reference to all instances of the texture squares.
    instance_buffer: wgpu::Buffer,
}

impl Pipeline {
    /// Create and upload a pipeline from a shader.
    pub(crate) fn new(
        shader_source: &str,
        fragment_shader_entry_point: Option<&str>,
        device: &wgpu::Device,
        screen_info: &UniformState<ScreenInfo>,
        atlas: &Atlas,
    ) -> Self {
        // Create a new render pipeline first
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Component Render Pipeline Layout"),
                bind_group_layouts: &[
                    &atlas.bind_group_layout,
                    &atlas.rects.bind_group_layout,
                    &screen_info.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        // Upload the shader to the GPU
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Diffuse Texture Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_source)),
        });

        // Create the pipeline for rendering textures
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                buffers: &[TexturedVertex::descriptor(), Instances::descriptor()],
                module: &shader,
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: fragment_shader_entry_point,
                targets: &[Some(wgpu::ColorTargetState {
                    format: PREFERRED_TEXTURE_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
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
            cache: None,
        });

        // Create the initial empty instance buffer, will be resized by the render call
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // Create the instances list
        let instances = Instances::default();

        Self {
            instances,
            render: render_pipeline,
            instance_buffer,
        }
    }

    /// Render the instances if applicable.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertex_buffer: &wgpu::Buffer,
        index_buffer: &wgpu::Buffer,
        render_pass: &mut wgpu::RenderPass<'_>,
        screen_info: &UniformState<ScreenInfo>,
        atlas: &Atlas,
    ) {
        if self.instances.is_empty() {
            // Nothing to render when there's no instances
            return;
        }

        // Get the instances
        let (instances_bytes, instances_len) = {
            // Construct the bytes of the instances to upload
            (self.instances.bytes(), self.instances.len())
        };

        // Resize the buffer if needed
        let instance_buffer_already_pushed = if instances_bytes.len() as u64
            > self.instance_buffer.size()
        {
            // We have more instances than the buffer size, recreate the buffer
            self.instance_buffer.destroy();
            self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: instances_bytes,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

            true
        } else {
            false
        };

        // Set our pipeline
        render_pass.set_pipeline(&self.render);

        // Bind the atlas texture
        render_pass.set_bind_group(0, &atlas.bind_group, &[]);
        // Bind the atlas texture info
        render_pass.set_bind_group(1, &atlas.rects.bind_group, &[]);

        // Bind the screen size
        render_pass.set_bind_group(2, &screen_info.bind_group, &[]);

        // Set the target indices
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        // Set the target vertices
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        // Set the instances
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        // Draw the instances
        render_pass.draw_indexed(0..6, 0, 0..instances_len as u32);

        // Upload the instance buffer
        if !instance_buffer_already_pushed {
            queue.write_buffer(&self.instance_buffer, 0, instances_bytes);
        }

        // Clear the instances to write a new frame
        self.instances.clear();
    }

    /// Push an item to the the instance array.
    pub(crate) fn push_instance(
        &mut self,
        transformation: Affine2,
        sub_rectangle: (f32, f32, f32, f32),
        texture_ref: TextureRef,
    ) {
        self.instances
            .push(transformation, sub_rectangle, texture_ref);
    }

    /// Extend the instances of the default shader or a custom shader.
    pub(crate) fn extend_instances(
        &mut self,
        items: impl Iterator<Item = (Affine2, (f32, f32, f32, f32), TextureRef)>,
    ) {
        self.instances.extend(items);
    }
}
