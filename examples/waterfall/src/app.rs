use winit::event::*;

use crate::sonar_data::SonarDataBuffer;

pub struct App {
    pub idx: usize,
    delta: i32,
    pub port_data_buffer: SonarDataBuffer,
    pub starboard_data_buffer: SonarDataBuffer,
    row_max: usize,
}

impl App {
    pub fn new(
        port_data: Vec<f32>,
        starboard_data: Vec<f32>,
        padded_len: usize,
        row_max: usize,
    ) -> Self {
        let dimensions = (padded_len as u32, 2048);
        let port_data_buffer = SonarDataBuffer::new(port_data, dimensions);
        let starboard_data_buffer = SonarDataBuffer::new(starboard_data, dimensions);

        Self {
            idx: 0,
            delta: 0,
            port_data_buffer,
            starboard_data_buffer,
            row_max,
        }
    }

    pub fn initialize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.port_data_buffer.initialize(device, queue);
        self.starboard_data_buffer.initialize(device, queue);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
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
                self.delta = 10;
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
                self.delta = -10;
                true
            }
            _ => false,
        }
    }

    // Is there a better way to allow the app to access the GPU device/queue?
    pub fn update(&mut self, queue: &wgpu::Queue) {
        let delta = self.delta;
        let old_row_idx = self.idx as i32;
        let old_tile_idx = old_row_idx / 256;

        let tile_max = (self.row_max / 256) as i32;

        let new_row_idx = (old_row_idx + delta).clamp(0, 256 * tile_max - 1024);
        let new_tile_idx = new_row_idx / 256;

        if (new_tile_idx > old_tile_idx) && (new_tile_idx < tile_max - 5) {
            self.port_data_buffer
                .update_buffer_from_tile(queue, (new_tile_idx + 5) as usize);
            self.starboard_data_buffer
                .update_buffer_from_tile(queue, (new_tile_idx + 5) as usize);
            log::debug!("Loading tile {}", new_tile_idx + 5);
        } else if (new_tile_idx < old_tile_idx) && (new_tile_idx >= 2) {
            self.port_data_buffer
                .update_buffer_from_tile(queue, (new_tile_idx - 2) as usize);
            self.starboard_data_buffer
                .update_buffer_from_tile(queue, (new_tile_idx - 2) as usize);
            log::debug!("Loading tile {}", new_tile_idx - 2);
        }

        // Update the index
        self.idx = new_row_idx as usize;
        self.delta = 0;
    }
}
