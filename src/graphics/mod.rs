//! Graphics state handling drawing items.

pub mod atlas;
mod data;
mod instance;
mod post_processing;
mod uniform;

use std::{borrow::Cow, sync::Arc};

#[cfg(feature = "embed-assets")]
use imgref::ImgVec;
use rgb::RGBA8;
use wgpu::util::DeviceExt as _;
use winit::window::Window;

use self::{
    atlas::{Atlas, TextureRef},
    data::{ScreenInfo, TexturedVertex},
    instance::Instances,
    post_processing::PostProcessingState,
    uniform::UniformState,
};
#[cfg(feature = "embed-assets")]
use crate::assets::{
    Id,
    loader::{Loader as _, png::PngLoader},
};
use crate::{
    AssetSource,
    config::{Config, RotationAlgorithm},
};

/// Texture format we prefer to use for everything.
///
/// We choose sRGB since most source images are created with this format and otherwise everything will be quite dark.
pub(crate) const PREFERRED_TEXTURE_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Rgba8UnormSrgb;

/// Interface with the GPU.
pub(crate) struct Graphics {
    /// Reference to the winit window.
    pub(crate) window: Arc<Window>,
    /// GPU device.
    pub(crate) device: wgpu::Device,
    /// GPU surface.
    pub(crate) surface: wgpu::Surface<'static>,
    /// GPU queue.
    pub(crate) queue: wgpu::Queue,
    /// GPU surface configuration.
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
    /// Pipeline of the rendering itself.
    pub(crate) render_pipeline: wgpu::RenderPipeline,
    /// GPU buffer reference to the vertices of the texture squares.
    pub(crate) vertex_buffer: wgpu::Buffer,
    /// GPU buffer reference to the indices of the texture squares.
    pub(crate) index_buffer: wgpu::Buffer,
    /// GPU buffer reference to all instances of the texture squares.
    pub(crate) instance_buffer: wgpu::Buffer,
    /// Texture atlas.
    pub(crate) atlas: Atlas,
    /// Width of the final buffer to draw.
    ///
    /// Will be scaled with integer scaling and letterboxing to fit the screen.
    pub(crate) buffer_width: f32,
    /// Height of the final buffer to draw.
    ///
    /// Will be scaled with integer scaling and letterboxing to fit the screen.
    pub(crate) buffer_height: f32,
    /// Uniform screen info (size and scale) to the shaders.
    pub(crate) screen_info: UniformState<ScreenInfo>,
    /// Post processing effect to downscale the result to a viewport with the exact buffer size.
    pub(crate) downscale: PostProcessingState,
    /// All instances to render.
    pub(crate) instances: Instances,
    /// Letterbox output `(x, y, width, height)` for the final render pass viewport.
    pub(crate) letterbox: (f32, f32, f32, f32),
    /// Background color.
    pub(crate) background_color: wgpu::Color,
    /// Viewport color
    pub(crate) viewport_color: wgpu::Color,
}

impl Graphics {
    /// Upload a texture to the GPU.
    pub fn upload_texture(&mut self, width: u32, height: u32, pixels: &[RGBA8]) -> TextureRef {
        self.atlas.add_texture(width, height, pixels, &self.queue)
    }

    /// Setup the GPU buffers and data structures.
    pub(crate) async fn new(
        Config {
            buffer_width,
            buffer_height,
            scaling,
            vsync,
            viewport_color,
            background_color,
            rotation_algorithm,
            ..
        }: Config,
        window: Arc<Window>,
        asset_source: &AssetSource,
    ) -> Self {
        // Get a handle to our GPU
        let instance = wgpu::Instance::default();

        // Create a GPU surface on the window
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

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

        // Check that we support the texture format
        assert!(
            swapchain_capabilities
                .formats
                .contains(&PREFERRED_TEXTURE_FORMAT),
            "{PREFERRED_TEXTURE_FORMAT:?} texture format not supported"
        );

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
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .unwrap();

        let width = buffer_width as u32;
        let height = buffer_height as u32;

        // Configure the render surface
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: PREFERRED_TEXTURE_FORMAT,
            width,
            height,
            present_mode: if vsync {
                wgpu::PresentMode::AutoVsync
            } else {
                wgpu::PresentMode::AutoNoVsync
            },
            desired_maximum_frame_latency: 2,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![PREFERRED_TEXTURE_FORMAT],
        };
        surface.configure(&device, &surface_config);

        // Setup the texture atlas
        let embedded_atlas = asset_source.embedded_atlas();
        #[allow(unused_mut)]
        let mut atlas = Atlas::new(embedded_atlas.textures.len(), &device, &queue);

