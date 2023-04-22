use vello::{
    kurbo::{Affine, BezPath, Circle, PathEl, Rect},
    peniko::{Color, Fill, Stroke},
    SceneBuilder, SceneFragment,
};

pub struct RenderContext<'a> {
    builder: SceneBuilder<'a>,
}

impl<'a> RenderContext<'a> {
    pub fn new(builder: SceneBuilder<'a>) -> Self {
        Self { builder }
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
    fn layout(&mut self, min_size: &Size, max_size: &Size) -> Size {
        max_size.to_owned()
    }
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

	log::debug!("Box size: {:?}",self.actual_size);

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
            width: min_width - 2.0 * self.padding.width,
            height: min_height - 2.0 * self.padding.height,
        };
        let child_max_size = Size {
            width: max_width - 2.0 * self.padding.width,
            height: max_height - 2.0 * self.padding.height,
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

	log::debug!("Container size: {:?}",self.actual_size);

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

struct VerticalStack<T, B>
where
    T: View,
    B: View,
{
    background: Color,
    top: T,
    bottom: B,
}

impl<T, B> View for VerticalStack<T, B>
where
    T: View,
    B: View,
{
    fn draw(&self, pos: &Point, cx: &mut RenderContext) {
        self.top.draw(pos, cx);
        self.bottom.draw(pos, cx);
    }
}

struct HorizontalStack<L, R>
where
    L: View,
    R: View,
{
    left: L,
    right: R,
}

impl<L, R> View for HorizontalStack<L, R>
where
    L: View,
    R: View,
{
    fn draw(&self, pos: &Point, cx: &mut RenderContext) {
        self.left.draw(pos, cx);
        self.right.draw(pos, cx);
    }
}

pub struct PingPlot<'a> {
    starboard_data: &'a [f32],
    port_data: &'a [f32],
    background_color: Color,
    foreground_color: Color,
    origin: Point,
    size: Size,
}

impl<'a> PingPlot<'a> {
    pub fn new(
        starboard_data: &'a [f32],
        port_data: &'a [f32],
        background_color: Color,
        foreground_color: Color,
        origin: Point,
        size: Size,
    ) -> Self {
        PingPlot {
            starboard_data,
            port_data,
            foreground_color,
            background_color,
            origin,
            size,
        }
    }
}

impl<'a> View for PingPlot<'a> {
    fn draw(&self, pos: &Point, cx: &mut RenderContext) {
        let mut fragment = SceneFragment::new();
        let mut builder = SceneBuilder::for_fragment(&mut fragment);

        let origin = &self.origin;
        let size = &self.size;

        let starboard_transform = Affine::map_unit_square(Rect::new(
            origin.x + size.width / 2.0,
            origin.y + size.height,
            origin.x + size.width,
            origin.y,
        ));

        let port_transform = Affine::map_unit_square(Rect::new(
            origin.x + size.width / 2.0,
            origin.y + size.height,
            origin.x,
            origin.y,
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

struct WaterfallPlot {}

impl View for WaterfallPlot {
    fn draw(&self, pos: &Point, cx: &mut RenderContext) {
        // This is different from the ping plot because we need
        // to run the sonar rendering pipeline
    }
}

pub struct ScrollBar {
    location: f64,
    background_color: Color,
    foreground_color: Color,
    origin: Point,
    size: Size,
    row_max: usize,
}

impl ScrollBar {
    pub fn new(
        location: f64,
        background_color: Color,
        foreground_color: Color,
        origin: Point,
        size: Size,
        row_max: usize,
    ) -> Self {
        Self {
            location,
            background_color,
            foreground_color,
            origin,
            size,
            row_max,
        }
    }
}

impl View for ScrollBar {
    fn draw(&self, pos: &Point, cx: &mut RenderContext) {
        let mut fragment = SceneFragment::new();
        let mut builder = SceneBuilder::for_fragment(&mut fragment);

        let origin = &self.origin;
        let size = &self.size;

        let transform = Affine::map_unit_square(Rect::new(
            origin.x,
            origin.y + size.height,
            origin.x + size.width,
            origin.y,
        ));

        builder.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.background_color,
            None,
            &Rect::new(
                origin.x,
                origin.y,
                origin.x + size.width,
                origin.y + size.height,
            ),
        );

        let pos = (origin.x, origin.y + size.height * (1.0 - self.location));
        let slider_height = 1024.0 * (size.height / (self.row_max as f64));

        builder.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.foreground_color,
            None,
            &Rect::new(pos.0, pos.1, pos.0 + size.width, pos.1 - slider_height),
        );

        // Append our fragment to the scene
        cx.builder.append(&fragment, Some(Affine::IDENTITY));
    }
}