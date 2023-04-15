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

    pub fn copy_buffer_to_texture(
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

    pub fn update_buffer_from_tile(&mut self, queue: &wgpu::Queue, tile: usize) {
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
