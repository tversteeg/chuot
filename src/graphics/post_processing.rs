//! State for post-processing shaders.

use std::borrow::Cow;

use bytemuck::NoUninit;
use glamour::{Rect, Size2};

use super::{data::ScreenInfo, gpu::Frame, state::PREFERRED_TEXTURE_FORMAT, uniform::UniformState};

/// State data collection for post processing stages.
pub(crate) struct PostProcessingState {
    /// Resulting texture that the post processing pass will be drawn to.
    pub(crate) texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl PostProcessingState {
    /// Upload a new post processing state effect.
    pub(crate) fn new<T: NoUninit>(
        buffer_size: Size2,
        device: &wgpu::Device,
        uniform: &UniformState<T>,
        shader: &'static str,
    ) -> Self {
        // Create the internal texture for rendering the first pass to
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Post Processing Texture"),
            size: wgpu::Extent3d {
                width: buffer_size.width as u32,
                height: buffer_size.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: PREFERRED_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create the bind group layout for the screen after it has been upscaled
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post Processing Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create the sampler we use to sample from the input texture view
        let input_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Post Processing Input Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create the bind group binding the layout with the texture view
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Post Processing Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&input_sampler),
                },
            ],
        });

        // Create a new render pipeline first
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Post Processing Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout, &uniform.bind_group_layout],
                push_constant_ranges: &[],
            });

        // Load the shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Post Processing Texture Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader)),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Post Processing Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                buffers: &[],
                module: &shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: PREFERRED_TEXTURE_FORMAT,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            texture_view,
            bind_group,
            render_pipeline,
        }
    }

    /// Render the post processing shader.
    ///
    /// Takes the surface texture from the frame as the texture view if `view` is `None`.
    pub(crate) fn render(
        &self,
        frame: &mut Frame,
        view: Option<&wgpu::TextureView>,
        screen_info: &UniformState<ScreenInfo>,
        letterbox: Option<Rect>,
        background_color: wgpu::Color,
    ) {
        // Start the render pass
        let mut upscaled_render_pass =
            frame
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Post Processing Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: view.unwrap_or(&frame.surface_view),
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

        upscaled_render_pass.set_pipeline(&self.render_pipeline);

        // Only draw in the calculated letterbox to get nice integer scaling
        if let Some(Rect { origin, size }) = letterbox {
            upscaled_render_pass.set_viewport(
                origin.x,
                origin.y,
                size.width,
                size.height,
                0.0,
                1.0,
            );
        }

        // Bind the source texture
        upscaled_render_pass.set_bind_group(0, &self.bind_group, &[]);

        // Bind the screen info uniform
        upscaled_render_pass.set_bind_group(1, &screen_info.bind_group, &[]);

        // Draw the 'buffer' defined in the vertex shader
        upscaled_render_pass.draw(0..3, 0..1);
    }
}
