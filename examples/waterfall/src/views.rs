use vello::{
    kurbo::{Affine, BezPath, PathEl, Rect, RoundedRect},
    peniko::{Color, Fill, Stroke},
    SceneBuilder, SceneFragment,
};

pub struct RenderContext<'a, 'b> {
    builder: SceneBuilder<'a>,
    queue: &'b wgpu::Queue,
}

impl<'a, 'b> RenderContext<'a, 'b> {
    pub fn new(builder: SceneBuilder<'a>, queue: &'b wgpu::Queue) -> Self {
        Self { builder, queue }
    }
}

#[derive(Debug, Clone)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    width: f64,
    height: f64,
}

impl Size {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

pub trait View {
    fn layout(&mut self, min_size: &Size, max_size: &Size) -> Size;
    fn draw(&self, pos: &Point, cx: &mut RenderContext);
}

pub struct Box {
    foreground_color: Color,
    nominal_size: Size,
    actual_size: Size,
}

impl Box {
    pub fn new(foreground_color: Color, size: Size) -> Self {
        Self {
            foreground_color,
            nominal_size: size.clone(),
            actual_size: size.clone(),
        }
    }
}

impl View for Box {
    fn layout(&mut self, min_size: &Size, max_size: &Size) -> Size {
        let (nominal_width, nominal_height) = (self.nominal_size.width, self.nominal_size.height);

        let width = nominal_width.min(max_size.width).max(min_size.width);
        let height = nominal_height.min(max_size.height).max(min_size.height);
        let size = Size { width, height };

        self.actual_size = size;

        self.actual_size.to_owned()
    }

    fn draw(&self, pos: &Point, cx: &mut RenderContext) {
        let mut fragment = SceneFragment::new();
        let mut builder = SceneBuilder::for_fragment(&mut fragment);

        builder.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.foreground_color,
            None,
            &Rect::new(
                pos.x,
                pos.y,
                pos.x + self.actual_size.width,
                pos.y + self.actual_size.height,
            ),
        );

        cx.builder.append(&fragment, Some(Affine::IDENTITY));
    }
}

pub struct Container<V: View> {
    child: V,
    child_pos: Point,
    background_color: Color,
    padding: Size,
    nominal_size: Size,
    actual_size: Size,
}

impl<V: View> Container<V> {
    pub fn new(child: V, background_color: Color, padding: Size, size: Size) -> Self {
        Self {
            child,
            child_pos: Point { x: 0.0, y: 0.0 },
            background_color,
            padding,
            nominal_size: size.clone(),
            actual_size: size.clone(),
        }
    }
}

impl<V: View> View for Container<V> {
    fn layout(&mut self, min_size: &Size, max_size: &Size) -> Size {
        let (min_width, min_height) = (min_size.width, min_size.height);
        let (max_width, max_height) = (max_size.width, max_size.height);

        let child_min_size = Size {
            width: (min_width - 2.0 * self.padding.width).max(0.0),
            height: (min_height - 2.0 * self.padding.height).max(0.0),
        };
        let child_max_size = Size {
            width: (max_width - 2.0 * self.padding.width).max(0.0),
            height: (max_height - 2.0 * self.padding.height).max(0.0),
        };

        let child_size = self.child.layout(&child_min_size, &child_max_size);

        let size = Size {
            width: (child_size.width + 2.0 * self.padding.width).max(self.nominal_size.width),
            height: (child_size.height + 2.0 * self.padding.height).max(self.nominal_size.height),
        };

        let child_pos = Point {
            x: size.width / 2.0 - child_size.width / 2.0,
            y: size.height / 2.0 - child_size.height / 2.0,
        };

        self.actual_size = size;
        self.child_pos = child_pos;

        self.actual_size.to_owned()
    }

    fn draw(&self, pos: &Point, cx: &mut RenderContext) {
        let mut fragment = SceneFragment::new();
        let mut builder = SceneBuilder::for_fragment(&mut fragment);

        builder.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.background_color,
            None,
            &Rect::new(
                pos.x,
                pos.y,
                pos.x + self.actual_size.width,
                pos.y + self.actual_size.height,
            ),
        );

        cx.builder.append(&fragment, Some(Affine::IDENTITY));

        let child_pos = Point {
            x: pos.x + self.child_pos.x,
            y: pos.y + self.child_pos.y,
        };
        self.child.draw(&child_pos, cx);
    }
}

pub struct VerticalStack<T, B>
where
    T: View,
    B: View,
{
    top: T,
    top_pos: Point,
    bottom: B,
    bottom_pos: Point,
    background_color: Color,
    actual_size: Size,
}

