use vello::{
    kurbo::{Affine, Rect, RoundedRect},
    peniko::{Color, Fill},
    SceneBuilder, SceneFragment,
};

use super::{Size,Point,View,RenderContext};

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