        // Upload embedded assets to atlas
        #[cfg(feature = "embed-assets")]
        if !embedded_atlas.textures.is_empty() {
            // Read the PNG, using the loader
            let (width, height, pixels) = PngLoader::load(
                embedded_atlas.diced_atlas_png_bytes,
                &Id::new_inline("_embedded_atlas"),
            );

            // Treat the 4 color components as a single numeric value
            let img = ImgVec::new(pixels, width as usize, height as usize);

            // Upload all textures
            for texture in embedded_atlas.textures.values() {
                // Create an empty texture we can upload all parts to
                atlas.add_preallocated_empty_texture(
                    texture.reference,
                    texture.width as u32,
                    texture.height as u32,
                    &queue,
                );

                // Upload all diced parts
                for diced in texture.diced {
                    // Copy the pixels from the slice into the target
                    let diced_texture = img.sub_image(
                        diced.diced_u as usize,
                        diced.diced_v as usize,
                        diced.width as usize,
                        diced.height as usize,
                    );

                    // Upload to the GPU
                    atlas.update_pixels(
                        texture.reference,
                        (
                            diced.texture_u as f32,
                            diced.texture_v as f32,
                            diced.width as f32,
                            diced.height as f32,
                        ),
                        &diced_texture.pixels().collect::<Vec<_>>(),
                        &queue,
                    );
                }
            }
        }

        // Create the uniforms
        let screen_info = UniformState::new(&device, &ScreenInfo {
            width: buffer_width,
            height: buffer_height,
            half_width: buffer_width / 2.0,
            half_height: buffer_height / 2.0,
        });