impl<T: View, B: View> VerticalStack<T, B> {
    pub fn new(top: T, bottom: B, background_color: Color) -> Self {
        Self {
            top,
            top_pos: Point { x: 0.0, y: 0.0 },
            bottom,
            bottom_pos: Point { x: 0.0, y: 0.0 },
            background_color,
            actual_size: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }
}

impl<T, B> View for VerticalStack<T, B>
where
    T: View,
    B: View,
{
    fn layout(&mut self, min_size: &Size, max_size: &Size) -> Size {
        let (min_width, min_height) = (min_size.width, min_size.height);
        let (max_width, max_height) = (max_size.width, max_size.height);

        // Figure out how big the top element wants to be
        let top_size = self.top.layout(min_size, max_size);

        let bottom_min_size = Size {
            width: min_width,
            height: (min_height - top_size.height).max(0.0),
        };
        let bottom_max_size = Size {
            width: max_width,
            height: (max_height - top_size.height).max(0.0),
        };

        let bottom_size = self.bottom.layout(&bottom_min_size, &bottom_max_size);

        let size = Size {
            width: top_size.width.max(bottom_size.width),
            height: top_size.height + bottom_size.height,
        };

        self.actual_size = size;
        self.top_pos = Point {
            x: size.width / 2.0 - top_size.width / 2.0,
            y: 0.0,
        };
        self.bottom_pos = Point {
            x: size.width / 2.0 - bottom_size.width / 2.0,
            y: top_size.height,
        };

        size.to_owned()
    }

    fn draw(&self, pos: &Point, cx: &mut RenderContext) {
        let mut fragment = SceneFragment::new();
        let mut builder = SceneBuilder::for_fragment(&mut fragment);

        builder.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.background_color,
            None,
            &Rect::new(
                pos.x,
                pos.y,
                pos.x + self.actual_size.width,
                pos.y + self.actual_size.height,
            ),
        );

        cx.builder.append(&fragment, Some(Affine::IDENTITY));

        let top_pos = Point {
            x: pos.x + self.top_pos.x,
            y: pos.y + self.top_pos.y,
        };
        self.top.draw(&top_pos, cx);

        let bottom_pos = Point {
            x: pos.x + self.bottom_pos.x,
            y: pos.y + self.bottom_pos.y,
        };
        self.bottom.draw(&bottom_pos, cx);
    }
}

pub struct HorizontalStack<L, R>
where
    L: View,
    R: View,
{
    left: L,
    left_pos: Point,
    right: R,
    right_pos: Point,
    background_color: Color,
    actual_size: Size,
}

impl<L: View, R: View> HorizontalStack<L, R> {
    pub fn new(left: L, right: R, background_color: Color) -> Self {
        Self {
            left,
            left_pos: Point { x: 0.0, y: 0.0 },
            right,
            right_pos: Point { x: 0.0, y: 0.0 },
            background_color,
            actual_size: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }
}

impl<L, R> View for HorizontalStack<L, R>
where
    L: View,
    R: View,
{
    fn layout(&mut self, min_size: &Size, max_size: &Size) -> Size {
        let (min_width, min_height) = (min_size.width, min_size.height);
        let (max_width, max_height) = (max_size.width, max_size.height);

        let left_size = self.left.layout(min_size, max_size);

        let right_min_size = Size {
            width: (min_width - left_size.width).max(0.0),
            height: min_height,
        };
        let right_max_size = Size {
            width: (max_width - left_size.width).max(0.0),
            height: max_height,
        };

        let right_size = self.right.layout(&right_min_size, &right_max_size);

        let size = Size {
            width: left_size.width + right_size.width,
            height: left_size.height.max(right_size.height),
        };

        self.actual_size = size;

        self.left_pos = Point {
            x: 0.0,
            y: size.height / 2.0 - left_size.height / 2.0,
        };
        self.right_pos = Point {
            x: left_size.width,
            y: size.height / 2.0 - right_size.height / 2.0,
        };

        size.to_owned()
    }

    fn draw(&self, pos: &Point, cx: &mut RenderContext) {
        let mut fragment = SceneFragment::new();
        let mut builder = SceneBuilder::for_fragment(&mut fragment);

        builder.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.background_color,
            None,
            &Rect::new(
                pos.x,
                pos.y,
                pos.x + self.actual_size.width,
                pos.y + self.actual_size.height,
            ),
        );

        cx.builder.append(&fragment, Some(Affine::IDENTITY));

        let left_pos = Point {
            x: pos.x + self.left_pos.x,
            y: pos.y + self.left_pos.y,
        };

        self.left.draw(&left_pos, cx);

        let right_pos = Point {
            x: pos.x + self.right_pos.x,
            y: pos.y + self.right_pos.y,
        };

        self.right.draw(&right_pos, cx);
    }
}

pub struct PingPlot<'a> {
    starboard_data: &'a [f32],
    port_data: &'a [f32],
    background_color: Color,
    foreground_color: Color,
    nominal_size: Size,
    actual_size: Size,
}

impl<'a> PingPlot<'a> {
    pub fn new(
        starboard_data: &'a [f32],
        port_data: &'a [f32],
        background_color: Color,
        foreground_color: Color,
        size: Size,
    ) -> Self {
        PingPlot {
            starboard_data,
            port_data,
            foreground_color,
            background_color,
            nominal_size: size,
            actual_size: size,
        }
    }
}

