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
	Self {builder}
    }
}

pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

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
    fn measure(&self, size: Size) -> Size;
    fn layout(&self);
    fn draw(&self, cx: &mut RenderContext);
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
    fn measure(&self, size: Size) -> Size {
        size
    }
    fn layout(&self) {}
    fn draw(&self, cx: &mut RenderContext) {
        self.top.draw(cx);
        self.bottom.draw(cx);
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
    fn measure(&self, size: Size) -> Size {
        size
    }
    fn layout(&self) {}
    fn draw(&self, cx: &mut RenderContext) {
        self.left.draw(cx);
        self.right.draw(cx);
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
    fn measure(&self, size: Size) -> Size {
        size
    }

    fn layout(&self) {}

    fn draw(&self, cx: &mut RenderContext) {
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
    fn measure(&self, size: Size) -> Size {
        size
    }

    fn layout(&self) {}

    fn draw(&self, cx: &mut RenderContext) {
        // This is different from the ping plot because we need
        // to run the sonar rendering pipeline
    }
}

pub struct ScrollBar {
    location: f64,
    radius: f64,
    background_color: Color,
    foreground_color: Color,
    origin: Point,
    size: Size,
}

impl ScrollBar {
    pub fn new(
        location: f64,
        radius: f64,
        background_color: Color,
        foreground_color: Color,
        origin: Point,
        size: Size,
    ) -> Self {
        Self {
            location,
            radius,
            background_color,
            foreground_color,
            origin,
            size,
        }
    }
}

impl View for ScrollBar {
    fn measure(&self, size: Size) -> Size {
        size
    }

    fn layout(&self) {}

    fn draw(&self, cx: &mut RenderContext) {
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
            &Rect::new(origin.x, origin.y, origin.x + size.width, origin.y + size.height),
        );

        let idx_point = (origin.x + self.radius, origin.y + size.height * (1.0 - self.location));

        builder.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.foreground_color,
            None,
            &Circle::new(idx_point, self.radius),
        );

        // Append our fragment to the scene
        cx.builder.append(&fragment, Some(Affine::IDENTITY));
    }
}
