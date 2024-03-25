//! Main rendering state.

use glamour::{Contains, Rect, Size2, Vector2};
use miette::{IntoDiagnostic, Result, WrapErr};
use wgpu::{
    Color, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance,
    Limits, PowerPreference, Queue, RequestAdapterOptionsBase, Surface, SurfaceConfiguration,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, WindowHandle,
};

use crate::Context;

use super::{
    atlas::Atlas, component::SpriteRenderState, data::ScreenInfo,
    post_processing::PostProcessingState, uniform::UniformState,
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
    /// Uniform screen info (size and scale) to the shaders.
    screen_info: UniformState<ScreenInfo>,
    /// Sprite component specific render pipelines.
    sprite_render_state: SpriteRenderState,
    /// Texture atlas.
    atlas: Atlas,
    /// Size of the final buffer to draw.
    ///
    /// Will be scaled with integer scaling and letterboxing to fit the screen.
    buffer_size: Size2<f32>,
    /// Post processing effect to downscale the result to a viewport with the exact buffer size.
    downscale: PostProcessingState,
    /// Letterbox output for the final render pass viewport.
    letterbox: Rect,
    /// Background color.
    background_color: Color,
    /// Viewport color
    viewport_color: Color,
}

impl<'window> MainRenderState<'window> {
    /// Create a GPU surface on the window.
    pub(crate) async fn new<W>(
        buffer_size: Size2,
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
            width: buffer_size.width as u32,
            height: buffer_size.height as u32,
            present_mode: swapchain_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![PREFERRED_TEXTURE_FORMAT],
        };
        surface.configure(&device, &config);

        // Create the uniforms
        let screen_info = UniformState::new(
            &device,
            &ScreenInfo {
                buffer_size: buffer_size.cast::<f32>(),
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

        // Create a new texture atlas
        let atlas = Atlas::new(&device);

        // Create a custom pipeline for each component
        let sprite_render_state =
            SpriteRenderState::new(&device, &screen_info.bind_group_layout, &atlas);

        // The letterbox will be changed on resize, but the size cannot be zero because then the buffer will crash
        let letterbox = Rect::new(Vector2::ZERO, Size2::splat(1.0));

        // Convert the u32 colors to WGPU colors
        let background_color = super::u32_to_wgpu_color(background_color);
        let viewport_color = super::u32_to_wgpu_color(viewport_color);

        Ok(Self {
            surface,
            device,
            config,
            sprite_render_state,
            queue,
            screen_info,
            buffer_size,
            letterbox,
            downscale,
            background_color,
            viewport_color,
            atlas,
        })
    }

    /// Render the frame and call the user `render` function.
    pub(crate) fn render(
        &mut self,
        ctx: &mut Context,
        mut custom_pass_cb: impl FnMut(&Device, &Queue, &mut CommandEncoder, &TextureView),
    ) {
        // Upload the pending textures
        self.upload_textures(ctx);

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

        // Determine whether we need a downscale pass, we know this if the letterbox is at position zero it fits exactly
        let needs_downscale_pass = self.letterbox.origin.x != 0.0 || self.letterbox.origin.y != 0.0;

        // First pass, render the contents to a custom buffer
        ctx.read(|ctx| {
            profiling::scope!("Render sprites");

            // If we need a downscale pass use that as the texture target, otherwise use the framebuffer directly
            let target_texture_view = if needs_downscale_pass {
                &self.downscale.texture_view
            } else {
                &surface_view
            };

            // Render the sprites
            self.sprite_render_state.render(
                &ctx.instances,
                &mut encoder,
                target_texture_view,
                &self.queue,
                &self.device,
                &self.screen_info.bind_group,
                &self.atlas,
                self.background_color,
            );
        });

        // Second pass, render the custom buffer to the viewport
        if needs_downscale_pass {
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
    pub(crate) fn resize(&mut self, new_size: Size2<u32>) {
        log::debug!(
            "Resizing the surface to ({}x{})",
            new_size.width,
            new_size.height
        );

        // Ensure that the render surface is at least 1 pixel big, otherwise an error would occur
        self.config.width = new_size.width.max(1);
        self.config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.config);

        // Recalculate the letterbox with the new size
        self.recalculate_letterbox();
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
    pub(crate) fn screen_size(&self) -> Size2<f32> {
        Size2::new(self.config.width as f32, self.config.height as f32)
    }

    /// Map a coordinate to relative coordinates of the buffer in the letterbox.
    pub(crate) fn map_coordinate(&self, coordinate: Vector2) -> Option<Vector2> {
        // On desktop map the cursor to the viewport

        // Ignore all coordinates outside of the letterbox
        if !self.letterbox.contains(coordinate.as_point()) {
            return None;
        }

        // Calculate the scale from the letterbox
        let scale = self.letterbox.width() / self.buffer_size.width;

        Some((coordinate - self.letterbox.origin.to_vector()) / scale)
    }

    /// Recalculate the letterbox based on the size of the surface.
    ///
    /// # Panics
    ///
    /// - When resulting letterbox size is zero.
    fn recalculate_letterbox(&mut self) {
        // Calculate the integer scaling ratio first
        let buffer_width_u32 = self.buffer_size.width as u32;
        let buffer_height_u32 = self.buffer_size.height as u32;
        let scale =
            if self.config.height * buffer_width_u32 < self.config.width * buffer_height_u32 {
                // Height fits
                self.config.height / buffer_height_u32
            } else {
                // Width fits
                self.config.width / buffer_width_u32
            }
            // We don't want a scale smaller than one
            .max(1);

        let scaled_buffer_size = self.buffer_size * scale as f32;

        // Calculate the offset to center the scaled rectangle inside the other rectangle
        let offset = ((self.screen_size() - scaled_buffer_size) / 2.0)
            .to_vector()
            .round();

        self.letterbox = Rect::new(offset, scaled_buffer_size).cast();

        log::debug!(
            "Setting new letterbox to ({}:{} x {}:{}) with {scale} scaling",
            offset.x,
            offset.y,
            scaled_buffer_size.width,
            scaled_buffer_size.height
        );

        assert!(
            self.letterbox.width() > 0.0 && self.letterbox.height() > 0.0,
            "Error with invalid letterbox size dimensions"
        );
    }

    /// Upload all pending textures.
    fn upload_textures(&mut self, ctx: &mut Context) {
        profiling::scope!("Upload pending textures");

        // Upload the un-uploaded sprites
        ctx.write(|ctx| {
            ctx.sprites_iter_mut().for_each(|sprite| {
                sprite.image.upload(&mut self.atlas, &self.queue);
            })
        });
    }
}
