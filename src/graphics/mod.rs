//! Graphics state handling drawing items.

pub mod atlas;
mod uniform;

use std::sync::Arc;

use winit::window::Window;

use crate::config::Config;

use self::atlas::{Atlas, TextureRef};

/// Texture format we prefer to use for everything.
///
/// We choose sRGB since most source images are created with this format and otherwise everything will be quite dark.
pub(crate) const PREFERRED_TEXTURE_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Rgba8UnormSrgb;

/// Interface with the GPU.
pub(crate) struct Graphics {
    /// GPU device.
    pub(crate) device: wgpu::Device,
    /// GPU surface.
    pub(crate) surface: wgpu::Surface<'static>,
    /// GPU queue.
    pub(crate) queue: wgpu::Queue,
    /// GPU surface configuration.
    pub(crate) config: wgpu::SurfaceConfiguration,
    /// Texture atlas.
    pub(crate) atlas: Atlas,
}

impl Graphics {
    /// Upload a texture to the GPU.
    pub fn upload_texture(&mut self, width: u32, height: u32, pixels: &[u32]) -> TextureRef {
        self.atlas.add_texture(width, height, pixels, &self.queue)
    }

    /// Setup the GPU buffers and data structures.
    pub(crate) async fn new(config: &Config, window: Arc<Window>) -> Self {
        // Get a handle to our GPU
        let instance = wgpu::Instance::default();

        // Create a GPU surface on the window
        let surface = instance.create_surface(window).unwrap();

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
            .unwrap();

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
            .unwrap();

        // Configure the render surface
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: PREFERRED_TEXTURE_FORMAT,
            // Will be set by scaling
            width: config.buffer_width as u32,
            height: config.buffer_height as u32,
            present_mode: if config.vsync {
                wgpu::PresentMode::AutoVsync
            } else {
                wgpu::PresentMode::AutoNoVsync
            },
            desired_maximum_frame_latency: 2,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![PREFERRED_TEXTURE_FORMAT],
        };
        surface.configure(&device, &config);

        // Setup the texture atlas
        let atlas = Atlas::new(&device);

        Self {
            device,
            surface,
            queue,
            config,
            atlas,
        }
    }

    /// Render to the GPU and window.
    pub(crate) fn render(&self) {
        // Create the encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        // Get the main render texture
        let surface_texture = { self.surface.get_current_texture().unwrap() };

        // Create a texture view from the main render texture
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.queue.submit(Some(encoder.finish()));

        surface_texture.present();
    }

    /// Resize the render surface.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        // Ensure that the render surface is at least 1 pixel big, otherwise an error would occur
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
    }
}
