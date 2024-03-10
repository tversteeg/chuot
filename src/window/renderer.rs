use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    AddressMode, Backends, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferAddress, BufferBindingType,
    BufferSize, BufferUsages, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features,
    FilterMode, FragmentState, Instance, InstanceDescriptor, Limits, LoadOp, MultisampleState,
    PipelineLayoutDescriptor,
    PowerPreference::HighPerformance,
    PrimitiveState, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptionsBase, SamplerBindingType, SamplerDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, Surface, SurfaceConfiguration,
    TextureSampleType, TextureViewDescriptor, TextureViewDimension, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode, WindowHandle,
};

/// Pass multiple fields to the game state.
struct State<'window, G> {
    /// User passed game state.
    game_state: G,
    /// Winit input helper state.
    input: WinitInputHelper,
    /// GPU surface.
    surface: Surface<'window>,
    /// GPU device.
    device: Device,
    /// GPU surface configuration.
    config: SurfaceConfiguration,
    /// GPU render pipeline.
    render_pipeline: RenderPipeline,
    /// GPU queue.
    queue: Queue,
}

impl<'window, G> State<'window, G> {
    /// Setup the state including the GPU part.
    async fn new<W>(buffer_size: Extent2<u32>, window: W, game_state: G) -> Result<Self>
    where
        W: WindowHandle + 'window,
    {
        // Setup the winit input helper state
        let input = WinitInputHelper::new();

        // Get a handle to our GPU
        let instance = Instance::default();

        // Create a GPU surface on the window
        let surface = instance
            .create_surface(window)
            .into_diagnostic()
            .wrap_err("Error creating surface on window")?;

        // Request an adapter
        let adapter = instance
            .request_adapter(&RequestAdapterOptionsBase {
                // Ensure the strongest GPU is used
                power_preference: HighPerformance,
                force_fallback_adapter: false,
                // Request an adaptar which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or_else(|| miette::miette!("Error getting GPU adapter for window"))?;

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    // Use the texture resolution limits from the adapter
                    required_limits: Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .into_diagnostic()
            .wrap_err("Error getting logical GPU device for surface")?;

        // Load the shaders from disk
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("./shaders/window.wgsl"))),
        });

        // Create a texture sampler with nearest neighbor
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
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
            [-1.0, -1.0],
            [3.0, -1.0],
            [-1.0, 3.0],
        ];
        let vertex_data_slice = bytemuck::cast_slice(&vertex_data);
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: vertex_data_slice,
            usage: wgpu::BufferUsages::VERTEX,
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

        // Create uniform buffer
        let matrix = ScalingMatrix::new(buffer_size.as_(), buffer_size.as_());
        let transform_bytes = matrix.as_bytes();
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: transform_bytes,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create bind group
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(transform_bytes.len() as u64),
                    },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        let config = surface
            .get_default_config(&adapter, buffer_size.w, buffer_size.h)
            .ok_or_else(|| miette::miette!("Error getting window surface configuration"))?;
        surface.configure(&device, &config);

        Ok(Self {
            game_state,
            input,
            surface,
            device,
            config,
            render_pipeline,
            queue,
        })
    }
}

/// Matrix for scaling the output texture.
///
/// Source: https://github.com/parasyte/pixels/blob/main/src/renderers.rs
#[derive(Debug)]
struct ScalingMatrix {
    /// 4x4 transformation matrix.
    transform: [f32; 16],
    /// Area to draw inside.
    clip_rect: Rect<u32, u32>,
}

impl ScalingMatrix {
    // texture_size is the dimensions of the drawing texture
    // screen_size is the dimensions of the surface being drawn to
    fn new(texture_size: Extent2<f32>, screen_size: Extent2<f32>) -> Self {
        let width_ratio = (screen_size.w / texture_size.w).max(1.0);
        let height_ratio = (screen_size.h / texture_size.h).max(1.0);

        // Get smallest scale size
        let scale = width_ratio.clamp(1.0, height_ratio).floor();

        let scaled_size = texture_size * scale;

        // Create a transformation matrix
        let sw = scaled_size.w / screen_size.w;
        let sh = scaled_size.h / screen_size.h;
        let tx = (screen_size.w / 2.0).fract() / screen_size.w;
        let ty = (screen_size.h / 2.0).fract() / screen_size.h;
        #[rustfmt::skip]
        let transform: [f32; 16] = [
            sw,  0.0, 0.0, 0.0,
            0.0, sh,  0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            tx,  ty,  0.0, 1.0,
        ];

        // Create a clipping rectangle
        let clip_rect = {
            let scaled_size_w = scaled_size.w.min(screen_size.w);
            let scaled_size_h = scaled_size.h.min(screen_size.h);
            let x = (screen_size.w - scaled_size_w) / 2.0;
            let y = (screen_size.h - scaled_size_h) / 2.0;

            Rect::new(x, y, scaled_size_w, scaled_size_h).as_()
        };

        Self {
            transform,
            clip_rect,
        }
    }

    /// Represent the transform as bytes.
    fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.transform)
    }
}
