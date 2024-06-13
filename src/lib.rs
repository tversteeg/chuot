#![forbid(unsafe_code)]

use std::{borrow::Cow, rc::Rc, sync::Arc};

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowAttributes, WindowId},
};

pub struct Context {}

pub struct EmbeddedAssets {}

pub struct GameConfig {}

#[derive(Debug)]
struct Setup {
    event_loop_proxy: Option<EventLoopProxy<App>>,
    width: u32,
    height: u32,
}

#[derive(Debug)]
enum MaybeApp {
    Setup(Setup),
    App(App),
}

impl ApplicationHandler<App> for MaybeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Only resume when setting up
        let Self::Setup(setup) = self else {
            return;
        };

        // Handle the event loop proxy once
        let Some(event_loop_proxy) = setup.event_loop_proxy.take() else {
            return;
        };

        // Create the window with the graphics
        let app = pollster::block_on(App::new(event_loop, setup.width, setup.height));

        // Send the app as a user event
        event_loop_proxy.send_event(app).unwrap();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Self::App(app) = self else {
            return;
        };

        match event {
            WindowEvent::Resized(size) => {
                println!("Resize: {size:?}");
            }
            WindowEvent::RedrawRequested => app.draw(),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Self::App(app) = self else {
            return;
        };

        app.window.request_redraw();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, app: App) {
        *self = Self::App(app);
    }
}

#[derive(Debug)]
struct App {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    queue: wgpu::Queue,
    device: wgpu::Device,
    render_pipeline: wgpu::RenderPipeline,
}

impl App {
    async fn new(event_loop: &ActiveEventLoop, width: u32, height: u32) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_min_inner_size(LogicalSize::new(width, height))
                        .with_inner_size(LogicalSize::new(width, height)),
                )
                .unwrap(),
        );

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_config = surface.get_default_config(&adapter, width, height).unwrap();

        surface.configure(&device, &surface_config);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed("
                    @vertex
                    fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
                        let x = f32(i32(in_vertex_index) - 1);
                        let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
                        return vec4<f32>(x, y, 0.0, 1.0);
                    }

                    @fragment
                    fn fs_main() -> @location(0) vec4<f32> {
                        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
                    }
            "))
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        });

        Self {
            window,
            surface,
            queue,
            device,
            render_pipeline,
        }
    }

    fn draw(&mut self) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..3, 0..1);
        }

        let command_buffer = encoder.finish();
        self.queue.submit([command_buffer]);

        frame.present();
    }
}

pub trait PixelGame: Sized
where
    Self: 'static,
{
    fn update(&mut self, ctx: Context);

    fn render(&mut self, ctx: Context);

    #[inline]
    fn run(self, assets: EmbeddedAssets, game_config: GameConfig) {
        let event_loop = EventLoop::with_user_event().build().unwrap();
        let mut app = MaybeApp::Setup(Setup {
            event_loop_proxy: Some(event_loop.create_proxy()),
            width: 320,
            height: 240,
        });
        event_loop.run_app(&mut app).unwrap();
    }
}
