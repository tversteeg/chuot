//! Main rendering state.

use std::{
    borrow::Cow,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bytemuck::NoUninit;
use hashbrown::HashMap;
use miette::{IntoDiagnostic, Result, WrapErr};
use vek::{Extent2, Rect};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent, BlendState, Buffer,
    BufferBindingType, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Features, FragmentState,
    Instance, Limits, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
    PowerPreference, PrimitiveState, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptionsBase, SamplerBindingType,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, Surface, SurfaceConfiguration,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDescriptor, TextureViewDimension, VertexState, WindowHandle,
};

use crate::{sprite::Sprite, RenderContext};

use super::{
    component::RenderState,
    data::ScreenInfo,
    texture::{TextureRef, UploadedTextureState, PENDING_TEXTURES},
};

/// Scale at which the pixels are drawn for rotations.
const PIXEL_UPSCALE: u32 = 1;

/// Main render state holding the GPU information.
pub(crate) struct MainRenderState<'window> {
    /// GPU surface.
    surface: Surface<'window>,
    /// GPU device.
    device: Device,
    /// GPU surface configuration.
    config: SurfaceConfiguration,
    /// GPU queue.
    queue: Queue,
    /// Bind group layout for rendering diffuse textures.
    diffuse_texture_bind_group_layout: BindGroupLayout,
    /// Intermediate upscaled texture.
    upscaled_pass: PostProcessingState,
    /// Uniform screen info (size and scale) to the shaders.
    screen_info: UniformState<ScreenInfo>,
    /// Sprite component specific render pipelines.
    sprite_render_state: RenderState<Sprite>,
    /// Uploaded textures.
    uploaded_textures: HashMap<TextureRef, UploadedTextureState>,
    /// Render context passed to each user facing render frame.
    ctx: RenderContext,
    /// Size of the final buffer to draw.
    ///
    /// Will be scaled with integer scaling and letterboxing to fit the screen.
    buffer_size: Extent2<u32>,
    /// Letterbox output for the final render pass viewport.
    letterbox: Rect<f32, f32>,
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

        let swapchain_capabilities = surface.get_capabilities(&adapter);

        // Configure the render surface
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Rgba8UnormSrgb,
            // Well be set by scaling
            width: 1,
            height: 1,
            present_mode: swapchain_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

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

        // Create the uniforms
        let screen_info = UniformState::new(
            &device,
            &ScreenInfo {
                buffer_size: [buffer_size.w as f32, buffer_size.h as f32],
                upscale_factor: PIXEL_UPSCALE as f32,
                ..Default::default()
            },
        );

        // Create the postprocessing effects
        let upscaled_pass = PostProcessingState::new(
            buffer_size * PIXEL_UPSCALE,
            &device,
            &screen_info,
            include_str!("./shaders/upscale.wgsl"),
        );

        // Create a custom pipeline for each component
        let sprite_render_state = RenderState::new(
            &device,
            &diffuse_texture_bind_group_layout,
            &screen_info.bind_group_layout,
        );

        // We don't have any textures uploaded yet
        let uploaded_textures = HashMap::new();

        // Construct a default empty render context
        let ctx = RenderContext::default();

        // The letterbox will be changed on resize
        let letterbox = Rect::new(0.0, 0.0, 1.0, 1.0);

        Ok(Self {
            surface,
            device,
            config,
            sprite_render_state,
            queue,
            diffuse_texture_bind_group_layout,
            screen_info,
            upscaled_pass,
            uploaded_textures,
            ctx,
            letterbox,
            buffer_size,
        })
    }

    /// Render the frame and call the user `render` function.
    pub(crate) fn render(&mut self) {
        // Upload the pending textures
        self.upload_textures();

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Pixel Game Command Encoder"),
            });

        // First pass, render the contents to an upscaled buffer
        if self.ctx.sprites.is_empty() {
            // Nothing to render, render the solid background color
            todo!()
        } else {
            // Render each sprite
            self.ctx.sprites.iter_mut().for_each(|(_, sprite)| {
                // Render the sprite
                self.sprite_render_state.render(
                    sprite,
                    &mut encoder,
                    &self.upscaled_pass.texture_view,
                    &self.queue,
                    &self.device,
                    &self.screen_info.bind_group,
                    &self.uploaded_textures,
                );
            });
        }

        // Get the main render texture
        let frame = self
            .surface
            .get_current_texture()
            .expect("Error acquiring next swap chain texture");

        // Second pass, downscale the upscaled buffer
        {
            // Create a texture view from the main render texture
            let view = frame.texture.create_view(&TextureViewDescriptor::default());

            // Start the render pass
            let mut upscaled_render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Upscaled Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
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

            upscaled_render_pass.set_pipeline(&self.upscaled_pass.render_pipeline);

            // Only draw in the calculated letterbox to get nice integer scaling
            let Rect { x, y, w, h } = self.letterbox;
            upscaled_render_pass.set_viewport(x, y, w, h, 0.0, 1.0);

            // Bind the source texture
            upscaled_render_pass.set_bind_group(0, &self.upscaled_pass.bind_group, &[]);

            // Bind the screen info uniform
            upscaled_render_pass.set_bind_group(1, &self.screen_info.bind_group, &[]);

            // Draw the 'buffer' defined in the vertex shader
            upscaled_render_pass.draw(0..3, 0..1);
        }

        // Draw to the texture
        self.queue.submit(Some(encoder.finish()));

        // Show the texture in the window
        frame.present();
    }

    // Resize the surface.
    pub(crate) fn resize(&mut self, new_size: Extent2<u32>) {
        log::debug!("Resizing the surface to {new_size}");

        // Ensure that the render surface is at least 1 pixel big, otherwise an error would occur
        self.config.width = new_size.w.max(1);
        self.config.height = new_size.h.max(1);
        self.surface.configure(&self.device, &self.config);

        // Recalculate the letterbox with the new size
        self.recalculate_letterbox();
    }

    /// Get a mutable reference to the render context for passing to the render call.
    pub(crate) fn ctx(&mut self) -> &mut RenderContext {
        &mut self.ctx
    }

    /// Recalculate the letterbox based on the size of the surface.
    fn recalculate_letterbox(&mut self) {
        // Calculate the integer scaling ratio first
        let screen_size = Extent2::new(self.config.width, self.config.height);
        let scale = if screen_size.h * self.buffer_size.w < screen_size.w * self.buffer_size.h {
            // Height fits
            screen_size.h / self.buffer_size.h
        } else {
            // Width fits
            screen_size.w / self.buffer_size.w
        }
        // We don't want a scale smaller than one
        .max(1);

        let scaled_buffer_size = self.buffer_size * scale;

        // Calculate the offset to center the scaled rectangle inside the other rectangle
        let offset = (screen_size - scaled_buffer_size) / 2;

        self.letterbox = Rect::new(
            offset.w,
            offset.h,
            scaled_buffer_size.w,
            scaled_buffer_size.h,
        )
        .as_();

        log::debug!(
            "Setting new letterbox to ({}:{} x {}:{}) with {scale} scaling",
            offset.w,
            offset.h,
            scaled_buffer_size.w,
            scaled_buffer_size.h
        );
    }

    /// Upload all pending textures.
    fn upload_textures(&mut self) {
        // Get a reference to the pending textures map
        let mut pending_textures = PENDING_TEXTURES
            // If it doesn't exist yet create a new one
            .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
            .lock()
            .expect("Error locking mutex");

        // Remove all pending textures and upload them
        pending_textures
            .drain()
            .for_each(|(texture_ref, pending_texture)| {
                if !self.uploaded_textures.contains_key(&texture_ref) {
                    log::debug!("Uploading texture '{texture_ref}'");

                    // Upload not-yet-uploaded textures
                    self.uploaded_textures.insert(
                        texture_ref.clone(),
                        pending_texture
                            .upload(&self.device, &self.diffuse_texture_bind_group_layout),
                    );
                }

                // Get a reference to possibly just uploaded state
                let uploaded_texture_state = self
                    .uploaded_textures
                    .get(&texture_ref)
                    .expect("Error getting uploaded texture");

                log::debug!("Writing texture data for '{texture_ref}'");

                // Write the pixels of the texture
                pending_texture.write(&self.queue, uploaded_texture_state);
            });
    }
}

