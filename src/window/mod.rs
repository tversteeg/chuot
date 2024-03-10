#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(target_arch = "wasm32")]
mod web;

use wgpu::{
    Backends, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, FragmentState,
    Instance, InstanceDescriptor, Limits, MultisampleState, PipelineLayoutDescriptor,
    PowerPreference::HighPerformance, PrimitiveState, Queue, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptionsBase, ShaderModuleDescriptor, ShaderSource,
    Surface, SurfaceConfiguration, TextureViewDescriptor, VertexState, WindowHandle,
};
/// Re-export winit types.
pub use winit::{
    dpi::PhysicalSize,
    event::MouseButton,
    keyboard::{Key, KeyCode},
};
/// Re-export winit_input_helper type.
pub use winit_input_helper::WinitInputHelper as Input;

use std::{borrow::Cow, rc::Rc, sync::Arc};

use miette::{Context, IntoDiagnostic, Result};
use vek::{Extent2, Rect, Vec2};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;

use crate::canvas::Canvas;

/// Window configuration.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Amount of pixels for the canvas.
    ///
    /// Defaults to `(320, 280)`.
    pub buffer_size: Extent2<usize>,
    /// How many times the buffer should be scaled to fit the window.
    ///
    /// Defaults to `1`.
    pub scaling: usize,
    /// Name in the title bar.
    ///
    /// On WASM this will display as a header underneath the rendered content.
    ///
    /// Defaults to `"Pixel Game"`.
    pub title: String,
    /// Updates per second for the update loop.
    ///
    /// Defaults to `60`.
    pub updates_per_second: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            buffer_size: Extent2::new(320, 280),
            scaling: 1,
            title: "Pixel Game".to_string(),
            updates_per_second: 60,
        }
    }
}

