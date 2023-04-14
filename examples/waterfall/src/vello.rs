mod sonar_data;

use sonar_data::{SonarDataBuffer, SonarRenderer};

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

fn run(
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

    let dimensions: (u32, u32) = (padded_len as u32, 2048);
    let texture_dimensions: (u32, u32) = (padded_len as u32, 256);

    let tile_max = (row_max / 256) as i32;

    let mut port_data_buffer = SonarDataBuffer::new(port_data, dimensions);
    let mut starboard_data_buffer = SonarDataBuffer::new(starboard_data, dimensions);

    let mut sonar_renderer = None::<SonarRenderer>;

    event_loop.run(move |event, _event_loop, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } => {
            let Some(render_state) = &mut render_state else { return };
            if render_state.window.id() != window_id {
                return;
            }
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
                    render_cx.resize_surface(&mut render_state.surface, size.width, size.height);
                    render_state.window.request_redraw();
                }
                _ => {}
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

            // Build the vello Scene that we want to display over the sonar data
            let mut builder = SceneBuilder::for_fragment(&mut fragment);

            // This is one of the vello test_scenes
            use vello::kurbo::PathEl::{self, *};
            let missing_movetos = [
                LineTo((100.0, 100.0).into()),
                LineTo((100.0, 200.0).into()),
                ClosePath,
                LineTo((0.0, 400.0).into()),
                LineTo((100.0, 400.0).into()),
            ];
            let only_movetos = [MoveTo((0.0, 0.0).into()), MoveTo((100.0, 100.0).into())];
            let empty: [PathEl; 0] = [];
            builder.fill(
                Fill::NonZero,
                Affine::translate((100.0, 100.0)),
                Color::rgb8(0, 0, 255),
                None,
                &missing_movetos,
            );
            builder.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                Color::rgb8(0, 0, 255),
                None,
                &empty,
            );
            builder.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                Color::rgb8(0, 0, 255),
                None,
                &only_movetos,
            );
            builder.stroke(
                &Stroke::new(8.0),
                Affine::translate((100.0, 100.0)),
                Color::rgb8(0, 255, 255),
                None,
                &missing_movetos,
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
                        &device_handle.device,
                        &device_handle.queue,
                        &surface_texture,
                        &port_data_buffer,
                        &starboard_data_buffer,
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
                    port_data_buffer.initialize(device, queue);
                    starboard_data_buffer.initialize(device, queue);

                    // And initialize the sonar renderer
                    sonar_renderer.get_or_insert(SonarRenderer::new(
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

// This is taken from the vello/with_winit example, but with everything not essential stripped out
// that includes hot reloading, WASM support, Android support, etc.
pub fn main(port_data: Vec<f32>, starboard_data: Vec<f32>, padded_len: usize, row_max: usize) {
    env_logger::init();

    // The event loop comes from winit
    let event_loop = EventLoop::new();

    // The render context comes from vello::util::RenderContext
    // It basically wraps all the necessary wgpu rendering state
    let render_cx = RenderContext::new().unwrap();

    // This is our run function (see above)
    run(
        event_loop,
        render_cx,
        port_data,
        starboard_data,
        padded_len,
        row_max,
    );
}