/// State data collection for post processing stages.
struct PostProcessingState {
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    texture_view: TextureView,
}

impl PostProcessingState {
    /// Upload a new post processing state effect.
    fn new<T: NoUninit>(
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
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Post Processing Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        // Create the bind group binding the layout with the texture view
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Post Processing Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&texture_view),
            }],
        });

        // Create a new render pipeline first
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Post Processing Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &uniform.bind_group_layout],
            push_constant_ranges: &[],
        });

        // Load the shaders from disk
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
}

/// State data collection for uniforms for a shader.
struct UniformState<T: NoUninit> {
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
    buffer: Buffer,
    /// Store the type information.
    _phantom: PhantomData<T>,
}

impl<T: NoUninit> UniformState<T> {
    /// Upload a new uniform.
    fn new(device: &Device, initial_value: &T) -> Self {
        // Convert initial value to bytes
        let contents = bytemuck::bytes_of(initial_value);

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents,
            // Allow us to update this buffer
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create the bind group layout for passing the screen size
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Also already create the bind group, since it will be used without changing the size
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
        });

        Self {
            bind_group,
            buffer,
            bind_group_layout,
            _phantom: PhantomData,
        }
    }

    /// Update the value of the uniform.
    fn update(&mut self, value: &T, queue: &Queue) {
        // Convert value to bytes
        let data = bytemuck::bytes_of(value);

        // PUpdate the buffer and push to queue
        queue.write_buffer(&self.buffer, 0, data);
    }
}
