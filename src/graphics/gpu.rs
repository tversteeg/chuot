//! Abstractions over GPU calls which can be profiled.

use glamour::Size2;
use miette::{Context, IntoDiagnostic, Result};

use crate::{graphics::state::PREFERRED_TEXTURE_FORMAT, GameConfig};

/// GPU state abstracted so GPU calls can be profiled if the feature flags are enabled.
pub(crate) struct Gpu<'window> {
    /// GPU device.
    pub(crate) device: wgpu::Device,
    /// GPU surface.
    pub(crate) surface: wgpu::Surface<'window>,
    /// GPU queue.
    pub(crate) queue: wgpu::Queue,
    /// GPU surface configuration.
    config: wgpu::SurfaceConfiguration,
}

impl<'window> Gpu<'window> {
    /// Create a GPU surface on the window.
    pub(crate) async fn new<W>(game_config: &GameConfig, window: W) -> Result<Self>
    where
        W: wgpu::WindowHandle + 'window,
    {
        // Get a handle to our GPU
        let instance = wgpu::Instance::default();

        log::debug!("Creating GPU surface on the window");

        // Create a GPU surface on the window
        let surface = instance
            .create_surface(window)
            .into_diagnostic()
            .wrap_err("Error creating surface on window")?;

        log::debug!("Requesting adapter");

        // Request an adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                // Ensure the strongest GPU is used
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                // Request an adaptar which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or_else(|| miette::miette!("Error getting GPU adapter for window"))?;

        // Get the surface capabilities
        let swapchain_capabilities = surface.get_capabilities(&adapter);

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all features, so use the lowest limits
                    // On desktop we can use a cfg! flag to set it to defaults, but this will allow us to create an application that might not work on the web
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .into_diagnostic()
            .wrap_err("Error getting logical GPU device for surface")?;

        // Configure the render surface
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: PREFERRED_TEXTURE_FORMAT,
            // Will be set by scaling
            width: game_config.buffer_size.width as u32,
            height: game_config.buffer_size.height as u32,
            present_mode: if game_config.vsync {
                wgpu::PresentMode::AutoVsync
            } else {
                wgpu::PresentMode::AutoNoVsync
            },
            desired_maximum_frame_latency: 2,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![PREFERRED_TEXTURE_FORMAT],
        };
        surface.configure(&device, &config);

        Ok(Self {
            device,
            surface,
            queue,
            config,
        })
    }

    /// Start a new rendering event.
    #[inline]
    pub(crate) fn start(&mut self) -> Frame {
        profiling::scope!("Create command encoder");

        // Create the encoder
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Pixel Game Command Encoder"),
            });

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
            .create_view(&wgpu::TextureViewDescriptor::default());

        Frame {
            encoder,
            surface_view,
            surface_texture,
            device: &self.device,
            queue: &self.queue,
        }
    }

    /// Resize the surface.
    ///
    /// Only resize the surface on the desktop, on the web we keep the canvas the same size.
    #[cfg(not(target_arch = "wasm32"))]
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
    }

    /// Size of the screen in pixels.
    ///
    /// Is allowed to be unused because the `in-game-profiler` feature flag uses it.
    #[inline]
    pub(crate) fn screen_size(&self) -> Size2<u32> {
        Size2::new(self.config.width, self.config.height)
    }
}

/// Rendering state for a single frame.
pub(crate) struct Frame<'gpu> {
    /// GPU command encoder.
    pub(crate) encoder: wgpu::CommandEncoder,
    /// GPU surface view.
    pub(crate) surface_view: wgpu::TextureView,
    /// GPU surface texture.
    pub(crate) surface_texture: wgpu::SurfaceTexture,
    /// GPU device.
    pub(crate) device: &'gpu wgpu::Device,
    /// GPU queue.
    pub(crate) queue: &'gpu wgpu::Queue,
}

impl<'gpu> Frame<'gpu> {
    /// Finish rendering event.
    #[inline]
    pub(crate) fn present(self) {
        // Draw to the texture
        {
            profiling::scope!("Submit queue");

            self.queue.submit(Some(self.encoder.finish()));
        }

        // Show the texture in the window
        {
            profiling::scope!("Present surface texture");

            self.surface_texture.present();
        }
    }
}