/// Manually create a new window with an event loop and run the game.
///
/// For a more integrated and easier use it's recommended to use [`crate::PixelGame`].
///
/// If the `audio` feature is enabled this will also start a new audio backend.
///
/// # Arguments
///
/// * `game_state` - Global state passed around in the render and update functions.
/// * `window_config` - Configuration options for the window.
/// * `update` - Function called every update tick, arguments are the state, window input event that can be used to handle input events, mouse position in pixels and the time between this and the previous tick. When `true` is returned the window will be closed.
/// * `render` - Function called every render tick, arguments are the state and the time between this and the previous tick.
///
/// # Errors
///
/// - When the audio manager could not find a device to play audio on.
pub fn window<G, U, R>(
    game_state: G,
    window_config: WindowConfig,
    update: U,
    render: R,
) -> Result<()>
where
    G: 'static,
    U: FnMut(&mut G, &WinitInputHelper, Option<Vec2<usize>>, f32) -> bool + 'static,
    R: FnMut(&mut G, &mut Canvas, f32) + 'static,
{
    // Build the window builder with the event loop the user supplied
    let logical_size = LogicalSize::new(
        window_config.buffer_size.w as f64,
        window_config.buffer_size.h as f64,
    );
    let window_builder = WindowBuilder::new()
        .with_title(window_config.title.clone())
        .with_inner_size(logical_size)
        .with_min_inner_size(logical_size);

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Enable environment logger for winit
        env_logger::init();

        pollster::block_on(async {
            desktop::window(window_builder, game_state, window_config, update, render).await
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        // Show panics in the browser console log
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        // Web window function is async, so we need to spawn it into a local async runtime
        wasm_bindgen_futures::spawn_local(async {
            web::window(window_builder, game_state, window_config, update, render)
                .await
                .expect("Error opening WASM window")
        });

        Ok(())
    }
}

/// Open a winit window with an event loop.
async fn winit_start<G, U, R>(
    event_loop: EventLoop<()>,
    window: Window,
    game_state: G,
    mut update: U,
    mut render: R,
    WindowConfig {
        buffer_size,
        updates_per_second,
        ..
    }: WindowConfig,
) -> Result<()>
where
    G: 'static,
    U: FnMut(&mut G, &WinitInputHelper, Option<Vec2<usize>>, f32) -> bool + 'static,
    R: FnMut(&mut G, &mut Canvas, f32) + 'static,
{
    // Setup the audio
    #[cfg(feature = "audio")]
    crate::audio::init_audio()?;

    // Wrap the window in an atomic reference counter, needed for game_loop
    let window = Arc::new(window);

    // Setup the game and GPU state
    let state = State::new(buffer_size.as_(), window.clone(), game_state).await?;

    // Start the game loop
    game_loop::game_loop(
        event_loop,
        window,
        state,
        updates_per_second,
        0.1,
        move |g| {
            // Calculate mouse in pixels
            let mouse = g.game.input.cursor().and_then(|mouse| None);

            // Call update and exit when it returns true
            if update(
                &mut g.game.game_state,
                &g.game.input,
                mouse,
                (updates_per_second as f32).recip(),
            ) {
                g.exit();
            }
        },
        move |g| {
            let frame_time = g.last_frame_time();

            let State {
                surface,
                queue,
                render_pipeline,
                device,
                ..
            } = &mut g.game;

            // Get the main render texture
            let frame = surface
                .get_current_texture()
                .expect("Error acquiring next swap chain texture");
            let view = frame.texture.create_view(&TextureViewDescriptor::default());

            let mut encoder =
                device.create_command_encoder(&CommandEncoderDescriptor { label: None });

            // First render pass
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                rpass.set_pipeline(render_pipeline);
                rpass.draw(0..3, 0..1);
            }

            // Draw to the texture
            queue.submit(Some(encoder.finish()));

            // Show the texture in the window
            frame.present();
        },
        |g, event| {
            if g.game.input.update(event) {
                // Handle close requests
                if g.game.input.close_requested() {
                    g.exit();
                    return;
                }

                // Resize pixels surface if window is resized
                if let Some(new_size) = g.game.input.window_resized() {
                    let State {
                        config,
                        surface,
                        device,
                        ..
                    } = &mut g.game;

                    // Resize GPU surface
                    config.width = new_size.width.max(1);
                    config.height = new_size.height.max(1);
                    surface.configure(device, config);

                    // On MacOS the window needs to be redrawn manually after resizing
                    g.window.request_redraw();
                }
            }
        },
    )
    .into_diagnostic()
    .wrap_err("Error running game loop")
}

/// Pass multiple fields to the game state.
struct State<'window, G> {
    /// User passed game state.
    game_state: G,
    /// Winit input helper state.
    input: WinitInputHelper,
    /// GPU surface.
    surface: Surface<'window>,
    /// GPU device.
    device: Device,
    /// GPU surface configuration.
    config: SurfaceConfiguration,
    /// GPU render pipeline.
    render_pipeline: RenderPipeline,
    /// GPU queue.
    queue: Queue,
}

impl<'window, G> State<'window, G> {
    /// Setup the state including the GPU part.
    async fn new<W>(buffer_size: Extent2<u32>, window: W, game_state: G) -> Result<Self>
    where
        W: WindowHandle + 'window,
    {
        // Setup the winit input helper state
        let input = WinitInputHelper::new();

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
                power_preference: HighPerformance,
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
                    // Use the texture resolution limits from the adapter
                    required_limits: Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .into_diagnostic()
            .wrap_err("Error getting logical GPU device for surface")?;

        // Load the shaders from disk
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("./shaders/window.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
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

        let config = surface
            .get_default_config(&adapter, buffer_size.w, buffer_size.h)
            .ok_or_else(|| miette::miette!("Error getting window surface configuration"))?;
        surface.configure(&device, &config);

        Ok(Self {
            game_state,
            input,
            surface,
            device,
            config,
            render_pipeline,
            queue,
        })
    }
}
