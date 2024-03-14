//! Main rendering state.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use miette::{IntoDiagnostic, Result, WrapErr};
use vek::Extent2;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance, Limits,
    PowerPreference, Queue, RequestAdapterOptionsBase, SamplerBindingType, ShaderStages, Surface,
    SurfaceConfiguration, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension, WindowHandle,
};

use crate::{sprite::Sprite, RenderContext};

use super::{
    render::RenderState,
    texture::{TextureRef, UploadedTextureState, PENDING_TEXTURES},
};

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
    /// Buffer for passing the screen size to the shaders.
    screen_size_buffer: Buffer,
    /// Bind group for passing the screen size to the shaders.
    screen_size_bind_group: BindGroup,
    /// Sprite component specific render pipelines.
    sprite_render_state: RenderState<Sprite>,
    /// Uploaded textures.
    uploaded_textures: HashMap<TextureRef, UploadedTextureState>,
    /// Render context passed to each user facing render frame.
    ctx: RenderContext,
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
            width: buffer_size.w,
            height: buffer_size.h,
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

        // We don't have any textures uploaded yet
        let uploaded_textures = HashMap::new();

        // Construct a default empty render context
        let ctx = RenderContext::default();

        Ok(Self {
            surface,
            device,
            config,
            sprite_render_state,
            queue,
            diffuse_texture_bind_group_layout,
            screen_size_buffer,
            screen_size_bind_group,
            uploaded_textures,
            ctx,
        })
    }

    /// Render the frame and call the user `render` function.
    pub(crate) fn render(&mut self) {
        // Upload the pending textures
        self.upload_textures();

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
                    &view,
                    &self.queue,
                    &self.device,
                    &self.screen_size_bind_group,
                    &self.uploaded_textures,
                );
            });
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

        // Update the screen size buffer to applied in the next render call
        let screen_size = [new_size.w as f32, new_size.h as f32];
        self.queue.write_buffer(
            &self.screen_size_buffer,
            0,
            bytemuck::cast_slice(&screen_size),
        );
    }

    /// Get a mutable reference to the render context for passing to the render call.
    pub(crate) fn ctx(&mut self) -> &mut RenderContext {
        &mut self.ctx
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
