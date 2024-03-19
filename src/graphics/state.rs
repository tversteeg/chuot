//! Main rendering state.

use std::sync::{Arc, Mutex};

use hashbrown::HashMap;
use miette::{IntoDiagnostic, Result, WrapErr};
use vek::{Extent2, Rect, Vec2};
use wgpu::{
    BindGroupLayout, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance,
    Limits, PowerPreference, Queue, RequestAdapterOptionsBase, Surface, SurfaceConfiguration,
    TextureFormat, TextureUsages, TextureViewDescriptor, WindowHandle,
};

use crate::{sprite::Sprite, RenderContext};

use super::{
    component::RenderState,
    data::ScreenInfo,
    post_processing::PostProcessingState,
    texture::{TextureRef, UploadedTextureState, PENDING_TEXTURES},
    uniform::UniformState,
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

        log::debug!("Creating GPU surface on the window");

        // Create a GPU surface on the window
        let surface = instance
            .create_surface(window)
            .into_diagnostic()
            .wrap_err("Error creating surface on window")?;

        log::debug!("Requesting adapter");

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
        let adapter_result = adapter
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
            .await;

        // For some reason `into_diagnostic` doesn't work for this call on WASM
        #[cfg(target_arch = "wasm32")]
        let (device, queue) = adapter_result.expect("Error getting logical GPU device for surface");
        #[cfg(not(target_arch = "wasm32"))]
        let (device, queue) = adapter_result
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
            super::texture::create_bind_group_layout(&device, "Diffuse Texture Bind Group Layout");

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

        // The letterbox will be changed on resize, but the size cannot be zero because then the buffer will crash
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

        // Create a texture view from the main render texture
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        // Second pass, downscale the upscaled buffer
        self.upscaled_pass
            .render(&mut encoder, &view, &self.screen_info, Some(self.letterbox));

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

    /// Map a coordinate to relative coordinates of the buffer in the letterbox.
    pub(crate) fn map_coordinate(&self, coordinate: Vec2<f32>) -> Option<Vec2<f32>> {
        // Ignore all coordinates outside of the letterbox
        if !self.letterbox.contains_point(coordinate) {
            return None;
        }

        // Calculate the scale from the letterbox
        let scale = self.letterbox.w / self.buffer_size.w as f32;

        Some((coordinate - self.letterbox.position()) / scale)
    }

    /// Recalculate the letterbox based on the size of the surface.
    ///
    /// # Panics
    ///
    /// - When resulting letterbox size is zero.
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

        assert!(
            self.letterbox.w > 0.0 && self.letterbox.h > 0.0,
            "Error with invalid letterbox size dimensions"
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
