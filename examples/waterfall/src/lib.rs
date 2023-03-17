use winit::{
    event::*,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub mod compute;
pub mod context;
pub mod texture;

struct SonarDataBuffer {
    data: Vec<f32>,
    buffer: wgpu::Buffer,
    dimensions: (u32, u32),
}

impl SonarDataBuffer {
    fn new(context: &context::Context, data: Vec<f32>, dimensions: (u32, u32)) -> Self {
        let buffer_size = (dimensions.0 as usize
            * dimensions.1 as usize
            * std::mem::size_of::<f32>()) as wgpu::BufferAddress;

        let buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sonar data buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        SonarDataBuffer {
            data,
            buffer,
            dimensions,
        }
    }

    fn copy_buffer_to_texture(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        texture: &texture::Texture,
    ) {
        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &self.buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * texture.dimensions().0),
                    rows_per_image: std::num::NonZeroU32::new(texture.dimensions().1),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: texture.dimensions().0,
                height: texture.dimensions().1,
                depth_or_array_layers: 8,
            },
        );
    }

    fn update_buffer_from_idx(&mut self, context: &context::Context, idx: usize) {
        let dims = self.dimensions;
        let new_data =
            &self.data[(idx * dims.0 as usize)..((idx + dims.1 as usize) * dims.0 as usize)];

        context
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(new_data));
    }

    fn update_buffer_from_tile(&mut self, context: &context::Context, tile: usize) {
        let tile_length = 256 * self.dimensions.0 as usize;
        let tile_start = tile * tile_length;
        let tile_end = (tile + 1) * tile_length;
        let tile_data = &self.data[tile_start..tile_end];

        let buffer_offset = (tile + 2) % 8 * tile_length * 4;

        context.queue.write_buffer(
            &self.buffer,
            buffer_offset as u64,
            bytemuck::cast_slice(tile_data),
        );
    }

    fn copy_tile_to_texture(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        texture: &texture::Texture,
        tile: u32,
    ) {
        let tile_length = 256 * self.dimensions.0;
        let array_level = (tile + 2) % 8;
        let buffer_offset = array_level * tile_length * 4;

        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &self.buffer,
                layout: wgpu::ImageDataLayout {
                    offset: buffer_offset.into(),
                    bytes_per_row: std::num::NonZeroU32::new(4 * texture.dimensions().0),
                    rows_per_image: std::num::NonZeroU32::new(texture.dimensions().1),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: array_level,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: texture.dimensions().0,
                height: texture.dimensions().1,
                depth_or_array_layers: 1,
            },
        );
    }
}

struct State {
    context: context::Context,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    port_texture: texture::Texture,
    port_bind_group: wgpu::BindGroup,
    starboard_texture: texture::Texture,
    starboard_bind_group: wgpu::BindGroup,
    idx: usize,
    port_data_buffer: SonarDataBuffer,
    starboard_data_buffer: SonarDataBuffer,
    row_max: usize,
    reduce_shader: compute::ComputeShader,
    num_blocks: u32,
    reduce_output_buffer: wgpu::Buffer,
    reduce_block_sums_buffer: wgpu::Buffer,
    reduce_bind_group: wgpu::BindGroup,
    block_increment_shader: compute::ComputeShader,
    block_increment_bind_group: wgpu::BindGroup,
    downsweep_shader: compute::ComputeShader,
    downsweep_output_buffer: wgpu::Buffer,
    downsweep_bind_group: wgpu::BindGroup,
}

