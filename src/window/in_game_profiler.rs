//! Show a in-game profiler window.
//!
//! Window is based on Egui.

use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::{
    egui::{FullOutput, ViewportId},
    State,
};
use game_loop::winit::{event::WindowEvent, window::Window};
use puffin_egui::egui::Context;
use vek::Extent2;
use wgpu::{
    CommandEncoder, Device, LoadOp, Operations, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureView, WindowHandle,
};

use crate::graphics::state::PREFERRED_TEXTURE_FORMAT;

/// State for showing the in-game profiler.
pub(crate) struct InGameProfiler {
    /// Egui WGPU renderer.
    renderer: Renderer,
    /// Egui winit state.
    state: State,
}

impl InGameProfiler {
    /// Creates a new render routine to render the in-game profiler.
    pub(super) fn new<W>(device: &Device, window: W) -> Self
    where
        W: WindowHandle,
    {
        let renderer = Renderer::new(device, PREFERRED_TEXTURE_FORMAT, None, 1);
        let state = State::new(
            Context::default(),
            ViewportId::default(),
            &window,
            None,
            None,
        );

        // Enable the profiler
        puffin::set_scopes_on(true);

        Self { renderer, state }
    }

    /// Render the window.
    pub(crate) fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        screen_size: Extent2<u32>,
        window: &Window,
    ) {
        profiling::scope!("Render profiling window");

        // Get egui input
        let input = self.state.take_egui_input(window);

        // Render egui frame
        let FullOutput {
            shapes,
            textures_delta,
            pixels_per_point,
            ..
        } = self.state.egui_ctx().run(input, |ctx| {
            // Draw the profiler window to the context if profiling is enabled
            puffin_egui::show_viewport_if_enabled(ctx);
        });

        for id in textures_delta.free {
            self.renderer.free_texture(&id);
        }

        for (id, image_delta) in textures_delta.set {
            self.renderer
                .update_texture(device, queue, id, &image_delta);
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: screen_size.into_array(),
            pixels_per_point,
        };

        let paint_jobs = self.state.egui_ctx().tessellate(shapes, pixels_per_point);

        self.renderer
            .update_buffers(device, queue, encoder, &paint_jobs, &screen_descriptor);

        // Start a new render pass for the egui window
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.renderer
            .render(&mut render_pass, &paint_jobs, &screen_descriptor);
    }

    /// Handle a winit event.
    pub(super) fn handle_window_event(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }
}
