use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Viewport {
    viewport: [f32; 2],
}

impl Viewport {
    pub fn new() -> Self {
        Viewport {
            viewport: [0.0, 0.0],
        }
    }
}

pub struct SonarDataBuffer {
    data: Vec<f32>,
    buffer: Option<wgpu::Buffer>,
    dimensions: (u32, u32),
}

impl SonarDataBuffer {
    pub fn new(data: Vec<f32>, dimensions: (u32, u32)) -> Self {
        SonarDataBuffer {
            data,
            buffer: None,
            dimensions,
        }
    }

    pub fn initialize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let buffer_size = (self.dimensions.0 as usize
            * self.dimensions.1 as usize
            * std::mem::size_of::<f32>()) as wgpu::BufferAddress;

        self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sonar data buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));

        self.update_buffer_from_tile(queue, 0);
        self.update_buffer_from_tile(queue, 1);
        self.update_buffer_from_tile(queue, 2);
        self.update_buffer_from_tile(queue, 3);
        self.update_buffer_from_tile(queue, 4);
        self.update_buffer_from_tile(queue, 5);
    }

    fn copy_buffer_to_texture(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::Texture,
        dimensions: (u32, u32), // These are the texture dimensions
        layers: u32,            // This is the texture layer count
    ) {
        if let Some(buffer) = &self.buffer {
            encoder.copy_buffer_to_texture(
                wgpu::ImageCopyBuffer {
                    buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                        rows_per_image: std::num::NonZeroU32::new(dimensions.1),
                    },
                },
                wgpu::ImageCopyTexture {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: dimensions.0,
                    height: dimensions.1,
                    depth_or_array_layers: layers,
                },
            );
        }
    }

    fn update_buffer_from_idx(&mut self, queue: &wgpu::Queue, idx: usize) {
        let dims = self.dimensions;
        let new_data =
            &self.data[(idx * dims.0 as usize)..((idx + dims.1 as usize) * dims.0 as usize)];

        if let Some(buffer) = &self.buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(new_data));
        }
    }

    fn update_buffer_from_tile(&mut self, queue: &wgpu::Queue, tile: usize) {
        let tile_length = 256 * self.dimensions.0 as usize;
        let tile_start = tile * tile_length;
        let tile_end = (tile + 1) * tile_length;
        let tile_data = &self.data[tile_start..tile_end];

        let buffer_offset = (tile + 2) % 8 * tile_length * 4;

        if let Some(buffer) = &self.buffer {
            queue.write_buffer(
                buffer,
                buffer_offset as u64,
                bytemuck::cast_slice(tile_data),
            );
        }
    }
}

pub struct TargetTexture {
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

impl TargetTexture {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            format: wgpu::TextureFormat::Rgba8Unorm,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            view,
            width,
            height,
        }
    }
}

pub struct SonarRenderer {
    render_pipeline: wgpu::RenderPipeline,
    texture_dimensions: (u32, u32),
    texture_layers: u32,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    vello_texture: TargetTexture,
    port_texture: wgpu::Texture,
    starboard_texture: wgpu::Texture,
    viewport_buffer: wgpu::Buffer,
    viewport_bind_group_layout: wgpu::BindGroupLayout,
}

impl SonarRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: &wgpu::TextureFormat,
        dimensions: (u32, u32),
        layers: u32,
        width: u32,
        height: u32,
    ) -> Self {
        let vello_texture = TargetTexture::new(device, width, height);

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: layers,
        };

        let port_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Port texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let starboard_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Port texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

	let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
                label: Some("Texture bind group layout"),
            });

        let viewport = Viewport::new();
        let viewport_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Viewport uniform buffer"),
            contents: bytemuck::cast_slice(&[viewport]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let viewport_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Viewport bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &viewport_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    format: *surface_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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

        Self {
            render_pipeline,
            texture_dimensions: dimensions,
            texture_layers: layers,
            texture_bind_group_layout,
	    sampler,
            vello_texture,
            port_texture,
            starboard_texture,
            viewport_buffer,
            viewport_bind_group_layout,
        }
    }

    pub fn resize_vello_texture(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.vello_texture.width != width || self.vello_texture.height != height {
            log::debug!("Resizing to {width}, {height}");
            self.vello_texture = TargetTexture::new(device, width, height);
        }
    }

    pub fn vello_texture_view(&self) -> &wgpu::TextureView {
        &self.vello_texture.view
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output: &wgpu::SurfaceTexture,
        port_data: &SonarDataBuffer,
        starboard_data: &SonarDataBuffer,
    ) -> Result<(), wgpu::SurfaceError> {
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let port_view = self.port_texture.create_view(&wgpu::TextureViewDescriptor::default());
	let starboard_view = self.starboard_texture.create_view(&wgpu::TextureViewDescriptor::default());
	
        let port_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&port_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.vello_texture.view),
                },
            ],
            label: Some("Port bind group"),
        });

	let starboard_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&starboard_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.vello_texture.view),
                },
            ],
            label: Some("Starboard bind group"),
        });

        let viewport_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Viewport bind group"),
            layout: &self.viewport_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.viewport_buffer.as_entire_binding(),
            }],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render encoder"),
        });

        port_data.copy_buffer_to_texture(
            &mut encoder,
            &self.port_texture,
            self.texture_dimensions,
            self.texture_layers,
        );
        starboard_data.copy_buffer_to_texture(
            &mut encoder,
            &self.starboard_texture,
            self.texture_dimensions,
            self.texture_layers,
        );

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

            // Draw and texture the starboard quad
            render_pass.set_bind_group(0, &starboard_bind_group, &[]);
            render_pass.set_bind_group(1, &viewport_bind_group, &[]);
            render_pass.draw(0..6, 0..1);

            // Draw and texture the port quad and the vello texture
            render_pass.set_bind_group(0, &port_bind_group, &[]);
            render_pass.set_bind_group(1, &viewport_bind_group, &[]);
            render_pass.draw(0..6, 1..3);
        }

        queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
