//! State for post-processing shaders.

use std::borrow::Cow;

use bytemuck::NoUninit;
use vek::{Extent2, Rect};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent, BlendState, Color,
    ColorTargetState, ColorWrites, CommandEncoder, Device, Extent3d, FilterMode, FragmentState,
    LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PrimitiveState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    Sampler, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDescriptor, TextureViewDimension, VertexState,
};

use super::{data::ScreenInfo, uniform::UniformState};

/// State data collection for post processing stages.
pub(crate) struct PostProcessingState {
    /// Resulting texture that the post processing pass will be drawn to.
    pub(crate) texture_view: TextureView,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
}

impl PostProcessingState {
    /// Upload a new post processing state effect.
    pub(crate) fn new<T: NoUninit>(
        buffer_size: Extent2<u32>,
        device: &Device,
        uniform: &UniformState<T>,
        shader: &'static str,
    ) -> Self {
        // Create the internal texture for rendering the first pass to
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Post Processing Texture"),
            size: Extent3d {
                width: buffer_size.w,
                height: buffer_size.h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        // Create the bind group layout for the screen after it has been upscaled
        let bind_group_layout =
            super::texture::create_bind_group_layout(device, "Post Processing Bind Group Layout");

        // Create the sampler we use to sample from the input texture view
        let input_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Post Processing Input Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        // Create the bind group binding the layout with the texture view
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Post Processing Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&input_sampler),
                },
            ],
        });

        // Create a new render pipeline first
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Post Processing Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &uniform.bind_group_layout],
            push_constant_ranges: &[],
        });

        // Load the shaders
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Post Processing Texture Shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(shader)),
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Post Processing Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                buffers: &[],
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
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        Self {
            bind_group,
            render_pipeline,
            texture_view,
        }
    }

    /// Render the post processing shader.
    pub(crate) fn render(
        &self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        screen_info: &UniformState<ScreenInfo>,
        letterbox: Option<Rect<f32, f32>>,
    ) {
        // Start the render pass
        let mut upscaled_render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Post Processing Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLUE),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        upscaled_render_pass.set_pipeline(&self.render_pipeline);

        // Only draw in the calculated letterbox to get nice integer scaling
        if let Some(Rect { x, y, w, h }) = letterbox {
            upscaled_render_pass.set_viewport(x, y, w, h, 0.0, 1.0);
        }

        // Bind the source texture
        upscaled_render_pass.set_bind_group(0, &self.bind_group, &[]);

        // Bind the screen info uniform
        upscaled_render_pass.set_bind_group(1, &screen_info.bind_group, &[]);

        // Draw the 'buffer' defined in the vertex shader
        upscaled_render_pass.draw(0..3, 0..1);
    }
}