impl State {
    async fn new(
        window: Window,
        port_data: Vec<f32>,
        starboard_data: Vec<f32>,
        padded_len: usize,
        row_max: usize,
    ) -> Self {
        let context = context::Context::new(window).await;

        let size = context.window().inner_size();

        let surface_caps = context.get_surface_capabilities();

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        context.surface.configure(&context.device, &config);

        let dimensions: (u32, u32) = (padded_len as u32, 2048);

        // Create data buffers
        let port_data_buffer = SonarDataBuffer::new(&context, port_data, dimensions);
        let starboard_data_buffer = SonarDataBuffer::new(&context, starboard_data, dimensions);

        let texture_dimensions: (u32, u32) = (padded_len as u32, 256);

        // Create texture
        let port_texture =
            texture::Texture::new(&context, texture_dimensions, 8, Some("Port texture"));
        let starboard_texture =
            texture::Texture::new(&context, texture_dimensions, 8, Some("Starboard texture"));

        let texture_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("Texture bind group layout"),
                });

        let port_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&port_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&port_texture.sampler),
                    },
                ],
                label: Some("Port bind group"),
            });

        let starboard_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&starboard_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&starboard_texture.sampler),
                    },
                ],
                label: Some("Starboard bind group"),
            });

        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
            });

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render pipeline layout"),
                    bind_group_layouts: &[&texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });

        // Create the reduce shader
        let reduce_shader =
            compute::ComputeShader::new(&context, include_str!("shaders/reduce.wgsl"));

        // Number of blocks
        let num_blocks = dimensions.0 / 256;

        let output_buffer_size = ((dimensions.0 * dimensions.1) as usize
            * std::mem::size_of::<f32>()) as wgpu::BufferAddress;
        let block_sum_size = ((num_blocks * dimensions.1) as usize * std::mem::size_of::<f32>())
            as wgpu::BufferAddress;

        let reduce_output_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Reduce output buffer"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let reduce_block_sums_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Reduce block sums buffer"),
            size: block_sum_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let reduce_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &reduce_shader.bind_group_layout(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: port_data_buffer.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: reduce_output_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: reduce_block_sums_buffer.as_entire_binding(),
                    },
                ],
            });

        // Create the block increment shader
        let block_increment_shader =
            compute::ComputeShader::new(&context, include_str!("shaders/block_increment.wgsl"));

        let block_increment_bind_group =
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &block_increment_shader.bind_group_layout(),
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: reduce_block_sums_buffer.as_entire_binding(),
                    }],
                });

        // Create the downsweep shader
        let downsweep_shader =
            compute::ComputeShader::new(&context, include_str!("shaders/downsweep.wgsl"));

        let downsweep_output_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Downsweep output buffer"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let downsweep_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &downsweep_shader.bind_group_layout(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: reduce_output_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: downsweep_output_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: reduce_block_sums_buffer.as_entire_binding(),
                    },
                ],
            });

        Self {
            context,
            config,
            size,
            render_pipeline,
            port_texture,
            port_bind_group,
            starboard_texture,
            starboard_bind_group,
            idx: 0,
            port_data_buffer,
            starboard_data_buffer,
            row_max,
            reduce_shader,
            num_blocks,
            reduce_output_buffer,
            reduce_block_sums_buffer,
            reduce_bind_group,
            block_increment_shader,
            block_increment_bind_group,
            downsweep_shader,
            downsweep_output_buffer,
            downsweep_bind_group,
        }
    }

    pub fn window(&self) -> &Window {
        &self.context.window()
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.context
                .surface
                .configure(&self.context.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Up),
                        ..
                    },
                ..
            } => {
                if self.idx < self.row_max - 1024 - 10 {
                    self.idx += 10;
                }
                true
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Down),
                        ..
                    },
                ..
            } => {
                if self.idx > 0 {
                    self.idx -= 10;
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {
        self.port_data_buffer
            .update_buffer_from_idx(&self.context, self.idx);
        self.starboard_data_buffer
            .update_buffer_from_idx(&self.context, self.idx);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.context.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render encoder"),
                });

        self.port_data_buffer
            .copy_tile_to_texture(&mut encoder, &self.port_texture, 0);
        self.starboard_data_buffer
            .copy_tile_to_texture(&mut encoder, &self.starboard_texture, 0);

        // Run compute shaders here

        /*
        // Reduce the sonar data
            self.reduce_shader.dispatch(
                &self.reduce_bind_group,
                &mut encoder,
                (self.num_blocks, self.port_data_buffer.dimensions.1, 1),
            );

            // Reduce the block sums
            self.block_increment_shader.dispatch(
                &self.block_increment_bind_group,
                &mut encoder,
                (1, self.port_data_buffer.dimensions.1, 1),
            );

            // Downsweep
            self.downsweep_shader.dispatch(
                &self.downsweep_bind_group,
                &mut encoder,
                (self.num_blocks, self.port_data_buffer.dimensions.1, 1),
            );

            // Copy the downsweep output buffer into the port texture for testing

            encoder.copy_buffer_to_texture(
                wgpu::ImageCopyBuffer {
                    buffer: &self.downsweep_output_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: std::num::NonZeroU32::new(4 * self.port_texture.dimensions().0),
                        rows_per_image: std::num::NonZeroU32::new(self.port_texture.dimensions().1),
                    },
                },
                wgpu::ImageCopyTexture {
                    texture: &self.port_texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: self.port_texture.dimensions().0,
                    height: self.port_texture.dimensions().1,
                    depth_or_array_layers: 1,
                },
            );
         */
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::default()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);

            // Draw and texture the port quad
            render_pass.set_bind_group(0, &self.port_bind_group, &[]);
            render_pass.draw(0..6, 1..2);

            // Draw and texture the starboard quad
            render_pass.set_bind_group(0, &self.starboard_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run(port_data: Vec<f32>, starboard_data: Vec<f32>, padded_len: usize, row_max: usize) {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("SDW Waterfall Viewer")
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(window, port_data, starboard_data, padded_len, row_max).await;

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.context.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.set_exit(),
                        WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size)
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.context.window().id() => {
                let start_time = std::time::Instant::now();
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => control_flow.set_exit(),
                    Err(e) => eprintln!("{:?}", e),
                }
                log::debug!("Rendering took {:?}", start_time.elapsed())
            }
            Event::MainEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        }
    })
}
