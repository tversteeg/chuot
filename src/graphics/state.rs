//! Main rendering state.

use std::sync::{Arc, Mutex};

use hashbrown::HashMap;
use miette::{IntoDiagnostic, Result, WrapErr};
use vek::{Extent2, Rect, Vec2};
use wgpu::{
    BindGroupLayout, Color, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor,
    Features, Instance, Limits, PowerPreference, Queue, RequestAdapterOptionsBase, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
    WindowHandle,
};

use crate::{sprite::Sprite, RenderContext};

use super::{
    component::RenderState,
    data::ScreenInfo,
    post_processing::PostProcessingState,
    texture::{TextureRef, UploadedTextureState, PENDING_TEXTURES},
    uniform::UniformState,
};

/// Texture format we prefer to use for everything.
///
/// We choose sRGB since most source images are created with this format and otherwise everything will be quite dark.
pub(crate) const PREFERRED_TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

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
    /// Post processing effect to downscale the result to a viewport with the exact buffer size.
    downscale: PostProcessingState,
    /// Letterbox output for the final render pass viewport.
    letterbox: Rect<f32, f32>,
    /// Background color.
    background_color: Color,
    /// Viewport color
    viewport_color: Color,
}

impl<'window> MainRenderState<'window> {
    /// Create a GPU surface on the window.
    pub(crate) async fn new<W>(
        buffer_size: Extent2<u32>,
        window: W,
        background_color: u32,
        viewport_color: u32,
    ) -> Result<Self>
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

        // Get the surface capabilities
        let swapchain_capabilities = surface.get_capabilities(&adapter);

        // Create the logical device and command queue
        let adapter_result = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    // WebGL doesn't support all features, so use the lowest limits
                    // On desktop we can use a cfg! flag to set it to defaults, but this will allow us to create an application that might not work on the web
                    required_limits: Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
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

        // Configure the render surface
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: PREFERRED_TEXTURE_FORMAT,
            // Will be set by scaling
            width: buffer_size.w,
            height: buffer_size.h,
            present_mode: swapchain_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![PREFERRED_TEXTURE_FORMAT],
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
                ..Default::default()
            },
        );

        // Create the postprocessing effects
        let downscale = PostProcessingState::new(
            buffer_size,
            &device,
            &screen_info,
            include_str!("./shaders/downscale.wgsl"),
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

        // Convert the u32 colors to WGPU colors
        let background_color = super::u32_to_wgpu_color(background_color);
        let viewport_color = super::u32_to_wgpu_color(viewport_color);

        Ok(Self {
            surface,
            device,
            config,
            sprite_render_state,
            queue,
            diffuse_texture_bind_group_layout,
            screen_info,
            uploaded_textures,
            ctx,
            buffer_size,
            letterbox,
            downscale,
            background_color,
            viewport_color,
        })
    }

    /// Render the frame and call the user `render` function.
    pub(crate) fn render(
        &mut self,
        mut custom_pass_cb: impl FnMut(&Device, &Queue, &mut CommandEncoder, &TextureView),
    ) {
        // Upload the pending textures
        self.upload_textures();

        let mut encoder = {
            profiling::scope!("Create command encoder");

            self.device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Pixel Game Command Encoder"),
                })
        };

        // Get the main render texture
        let surface_texture = {
            profiling::scope!("Retrieve surface texture");

            self.surface
                .get_current_texture()
                .expect("Error acquiring next swap chain texture")
        };

        // Create a texture view from the main render texture
        let surface_view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        // First pass, render the contents to a custom buffer
        if self.ctx.sprites.is_empty() {
            // Nothing to render, render the solid background color
            todo!()
        } else {
            profiling::scope!("Render sprites");

            // Render each sprite
            self.ctx.sprites.iter_mut().for_each(|(_, sprite)| {
                // Render the sprite
                self.sprite_render_state.render(
                    sprite,
                    &mut encoder,
                    &self.downscale.texture_view,
                    &self.queue,
                    &self.device,
                    &self.screen_info.bind_group,
                    &self.uploaded_textures,
                    self.background_color,
                );
            });
        }

        // Second pass, render the custom buffer to the viewport
        {
            profiling::scope!("Render downscale pass");

            self.downscale.render(
                &mut encoder,
                &surface_view,
                &self.screen_info,
                Some(self.letterbox),
                self.viewport_color,
            );
        }

        // Call the callback that allows other parts of the program to add a render pass
        custom_pass_cb(&self.device, &self.queue, &mut encoder, &surface_view);

        // Draw to the texture
        {
            profiling::scope!("Submit queue");

            self.queue.submit(Some(encoder.finish()));
        }

        // Show the texture in the window
        {
            profiling::scope!("Present surface texture");

            surface_texture.present();
        }
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
    pub(crate) fn ctx_mut(&mut self) -> &mut RenderContext {
        &mut self.ctx
    }

    /// Get a reference to WGPU device.
    ///
    /// Is allowed to be unused because the `in-game-profiler` feature flag uses it.
    #[allow(unused)]
    pub(crate) fn device(&self) -> &Device {
        &self.device
    }

    /// Size of the screen in pixels.
    ///
    /// Is allowed to be unused because the `in-game-profiler` feature flag uses it.
    #[allow(unused)]
    pub(crate) fn screen_size(&self) -> Extent2<u32> {
        Extent2::new(self.config.width, self.config.height)
    }

    /// Map a coordinate to relative coordinates of the buffer in the letterbox.
    pub(crate) fn map_coordinate(&self, coordinate: Vec2<f32>) -> Option<Vec2<f32>> {
        // On desktop map the cursor to the viewport

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
        profiling::scope!("Upload pending textures");

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
