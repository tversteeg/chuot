//! Pixel shader renderers.

use pixels::{
    wgpu::{
        util::{BufferInitDescriptor, DeviceExt},
        AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
        BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
        BlendComponent, BlendState, Buffer, BufferAddress, BufferUsages, Color, ColorTargetState,
        ColorWrites, CommandEncoder, Device, Extent3d, FilterMode, FragmentState, LoadOp,
        MultisampleState, Operations, PipelineLayoutDescriptor, PrimitiveState,
        RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
        Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, TextureDescriptor,
        TextureDimension, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
        TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
        VertexStepMode,
    },
    Pixels, TextureError,
};
use vek::{Extent2, Rect};

/// Convert RGBA pixels to BGRA.
pub(crate) struct RgbaToBgraRenderer {
    texture_view: TextureView,
    sampler: Sampler,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
}

impl RgbaToBgraRenderer {
    /// Setup the shader.
    pub(crate) fn new(pixels: &Pixels, size: Extent2<u32>) -> Result<Self, TextureError> {
        let device = pixels.device();
        let shader = pixels::wgpu::include_wgsl!("shaders/rgba_to_bgra.wgsl");
        let module = device.create_shader_module(shader);

        // Create a texture view that will be used as input
        // This will be used as the render target for the default scaling renderer
        let texture_view = create_texture_view(pixels, size.w, size.h)?;

        // Create a texture sampler with nearest neighbor
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("NoiseRenderer sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        // Create vertex buffer; array-of-array of position and texture coordinates
        let vertex_data: [[f32; 2]; 3] = [
            // One full-screen triangle
            // See: https://github.com/parasyte/pixels/issues/180
            [-1.0, -1.0],
            [3.0, -1.0],
            [-1.0, 3.0],
        ];
        let vertex_data_slice = bytemuck::cast_slice(&vertex_data);
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("NoiseRenderer vertex buffer"),
            contents: vertex_data_slice,
            usage: BufferUsages::VERTEX,
        });
        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: (vertex_data_slice.len() / vertex_data.len()) as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        };

        // Create bind group
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let bind_group = create_bind_group(device, &bind_group_layout, &texture_view, &sampler);

        // Create pipeline
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("NoiseRenderer pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("NoiseRenderer pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &module,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: pixels.render_texture_format(),
                    blend: Some(BlendState {
                        color: BlendComponent::REPLACE,
                        alpha: BlendComponent::REPLACE,
                    }),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Ok(Self {
            texture_view,
            sampler,
            bind_group_layout,
            bind_group,
            render_pipeline,
            vertex_buffer,
        })
    }

    /// Resize the texture.
    pub(crate) fn resize(
        &mut self,
        pixels: &Pixels,
        size: Extent2<u32>,
    ) -> Result<(), TextureError> {
        self.texture_view = create_texture_view(pixels, size.w, size.h)?;
        self.bind_group = create_bind_group(
            pixels.device(),
            &self.bind_group_layout,
            &self.texture_view,
            &self.sampler,
        );

        Ok(())
    }

    /// Draw the shader.
    pub(crate) fn render(
        &self,
        encoder: &mut CommandEncoder,
        render_target: &TextureView,
        clip_rect: Rect<u32, u32>,
    ) {
        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("NoiseRenderer render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: render_target,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_scissor_rect(clip_rect.x, clip_rect.y, clip_rect.w, clip_rect.h);
        rpass.draw(0..3, 0..1);
    }

    /// Get the texture view used.
    pub(crate) fn texture_view(&self) -> &TextureView {
        &self.texture_view
    }
}

fn create_texture_view(
    pixels: &pixels::Pixels,
    width: u32,
    height: u32,
) -> Result<TextureView, TextureError> {
    let device = pixels.device();
    pixels::check_texture_size(device, width, height)?;
    let texture_descriptor = TextureDescriptor {
        label: None,
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: pixels.render_texture_format(),
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };

    Ok(device
        .create_texture(&texture_descriptor)
        .create_view(&TextureViewDescriptor::default()))
}

fn create_bind_group(
    device: &Device,
    bind_group_layout: &BindGroupLayout,
    texture_view: &TextureView,
    sampler: &Sampler,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(sampler),
            },
        ],
    })
}
