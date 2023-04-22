pub mod app;
pub mod render;
pub mod sonar_data;
pub mod view;

use app::App;
use view::View;

use winit::{
    event::*,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use vello::{
    kurbo::Affine,
    peniko::{Color, Fill, Stroke},
    util::{RenderContext, RenderSurface},
    Renderer, RendererOptions, Scene, SceneBuilder, SceneFragment,
};

struct RenderState {
    surface: RenderSurface,
    window: Window,
}

pub fn run(
    event_loop: EventLoop<()>,
    render_cx: RenderContext,
    port_data: Vec<f32>,
    starboard_data: Vec<f32>,
    padded_len: usize,
    row_max: usize,
) {
    let mut renderers: Vec<Option<Renderer>> = vec![];
    let mut render_cx = render_cx;
    // When does render_state get initialized?
    // Upon Event::Resumed
    let mut render_state = None::<RenderState>;

    let mut cached_window = None;

    let mut scene = Scene::new();
    let mut fragment = SceneFragment::new();

    let transform = Affine::IDENTITY;

    let mut app = App::new(port_data, starboard_data, padded_len, row_max);

    let texture_dimensions: (u32, u32) = (padded_len as u32, 256);

    let mut sonar_renderer = None::<render::Renderer>;

    event_loop.run(move |event, _event_loop, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } => {
            let Some(render_state) = &mut render_state else { return };
            if render_state.window.id() != window_id {
                return;
            }
            if !app.input(event) {
                match event {
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.state == ElementState::Pressed {
                            match input.virtual_keycode {
                                Some(VirtualKeyCode::Escape) => control_flow.set_exit(),
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::Resized(size) => {
                        render_cx.resize_surface(
                            &mut render_state.surface,
                            size.width,
                            size.height,
                        );
                        render_state.window.request_redraw();
                    }
                    _ => {}
                }
            }
        }
        Event::MainEventsCleared => {
            if let Some(render_state) = &mut render_state {
                render_state.window.request_redraw();
            }
        }
        Event::RedrawRequested(_) => {
            // This is where we need to render everything
            let Some(render_state) = &mut render_state else { return };
            let width = render_state.surface.config.width;
            let height = render_state.surface.config.height;
            let device_handle = &render_cx.devices[render_state.surface.dev_id];

            let widthf64 = f64::from(width);
            let heightf64 = f64::from(height);

            // Should we update this every frame?
            app.update(&device_handle.queue);

            // Build the vello Scene that we want to display over the sonar data

            let builder = SceneBuilder::for_scene(&mut scene);
            let mut cx = view::RenderContext::new(builder);

            // Plot idx indicator
            let idx_plot = app.plot_idx();

            let scroll_bar = view::ScrollBar::new(
                idx_plot,
                Color::rgb8(0, 0, 0),
                Color::rgb8(200, 200, 200),
                view::Point::new(widthf64 - 20.0, 0.0),
                view::Size::new(20.0, 3.0 * heightf64 / 4.0),
                row_max,
            );

            scroll_bar.draw(&view::Point::new(widthf64 - 20.0, 0.0), &mut cx);

            // Plot pings
            let (starboard_ping_data, port_ping_data) = app.plot_pings();

            let ping_plot = view::PingPlot::new(
                starboard_ping_data,
                port_ping_data,
                Color::rgb8(255, 255, 255),
                Color::rgb8(0, 0, 0),
                view::Point::new(0.0, 3.0 * heightf64 / 4.0),
                view::Size::new(widthf64, heightf64 / 4.0),
            );

            ping_plot.draw(&view::Point::new(0.0, 3.0 * heightf64 / 4.0), &mut cx);

            let test_box = view::Box::new(Color::RED, view::Size::new(100.0, 100.0));
            let mut container = view::Container::new(
                test_box,
                Color::GREEN,
                view::Size::new(10.0, 10.0),
                view::Size::new(200.0, 200.0),
            );

            let screen_size = view::Size::new(widthf64, heightf64);
            let zero_size = view::Size::new(0.0, 0.0);
            container.layout(&zero_size, &screen_size);
            container.draw(&view::Point::new(widthf64 / 2.0, heightf64 / 2.0), &mut cx);

            // Render the vello scene to a texture
            let render_params = vello::RenderParams {
                base_color: Color::TRANSPARENT,
                width,
                height,
            };

            let surface_texture = render_state
                .surface
                .surface
                .get_current_texture()
                .expect("failed to get surface texture");

            if let Some(sonar_renderer) = &mut sonar_renderer {
                // Resize as necessary.
                sonar_renderer.resize_vello_texture(&device_handle.device, width, height);

                renderers[render_state.surface.dev_id]
                    .as_mut()
                    .unwrap()
                    .render_to_texture(
                        &device_handle.device,
                        &device_handle.queue,
                        &scene,
                        sonar_renderer.vello_texture_view(),
                        &render_params,
                    )
                    .expect("failed to render to surface");

                // Render the sonar data and the vello Scene to the surface texture
                sonar_renderer
                    .render(
                        &app,
                        &device_handle.device,
                        &device_handle.queue,
                        &surface_texture,
                    )
                    .unwrap();
            };

            surface_texture.present();
            device_handle.device.poll(wgpu::Maintain::Poll);
        }
        Event::Suspended => {
            eprintln!("Suspending");
            if let Some(render_state) = render_state.take() {
                cached_window = Some(render_state.window);
            }
            control_flow.set_wait();
        }
        Event::Resumed => {
            {
                let Option::None = render_state else { return };
                let window = cached_window.take().unwrap_or_else(|| {
                    WindowBuilder::new()
                        .with_inner_size(winit::dpi::LogicalSize::new(1044, 800))
                        .with_resizable(true)
                        .with_title("Waterfall Viewer")
                        .build(&_event_loop)
                        .unwrap()
                });
                let size = window.inner_size();
                let surface_future = render_cx.create_surface(&window, size.width, size.height);
                // We need to block here, in case a Suspended event appeared
                let surface = pollster::block_on(surface_future);
                render_state = {
                    let render_state = RenderState { window, surface };
                    renderers.resize_with(render_cx.devices.len(), || None);
                    let id = render_state.surface.dev_id;
                    renderers[id].get_or_insert_with(|| {
                        log::debug!("Creating renderer {id}");
                        log::debug!("Format {:?}", render_state.surface.format);
                        let device_features = render_cx.devices[id].device.features();
                        log::debug!("Device features: {device_features:?}");
                        Renderer::new(
                            &render_cx.devices[id].device,
                            &RendererOptions {
                                surface_format: Some(render_state.surface.format),
                            },
                        )
                        .expect("Could not create renderer")
                    });

                    // Initialize the vello texture
                    let device = &render_cx.devices[id].device;
                    let queue = &render_cx.devices[id].queue;

                    // I think we should initialize the sonar data buffers here
                    app.initialize(device, queue);

                    // And initialize the sonar renderer
                    sonar_renderer.get_or_insert(render::Renderer::new(
                        device,
                        &render_state.surface.format,
                        texture_dimensions,
                        8,
                        size.width,
                        size.height,
                    ));
                    Some(render_state)
                };

                control_flow.set_poll();
            }
        }
        _ => {}
    });
}