impl<'a> View for PingPlot<'a> {
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
        let mut fragment = SceneFragment::new();
        let mut builder = SceneBuilder::for_fragment(&mut fragment);

        let size = &self.actual_size;

        let starboard_transform = Affine::map_unit_square(Rect::new(
            pos.x + size.width / 2.0,
            pos.y + size.height,
            pos.x + size.width,
            pos.y,
        ));

        let port_transform = Affine::map_unit_square(Rect::new(
            pos.x + size.width / 2.0,
            pos.y + size.height,
            pos.x,
            pos.y,
        ));

        builder.fill(
            Fill::NonZero,
            port_transform,
            self.background_color,
            None,
            &Rect::new(0.0, 0.0, 1.0, 1.0),
        );

        builder.fill(
            Fill::NonZero,
            starboard_transform,
            self.background_color,
            None,
            &Rect::new(0.0, 0.0, 1.0, 1.0),
        );

        let ping_max = self
            .starboard_data
            .iter()
            .fold(0.0f32, |acc, &y| acc.max(y));
        let ping_max = self.port_data.iter().fold(ping_max, |acc, &y| acc.max(y));

        let data_len = self.starboard_data.len() as f64;

        let starboard_plot: BezPath = self
            .starboard_data
            .iter()
            .enumerate()
            .map(|(i, &y)| {
                let x = (i as f64) / data_len;
                PathEl::LineTo((x, f64::from(y / ping_max)).into())
            })
            .collect();

        let port_plot: BezPath = self
            .port_data
            .iter()
            .enumerate()
            .map(|(i, &y)| {
                let x = (i as f64) / data_len;
                PathEl::LineTo((x, f64::from(y / ping_max)).into())
            })
            .collect();

        builder.stroke(
            &Stroke::new(0.001),
            starboard_transform,
            self.foreground_color,
            None,
            &starboard_plot,
        );

        builder.stroke(
            &Stroke::new(0.001),
            port_transform,
            self.foreground_color,
            None,
            &port_plot,
        );

        cx.builder.append(&fragment, Some(Affine::IDENTITY));
    }
}

pub struct WaterfallPlot<'a> {
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
        viewport_location: f32,
        viewport_buffer: &'a wgpu::Buffer,
        starboard_offset_buffer: &'a wgpu::Buffer,
        port_offset_buffer: &'a wgpu::Buffer,
        scale_transform_buffer: &'a wgpu::Buffer,
        window_size: &Size,
        nominal_size: &Size,
    ) -> Self {
        Self {
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
            bytemuck::cast_slice(&[0.0f32, self.viewport_location]),
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

pub struct ScrollWrapper<V: View> {
    child: V,
    width: f64,
    background_color: Color,
    foreground_color: Color,
    actual_size: Size,
    scroll_position: f64,
    location: f64,
    slider_size: f64,
}

impl<V: View> ScrollWrapper<V> {
    pub fn new(
        child: V,
        location: f64,
        slider_size: f64,
        width: f64,
        background_color: Color,
        foreground_color: Color,
    ) -> Self {
        Self {
            child,
            width,
            background_color,
            foreground_color,
            actual_size: Size {
                width: 0.0,
                height: 0.0,
            },
            scroll_position: 0.0,
            location,
            slider_size,
        }
    }
}

impl<V: View> View for ScrollWrapper<V> {
    fn layout(&mut self, min_size: &Size, max_size: &Size) -> Size {
        let child_min_size = Size {
            width: (min_size.width - self.width).max(0.0),
            height: min_size.height,
        };
        let child_max_size = Size {
            width: (max_size.width - self.width).max(0.0),
            height: max_size.height,
        };

        let child_size = self.child.layout(&child_min_size, &child_max_size);

        let size = Size {
            width: child_size.width + self.width,
            height: child_size.height,
        };

        self.actual_size = size;
        self.scroll_position = child_size.width;

        size
    }

    fn draw(&self, pos: &Point, cx: &mut RenderContext) {
        let mut fragment = SceneFragment::new();
        let mut builder = SceneBuilder::for_fragment(&mut fragment);

        let size = &self.actual_size;

        // Draw track
        builder.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.background_color,
            None,
            &Rect::new(
                pos.x + self.scroll_position,
                pos.y,
                pos.x + self.scroll_position + self.width,
                pos.y + size.height,
            ),
        );

        // Draw slider
        let slider_pos = Point {
            x: pos.x + self.scroll_position,
            y: pos.y + size.height * (1.0 - self.location),
        };
        let slider_height = self.slider_size * size.height;

        builder.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.foreground_color,
            None,
            &RoundedRect::new(
                slider_pos.x,
                slider_pos.y,
                slider_pos.x + self.width,
                slider_pos.y - slider_height,
                self.width / 2.0,
            ),
        );

        // Append our fragment to the scene
        cx.builder.append(&fragment, Some(Affine::IDENTITY));

        // Draw child at the origin of the ScrollWrapper box
        self.child.draw(pos, cx);
    }
}