//! Types and helpers for drawing on the GPU.

pub mod data;
pub mod render;
pub mod texture;

use miette::{IntoDiagnostic, Result, WrapErr};
use vek::Extent2;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages,
    CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance, Limits,
    PowerPreference, Queue, RequestAdapterOptionsBase, SamplerBindingType, ShaderStages, Surface,
    SurfaceConfiguration, SurfaceTexture, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension, WindowHandle,
};

use crate::{sprite::Sprite, window::RenderFn};

use self::{render::RenderState, texture::Texture};

/// Main render state holding the GPU information.
pub(crate) struct MainRenderState<'window> {
    /// GPU surface.
    pub(crate) surface: Surface<'window>,
    /// GPU device.
    pub(crate) device: Device,
    /// GPU surface configuration.
    pub(crate) config: SurfaceConfiguration,
    /// Main GPU render pipeline.
    //render_pipeline: RenderPipeline,
    /// GPU queue.
    pub(crate) queue: Queue,
    /// Bind group layout for rendering diffuse textures.
    pub(crate) diffuse_texture_bind_group_layout: BindGroupLayout,
    /// Buffer for passing the screen size to the shaders.
    pub(crate) screen_size_buffer: Buffer,
    /// Bind group for passing the screen size to the shaders.
    pub(crate) screen_size_bind_group: BindGroup,
    /// Sprite component specific render pipelines.
    pub(crate) sprite_render_state: RenderState<Sprite>,
}

impl<'window> MainRenderState<'window> {
    /// Create a GPU surface on the window.
    pub(crate) async fn new<W>(buffer_size: Extent2<u32>, window: W) -> Result<Self>
    where
        W: WindowHandle + 'window,
    {
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
                power_preference: PowerPreference::HighPerformance,
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
                    // WebGL doesn't support all features
                    required_limits: if cfg!(target_arch = "wasm32") {
                        Limits::downlevel_webgl2_defaults()
                    } else {
                        Limits::default()
                    },
                },
                None,
            )
            .await
            .into_diagnostic()
            .wrap_err("Error getting logical GPU device for surface")?;

        /*
        // Load the shaders from disk
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Main Window Shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../graphics/shaders/window.wgsl"
            ))),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Main Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        */

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        // Find an sRGB surface texture
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(swapchain_capabilities.formats[0]);

        // Configure the render surface
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: buffer_size.w,
            height: buffer_size.h,
            present_mode: swapchain_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        /*
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Main Pipeline"),
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
        */

        // Create the bind group layout for all textures
        let diffuse_texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Diffuse Texture Bind Group Layout"),
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
                ],
            });

        // Upload a buffer for the screen size
        let initial_screen_size = [buffer_size.w as f32, buffer_size.h as f32];
        let screen_size_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Screen Size Buffer"),
            contents: bytemuck::cast_slice(&initial_screen_size),
            // Allow us to update this buffer
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create the bind group layout for passing the screen size
        let screen_size_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Screen Size Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Also already create the bind group, since it will be used without changing the size
        let screen_size_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Screen Size Bind Group"),
            layout: &screen_size_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(screen_size_buffer.as_entire_buffer_binding()),
            }],
        });

        // Create a custom pipeline for each component
        let sprite_render_state = RenderState::new(
            &device,
            &diffuse_texture_bind_group_layout,
            &screen_size_bind_group_layout,
        );

        Ok(Self {
            surface,
            device,
            config,
            //render_pipeline,
            sprite_render_state,
            queue,
            diffuse_texture_bind_group_layout,
            screen_size_buffer,
            screen_size_bind_group,
        })
    }

    /// Render the frame and call the user `render` function.
    pub(crate) fn render(&mut self, sprites: Vec<Sprite>) {
        // Upload all un-uploaded textures
        texture::upload(
            &self.device,
            &self.queue,
            &self.diffuse_texture_bind_group_layout,
        );

        // Get the main render texture
        let frame = self
            .surface
            .get_current_texture()
            .expect("Error acquiring next swap chain texture");
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        // Render each sprite
        for mut sprite in sprites {
            // Render the sprite
            self.sprite_render_state.render(
                &mut sprite,
                &mut encoder,
                &view,
                &self.queue,
                &self.device,
                &self.screen_size_bind_group,
            );
        }

        // Draw to the texture
        self.queue.submit(Some(encoder.finish()));

        // Show the texture in the window
        frame.present();
    }

    // Resize the surface.
    pub(crate) fn resize(&mut self, new_size: Extent2<u32>) {
        self.config.width = new_size.w.max(1);
        self.config.height = new_size.h.max(1);
        self.surface.configure(&self.device, &self.config);

        // Update the screen size buffer to applied in the next render call
        let screen_size = [new_size.w as f32, new_size.h as f32];
        self.queue.write_buffer(
            &self.screen_size_buffer,
            0,
            bytemuck::cast_slice(&screen_size),
        );
    }
}
