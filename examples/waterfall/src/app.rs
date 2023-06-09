use winit::event::*;

use crate::render::Renderer;
use crate::sonar_data::SonarDataBuffer;
use crate::views::{self, View};

use vello::peniko::Color;

pub struct App {
    pub idx: usize,
    delta: i32,
    pub port_data_buffer: SonarDataBuffer,
    pub starboard_data_buffer: SonarDataBuffer,
    col_max: usize,
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
            col_max: padded_len,
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
                self.delta = 32;
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
                self.delta = -32;
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.delta = match &delta {
                    winit::event::MouseScrollDelta::LineDelta(_, dy) => 32 * (dy.round() as i32),
                    _ => 0,
                };
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

    pub fn plot_pings(&self) -> (&[f32], &[f32]) {
        let dims = (self.col_max, self.row_max);
        let idx = self.idx;
        (
            &self
                .starboard_data_buffer
                .slice((idx * dims.0)..(idx + 1) * dims.0),
            &self
                .port_data_buffer
                .slice((idx * dims.0)..(idx + 1) * dims.0),
        )
    }

    pub fn plot_idx(&self) -> f64 {
        (self.idx as f64) / (self.row_max as f64)
    }

    pub fn to_view<'a>(
        &'a self,
        renderer: &'a Renderer,
        width: f64,
        height: f64,
    ) -> impl View + 'a {
        let idx_plot = self.plot_idx();
        let (starboard_ping_data, port_ping_data) = self.plot_pings();

        let ping_plot = views::PingPlot::new(
            starboard_ping_data,
            port_ping_data,
            Color::rgb8(255, 255, 255),
            Color::rgb8(0, 0, 0),
            views::Size::new(width, height / 4.0),
        );

        let waterfall = views::waterfall::WaterfallPlot::new(
            (self.idx as f32) / 256.0,
            &renderer.viewport_buffer,
            &renderer.starboard_offset_buffer,
            &renderer.port_offset_buffer,
            &renderer.scale_transform_buffer,
            &views::Size::new(width, height),
            &views::Size::new(width, 3.0 * height / 4.0),
        );

        let scroll_wrapper = views::scroll::ScrollOverlay::new(
            waterfall,
            idx_plot,
            1024.0 / (self.row_max as f64),
            10.0,
            Color::rgba8(0, 0, 0, 63),
            Color::rgba8(200, 200, 200, 127),
        );

        views::Container::new(
            views::VerticalStack::new(scroll_wrapper, ping_plot, Color::TRANSPARENT),
            Color::TRANSPARENT,
            views::Size::new(5.0, 5.0),
            views::Size::new(width, height),
        )
    }
}
