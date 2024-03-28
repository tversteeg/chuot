//! Show a in-game profiler window.
//!
//! Window is based on Egui.

use egui::{Align::Min, Layout, Ui, Window as EguiWindow};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::{
    egui::{FullOutput, ViewportId},
    State,
};
use glamour::Size2;
use puffin_egui::egui::Context;
use wgpu_profiler::{GpuProfiler, GpuProfilerSettings, GpuTimerQueryResult};
use winit::{event::WindowEvent, window::Window};

use crate::graphics::state::PREFERRED_TEXTURE_FORMAT;

/// State for showing the in-game profiler.
pub(crate) struct InGameProfiler {
    /// Egui WGPU renderer.
    renderer: Renderer,
    /// Egui winit state.
    state: State,
    /// GPU profiler.
    pub(crate) gpu_profiler: GpuProfiler,
}

impl InGameProfiler {
    /// Creates a new render routine to render the in-game profiler.
    pub(super) fn new<W>(device: &wgpu::Device, window: W) -> Self
    where
        W: wgpu::WindowHandle,
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

        // Setup the GPU profiler
        let gpu_profiler = GpuProfiler::new(GpuProfilerSettings::default())
            .expect("Error setting up GPU profiler");

        Self {
            renderer,
            state,
            gpu_profiler,
        }
    }

    /// Render the window.
    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_size: Size2<u32>,
    ) {
        profiling::scope!("Render profiling window");

        // End the frame for the GPU profiler
        self.gpu_profiler
            .end_frame()
            .expect("Error ending GPU profiler frame");

        // Get egui input
        let input = self.state.take_egui_input(window);

        // Render egui frame
        let FullOutput {
            shapes,
            textures_delta,
            pixels_per_point,
            ..
        } = self.state.egui_ctx().run(input, |ctx| {
            // Show a GUI window for the CPU & GPU profilers
            if let Some(query_results) = self
                .gpu_profiler
                .process_finished_frame(queue.get_timestamp_period())
            {
                EguiWindow::new("GPU & CPU Profilers").show(ctx, |ui| {
                    // GPU profiler
                    gpu_profiler_window(ui, &query_results);

                    // CPU profiler
                    puffin_egui::profiler_ui(ui);
                });
            }
        });

        for id in textures_delta.free {
            self.renderer.free_texture(&id);
        }

        for (id, image_delta) in textures_delta.set {
            self.renderer
                .update_texture(&device, &queue, id, &image_delta);
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [screen_size.width, screen_size.height],
            pixels_per_point,
        };

        let paint_jobs = self.state.egui_ctx().tessellate(shapes, pixels_per_point);

        self.renderer
            .update_buffers(&device, &queue, encoder, &paint_jobs, &screen_descriptor);

        // Start a new render pass for the egui window
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Render the egui window
        self.renderer
            .render(&mut render_pass, &paint_jobs, &screen_descriptor);

        // Ignore the egui rendering by pretending another frame got drawn
        self.gpu_profiler
            .end_frame()
            .expect("Error ending GPU profiler frame");
    }

    /// Handle a winit event.
    pub(super) fn handle_window_event(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }
}

/// Draw the GPU profiler window.
fn gpu_profiler_window(ui: &mut Ui, query_results: &[GpuTimerQueryResult]) {
    for query_result in query_results {
        ui.vertical(|ui| {
            // Draw the timing results in a column
            ui.columns(2, |columns| {
                columns[0].label(&query_result.label);
                columns[1].with_layout(Layout::right_to_left(Min), |ui| {
                    let time = (query_result.time.end - query_result.time.start) * 1000.0 * 1000.0;

                    ui.monospace(format!("{time:.3} µs"));
                });
            });
        });
    }
}
