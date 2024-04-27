//! Main rendering state.

use glamour::{Contains, Rect, Size2, Vector2};
use miette::Result;
use winit::window::Window;

use crate::{
    assets::{loader::png::PngReader, EmbeddedRawStaticAtlas},
    window::InGameProfiler,
    Context, GameConfig,
};

use super::{
    atlas::{Atlas, AtlasRef},
    component::SpriteRenderState,
    data::ScreenInfo,
    gpu::Gpu,
    post_processing::PostProcessingState,
    uniform::UniformState,
};

/// Texture format we prefer to use for everything.
///
/// We choose sRGB since most source images are created with this format and otherwise everything will be quite dark.
pub(crate) const PREFERRED_TEXTURE_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Rgba8UnormSrgb;

/// Main render state holding the GPU information.
pub(crate) struct MainRenderState<'window> {
    /// GPU state.
    pub(crate) gpu: Gpu<'window>,
    /// Uniform screen info (size and scale) to the shaders.
    screen_info: UniformState<ScreenInfo>,
    /// Sprite component specific render pipelines.
    sprite_render_state: SpriteRenderState,
    /// Size of the final buffer to draw.
    ///
    /// Will be scaled with integer scaling and letterboxing to fit the screen.
    buffer_size: Size2<f32>,
    /// Post processing effect to downscale the result to a viewport with the exact buffer size.
    downscale: PostProcessingState,
    /// Letterbox output for the final render pass viewport.
    letterbox: Rect,
    /// Background color.
    background_color: wgpu::Color,
    /// Viewport color
    viewport_color: wgpu::Color,
    /// Image atlas.
    atlas: Atlas,
}

