#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(target_arch = "wasm32")]
mod web;

use wgpu::{
    Backends, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features,
    FragmentState, Instance, InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, PowerPreference::HighPerformance, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptionsBase, SamplerBindingType, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, StoreOp, Surface, SurfaceConfiguration, TextureSampleType, TextureUsages,
    TextureViewDescriptor, TextureViewDimension, VertexState, WindowHandle,
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

use crate::{
    canvas::Canvas,
    graphics::{
        render::{Render, RenderState},
        texture::Texture,
        MainRenderState,
    },
    sprite::Sprite,
};

/// Update function signature.
pub trait UpdateFn<G>: FnMut(&mut G, &WinitInputHelper, Option<Vec2<usize>>, f64) -> bool {}

impl<G, T: FnMut(&mut G, &WinitInputHelper, Option<Vec2<usize>>, f64) -> bool> UpdateFn<G> for T {}

/// Render function signature.
pub trait RenderFn<G>: FnMut(&mut G) -> Vec<Sprite> {}

impl<G, T: FnMut(&mut G) -> Vec<Sprite>> RenderFn<G> for T {}

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
    U: UpdateFn<G> + 'static,
    R: RenderFn<G> + 'static,
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
    U: UpdateFn<G> + 'static,
    R: RenderFn<G> + 'static,
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
                (updates_per_second as f64).recip(),
            ) {
                g.exit();
            }
        },
        move |g| {
            // Get the items to be rendered for the game frame
            let sprites = render(&mut g.game.game_state);

            // Render everything
            g.game.render_state.render(sprites);

            // Second render pass
            /*
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::GREEN),
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                rpass.set_pipeline(render_pipeline);
                rpass.draw(0..3, 0..1);
            }
                */
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
                    // Resize GPU surface
                    g.game
                        .render_state
                        .resize(Extent2::new(new_size.width, new_size.height));

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
    /// Hold the global GPU references and information.
    render_state: MainRenderState<'window>,
}

impl<'window, G> State<'window, G> {
    /// Setup the state including the GPU part.
    async fn new<W>(buffer_size: Extent2<u32>, window: W, game_state: G) -> Result<Self>
    where
        W: WindowHandle + 'window,
    {
        // Setup the winit input helper state
        let input = WinitInputHelper::new();

        // Create a surface on the window and setup the render state to it
        let render_state = MainRenderState::new(buffer_size, window)
            .await
            .wrap_err("Error setting up the rendering pipeline")?;

        Ok(Self {
            game_state,
            input,
            render_state,
        })
    }
}
