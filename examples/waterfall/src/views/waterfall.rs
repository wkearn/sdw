use super::{Point, RenderContext, Size, View};

pub struct WaterfallPlot<'a> {
    data_max: f32,
    viewport_location: f32,
    viewport_buffer: &'a wgpu::Buffer,
    starboard_offset_buffer: &'a wgpu::Buffer,
    port_offset_buffer: &'a wgpu::Buffer,
    scale_transform_buffer: &'a wgpu::Buffer,
    window_size: Size,
    nominal_size: Size,
    actual_size: Size,
}

impl<'a> WaterfallPlot<'a> {
    pub fn new(
	data_max: f32,
        viewport_location: f32,
        viewport_buffer: &'a wgpu::Buffer,
        starboard_offset_buffer: &'a wgpu::Buffer,
        port_offset_buffer: &'a wgpu::Buffer,
        scale_transform_buffer: &'a wgpu::Buffer,
        window_size: &Size,
        nominal_size: &Size,
    ) -> Self {
        Self {
	    data_max,
            viewport_location,
            viewport_buffer,
            starboard_offset_buffer,
            port_offset_buffer,
            scale_transform_buffer,
            window_size: window_size.to_owned(),
            nominal_size: nominal_size.to_owned(),
            actual_size: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }
}

impl<'a> View for WaterfallPlot<'a> {
    fn layout(&mut self, min_size: &Size, max_size: &Size) -> Size {
        // This is the same as the box layout
        let (nominal_width, nominal_height) = (self.nominal_size.width, self.nominal_size.height);

        let width = nominal_width.min(max_size.width).max(min_size.width);
        let height = nominal_height.min(max_size.height).max(min_size.height);
        let size = Size { width, height };

        self.actual_size = size;

        size
    }
    fn draw(&self, pos: &Point, cx: &mut RenderContext) {
        // This is different from the ping plot because we need
        // to run the sonar rendering pipeline
        // Update the viewport, offset and transform buffers,

        let (actual_width, actual_height) = (self.actual_size.width, self.actual_size.height);
        let (window_width, window_height) = (self.window_size.width, self.window_size.height);

        let normalized_pos = Point {
            x: pos.x / window_width,
            y: pos.y / window_height,
        };
        let normalized_size = Size {
            width: actual_width / window_width,
            height: actual_height / window_height,
        };

        cx.queue.write_buffer(
            self.viewport_buffer,
            0,
            bytemuck::cast_slice(&[self.data_max, self.viewport_location]),
        );

        cx.queue.write_buffer(
            self.starboard_offset_buffer,
            0,
            bytemuck::cast_slice(&[
                (2.0 * normalized_pos.x + normalized_size.width - 1.0) as f32,
                (-2.0 * normalized_pos.y - 2.0 * normalized_size.height + 1.0) as f32,
                0.0f32,
                0.0f32,
            ]),
        );

        cx.queue.write_buffer(
            self.port_offset_buffer,
            0,
            bytemuck::cast_slice(&[
                (2.0 * normalized_pos.x - 1.0) as f32,
                (-2.0 * normalized_pos.y - 2.0 * normalized_size.height + 1.0) as f32,
                0.0f32,
                0.0f32,
            ]),
        );

        cx.queue.write_buffer(
            self.scale_transform_buffer,
            0,
            bytemuck::cast_slice(&[
                [normalized_size.width as f32, 0.0f32],
                [0.0f32, 2.0f32 * (normalized_size.height as f32)],
            ]),
        );

        // Update the textures
    }
}
