pub mod app;
pub mod render;
pub mod sonar_data;

use app::App;

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
            let mut builder = SceneBuilder::for_fragment(&mut fragment);

            let starboard_plot_transform = Affine::map_unit_square(vello::kurbo::Rect::new(
                widthf64 / 2.0,
                heightf64,
                widthf64,
                3.0 * heightf64 / 4.0,
            ));

            builder.fill(
                Fill::NonZero,
                starboard_plot_transform,
                Color::rgb8(255, 255, 255),
                None,
                &vello::kurbo::Rect::new(0.0, 0.0, 1.0, 1.0),
            );

            let port_plot_transform = Affine::map_unit_square(vello::kurbo::Rect::new(
                widthf64 / 2.0,
                heightf64,
                0.0,
                3.0 * heightf64 / 4.0,
            ));

            builder.fill(
                Fill::NonZero,
                port_plot_transform,
                Color::rgb8(255, 255, 255),
                None,
                &vello::kurbo::Rect::new(0.0, 0.0, 1.0, 1.0),
            );

            let (starboard_ping_data, port_ping_data) = app.plot_pings();

            let ping_max = starboard_ping_data
                .iter()
                .fold(0.0f32, |acc, &y| acc.max(y));
            let ping_max = port_ping_data.iter().fold(ping_max, |acc, &y| acc.max(y));

            let data_len = starboard_ping_data.len() as f64;

            let starboard_ping_plot: vello::kurbo::BezPath = starboard_ping_data
                .iter()
                .enumerate()
                .map(|(i, &y)| {
                    let x = (i as f64) / data_len;
                    vello::kurbo::PathEl::LineTo((x, f64::from(y / ping_max)).into())
                })
                .collect();

            let port_ping_plot: vello::kurbo::BezPath = port_ping_data
                .iter()
                .enumerate()
                .map(|(i, &y)| {
                    let x = (i as f64) / data_len;
                    vello::kurbo::PathEl::LineTo((x, f64::from(y / ping_max)).into())
                })
                .collect();

            builder.stroke(
                &Stroke::new(0.001),
                starboard_plot_transform,
                Color::rgb8(0, 0, 0),
                None,
                &starboard_ping_plot,
            );

            builder.stroke(
                &Stroke::new(0.001),
                port_plot_transform,
                Color::rgb8(0, 0, 0),
                None,
                &port_ping_plot,
            );

            // Render the vello scene to a texture
            let render_params = vello::RenderParams {
                base_color: Color::TRANSPARENT,
                width,
                height,
            };
            let mut builder = SceneBuilder::for_scene(&mut scene);
            builder.append(&fragment, Some(transform));

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