impl<'window> MainRenderState<'window> {
    /// Create a GPU surface on the window.
    pub(crate) async fn new<W>(
        game_config: &GameConfig,
        embedded_atlas: EmbeddedRawStaticAtlas,
        window: W,
    ) -> Result<Self>
    where
        W: wgpu::WindowHandle + 'window,
    {
        // Setup the GPU and attach it to the window surface
        let gpu = Gpu::new(game_config, window).await?;

        // Create the uniforms
        let screen_info = UniformState::new(
            &gpu.device,
            &ScreenInfo {
                buffer_size: game_config.buffer_size.cast::<f32>(),
                ..Default::default()
            },
        );

        // Create the postprocessing effects
        let downscale = PostProcessingState::new(
            game_config.buffer_size,
            &gpu.device,
            &screen_info,
            include_str!(concat!(env!("OUT_DIR"), "/downscale.wgsl")),
        );

        // Create the texture atlas
        let atlas = embedded_atlas.parse_and_upload(&gpu);

        // Create a custom pipeline for each component
        let sprite_render_state = SpriteRenderState::new(
            &gpu.device,
            &screen_info.bind_group_layout,
            &atlas,
            game_config.rotation_algorithm,
        );

        // The letterbox will be changed on resize on the desktop
        let letterbox = Rect::new(Vector2::ZERO, game_config.buffer_size * game_config.scaling);

        // Convert the u32 colors to WGPU colors
        let background_color = super::u32_to_wgpu_color(game_config.background_color);
        let viewport_color = super::u32_to_wgpu_color(game_config.viewport_color);

        let buffer_size = game_config.buffer_size;

        Ok(Self {
            gpu,
            sprite_render_state,
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
    #[allow(unused_variables)]
    pub(crate) fn render(
        &mut self,
        ctx: &mut Context,
        in_game_profiler: &mut InGameProfiler,
        window: &Window,
    ) {
        // Profile the allocations
        #[cfg(feature = "in-game-profiler")]
        let profile_region = InGameProfiler::start_profile_heap();

        #[cfg(feature = "in-game-profiler")]
        in_game_profiler.finish_profile_heap("Texture upload", profile_region);
        #[cfg(feature = "in-game-profiler")]
        let profile_region = InGameProfiler::start_profile_heap();

        // Get the screen size early because we can't access it later due to borrowing
        #[cfg(feature = "in-game-profiler")]
        let screen_size = self.screen_size();

        // Render on the GPU
        let mut frame = self.gpu.start();

        // Determine whether we need a downscale pass, we know this if the letterbox is at position zero it fits exactly
        let needs_downscale_pass = !cfg!(target_arch = "wasm32")
            && (self.letterbox.origin.x != 0.0 || self.letterbox.origin.y != 0.0);

        // If we need a downscale pass use that as the texture target, otherwise use the framebuffer directly
        let target_texture_view = if needs_downscale_pass {
            Some(&self.downscale.texture_view)
        } else {
            None
        };

        // First pass, render the contents to a custom buffer
        ctx.read(|ctx| {
            profiling::scope!("Render sprites");

            // Render the sprites
            self.sprite_render_state.render(
                &ctx.instances,
                &mut frame,
                target_texture_view,
                &self.screen_info.bind_group,
                &self.atlas,
                self.background_color,
            );
        });

        // Second optional pass, render the custom buffer to the viewport
        if needs_downscale_pass {
            profiling::scope!("Render downscale pass");

            self.downscale.render(
                &mut frame,
                None,
                &self.screen_info,
                Some(self.letterbox),
                self.viewport_color,
            );
        }

        #[cfg(feature = "in-game-profiler")]
        in_game_profiler.finish_profile_heap("Render", profile_region);

        // Call the callback that allows other parts of the program to add a render pass
        #[cfg(feature = "in-game-profiler")]
        in_game_profiler.render(
            &mut frame.encoder,
            &frame.surface_view,
            window,
            frame.device,
            frame.queue,
            screen_size,
        );

        #[cfg(feature = "in-game-profiler")]
        let profile_region = InGameProfiler::start_profile_heap();

        // Render the frame
        frame.present();

        #[cfg(feature = "in-game-profiler")]
        in_game_profiler.finish_profile_heap("Present frame", profile_region);
    }

    /// Upload everything that gets added at another point in time where the graphics state wasn't available.
    pub(crate) fn upload(&mut self, images: Vec<(AtlasRef, PngReader)>) {
        images.into_iter().for_each(|(atlas_id, mut image_reader)| {
            // Read the PNG
            let mut buf = vec![0; image_reader.output_buffer_size()];
            let info = image_reader
                .next_frame(&mut buf)
                .expect("Error reading image");

            // Upload the PNG
            let uploaded_id = self.atlas.add_texture(
                Size2::new(info.width, info.height),
                bytemuck::cast_slice(&buf),
                &self.gpu.queue,
            );
            assert_eq!(uploaded_id, atlas_id);
        });

        // Upload uniforsm
        self.atlas.rects.upload(&self.gpu.queue);
    }

    /// Resize the surface.
    ///
    /// Only resize the surface on the desktop, on the web we keep the canvas the same size.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn resize(&mut self, new_size: Size2<u32>) {
        // Resize the surface
        self.gpu.resize(new_size);

        // Recalculate the letterbox with the new size
        self.recalculate_letterbox();
    }

    /// Get a reference to WGPU device.
    ///
    /// Is allowed to be unused because the `in-game-profiler` feature flag uses it.
    #[allow(unused)]
    pub(crate) fn device(&self) -> &wgpu::Device {
        &self.gpu.device
    }

    /// Size of the screen in pixels.
    ///
    /// Is allowed to be unused because the `in-game-profiler` feature flag uses it.
    #[allow(unused)]
    pub(crate) fn screen_size(&self) -> Size2<u32> {
        self.gpu.screen_size()
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
    #[cfg(not(target_arch = "wasm32"))]
    fn recalculate_letterbox(&mut self) {
        // Calculate the integer scaling ratio first
        let buffer_width_u32 = self.buffer_size.width as u32;
        let buffer_height_u32 = self.buffer_size.height as u32;

        let screen_size = self.gpu.screen_size();
        let screen_width_u32 = screen_size.width;
        let screen_height_u32 = screen_size.height;

        let scale = {
            if screen_height_u32 * buffer_width_u32 < screen_width_u32 * buffer_height_u32 {
                // Height fits
                screen_height_u32 / buffer_height_u32
            } else {
                // Width fits
                screen_width_u32 / buffer_width_u32
            }
            // We don't want a scale smaller than one
            .max(1)
        };

        let scaled_buffer_size = Size2::new(buffer_width_u32, buffer_height_u32) * scale;

        // Calculate the offset to center the scaled rectangle inside the other rectangle
        let offset = (self.screen_size() - scaled_buffer_size).to_vector() / 2;

        self.letterbox = Rect::new(
            Vector2::new(offset.x as f32, offset.y as f32),
            Size2::new(
                scaled_buffer_size.width as f32,
                scaled_buffer_size.height as f32,
            ),
        );

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
}