        // Create a new render pipeline first
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Component Render Pipeline Layout"),
                bind_group_layouts: &[
                    &atlas.bind_group_layout,
                    &atlas.rects.bind_group_layout,
                    &screen_info.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        // Load the shaders from disk
        let shader_source = match rotation_algorithm {
            // Load the optimized nearest neighbor shader
            RotationAlgorithm::NearestNeighbor => {
                include_str!(concat!(env!("OUT_DIR"), "/nearest_neighbor.wgsl"))
            }
            // All other shaders are in the rotation file
            _ => include_str!(concat!(env!("OUT_DIR"), "/rotation.wgsl")),
        };

        // Upload the shader to the GPU
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Diffuse Texture Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_source)),
        });

        // Create the pipeline for rendering textures
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                buffers: &[TexturedVertex::descriptor(), Instances::descriptor()],
                module: &shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: match rotation_algorithm {
                    RotationAlgorithm::CleanEdge => "fs_main_clean_edge",
                    RotationAlgorithm::Scale3x => "fs_main_scale3x",
                    RotationAlgorithm::Scale2x => "fs_main_scale2x",
                    RotationAlgorithm::Diag2x => "fs_main_diag2x",
                    RotationAlgorithm::NearestNeighbor => "fs_main_nearest_neighbor",
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: PREFERRED_TEXTURE_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                // Irrelevant since we disable culling
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                // How many samples the pipeline will use
                count: 1,
                // Use all masks
                mask: !0,
                // Disable anti-aliasing
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Create the initial empty instance buffer, will be resized by the render call
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // Vertices for a rectangle
        let vertices = [
            // Top left
            TexturedVertex::new(0.0, 0.0, 0.0, 0.0, 0.0),
            // Top right
            TexturedVertex::new(1.0, 0.0, 0.0, 1.0, 0.0),
            // Bottom right
            TexturedVertex::new(1.0, 1.0, 0.0, 1.0, 1.0),
            // Bottom left
            TexturedVertex::new(0.0, 1.0, 0.0, 0.0, 1.0),
        ];

        // Indices for a rectangle
        let indices: [u16; 6] = [0, 1, 3, 3, 1, 2];

        // Create the vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // Create the index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        // Create the postprocessing effects
        let downscale = PostProcessingState::new(
            width,
            height,
            &device,
            &screen_info,
            include_str!(concat!(env!("OUT_DIR"), "/downscale.wgsl")),
        );

        // Create the instances list
        let instances = Instances::default();

        // Full size letterbox, will be rescaled
        let letterbox = (0.0, 0.0, buffer_width * scaling, buffer_height * scaling);

        // Convert the u32 colors to WGPU colors
        let background_color = rgba_to_wgpu_color(background_color);
        let viewport_color = rgba_to_wgpu_color(viewport_color);

        Self {
            window,
            device,
            surface,
            queue,
            surface_config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            atlas,
            buffer_width,
            buffer_height,
            screen_info,
            downscale,
            instances,
            letterbox,
            background_color,
            viewport_color,
        }
    }

    /// Render to the GPU and window.
    pub(crate) fn render(&mut self) {
        // Get the main render texture
        let surface_texture = self.surface.get_current_texture().unwrap();

        // Create the encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        // Create a texture view from the main render texture
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // First pass, render all instances
        self.render_instances(&mut encoder, None);

        // Second pass, render the custom buffer to the viewport
        self.downscale.render(
            &mut encoder,
            &surface_view,
            &self.screen_info,
            Some(self.letterbox),
            self.viewport_color,
        );

        // Send all the queued items to draw to the surface texture
        self.queue.submit(Some(encoder.finish()));

        // Tell winit we are going to draw something
        self.window.pre_present_notify();

        // Show the surface texture in the window
        surface_texture.present();
    }

    /// Resize the render surface.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        // Ensure that the render surface is at least 1 pixel big, otherwise an error would occur
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);

        // Recalculate the letterbox

        // Calculate the integer scaling ratio first
        let buffer_width_u32 = self.buffer_width.round() as u32;
        let buffer_height_u32 = self.buffer_height.round() as u32;

        let screen_width_u32 = self.surface_config.width;
        let screen_height_u32 = self.surface_config.height;

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
        // Calculate the new size with the scale
        let scaled_buffer_width = buffer_width_u32 * scale;
        let scaled_buffer_height = buffer_height_u32 * scale;

        // Calculate the offset to center the scaled rectangle inside the other rectangle
        let offset_x = screen_width_u32.saturating_sub(scaled_buffer_width) / 2;
        let offset_y = screen_height_u32.saturating_sub(scaled_buffer_height) / 2;

        self.letterbox = (
            offset_x as f32,
            offset_y as f32,
            scaled_buffer_width as f32,
            scaled_buffer_height as f32,
        );

        assert!(
            scaled_buffer_width > 0 && scaled_buffer_height > 0,
            "Error with invalid letterbox size dimensions"
        );
    }

    /// Map an absolute window coordinate to a relative coordinate of the buffer in the letterbox.
    pub(crate) fn map_window_coordinate(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        // On desktop map the cursor to the viewport

        let (letterbox_x, letterbox_y, letterbox_width, letterbox_height) = self.letterbox;

        // Ignore all coordinates outside of the letterbox
        if x < letterbox_x
            || y < letterbox_y
            || x >= letterbox_x + letterbox_width
            || y >= letterbox_y + letterbox_height
        {
            return None;
        }

        // Calculate the scale from the letterbox
        let scale = letterbox_width / self.buffer_width;

        // Map the coordinates based on the scale and offset of the letterbox
        Some(((x - letterbox_x) / scale, (y - letterbox_y) / scale))
    }

    /// Render the instances if applicable.
    fn render_instances(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        custom_view: Option<&wgpu::TextureView>,
    ) {
        if self.instances.is_empty() {
            // Nothing to render when there's no instances
            return;
        }

        // Get the instances
        let (instances_bytes, instances_len) = {
            // Construct the bytes of the instances to upload
            (self.instances.bytes(), self.instances.len())
        };

        // Resize the buffer if needed
        let instance_buffer_already_pushed =
            if instances_bytes.len() as u64 > self.instance_buffer.size() {
                // We have more instances than the buffer size, recreate the buffer
                self.instance_buffer.destroy();
                self.instance_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Instance Buffer"),
                            contents: instances_bytes,
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        });

                true
            } else {
                false
            };

        {
            // Start the render pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: custom_view.unwrap_or(&self.downscale.texture_view),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.background_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Set our pipeline
            render_pass.set_pipeline(&self.render_pipeline);

            // Bind the atlas texture
            render_pass.set_bind_group(0, &self.atlas.bind_group, &[]);
            // Bind the atlas texture info
            render_pass.set_bind_group(1, &self.atlas.rects.bind_group, &[]);

            // Bind the screen size
            render_pass.set_bind_group(2, &self.screen_info.bind_group, &[]);

            // Set the target vertices
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // Set the instances
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            // Set the target indices
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Draw the instances
            render_pass.draw_indexed(0..6, 0, 0..instances_len as u32);
        }

        // Upload the instance buffer
        if !instance_buffer_already_pushed {
            self.queue
                .write_buffer(&self.instance_buffer, 0, instances_bytes);
        }

        // Clear the instances to write a new frame
        self.instances.clear();
    }
}

/// Convert an `u32` color to a WGPU [`wgpu::Color`] taking in account sRGB.
fn rgba_to_wgpu_color(color: RGBA8) -> wgpu::Color {
    let r = color.r as f64 / 255.0;
    let g = color.g as f64 / 255.0;
    let b = color.b as f64 / 255.0;
    let a = color.a as f64 / 255.0;

    if PREFERRED_TEXTURE_FORMAT.is_srgb() {
        // Convert to sRGB space
        let a = a.powf(2.2);
        let r = r.powf(2.2);
        let g = g.powf(2.2);
        let b = b.powf(2.2);

        wgpu::Color { r, g, b, a }
    } else {
        wgpu::Color { r, g, b, a }
    }
}
