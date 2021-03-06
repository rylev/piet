//! The Web Canvas backend for the Piet 2D graphics abstraction.

use stdweb::web::{CanvasRenderingContext2d, FillRule};

use kurbo::{PathEl, Shape, Vec2};

use piet::{RenderContext, RoundInto};

pub struct WebRenderContext<'a> {
    ctx: &'a mut CanvasRenderingContext2d,
}

impl<'a> WebRenderContext<'a> {
    pub fn new(ctx: &mut CanvasRenderingContext2d) -> WebRenderContext {
        WebRenderContext { ctx }
    }
}

pub enum Brush {
    Solid(u32),
}

pub enum StrokeStyle {
    // TODO: actual stroke style options
    Default,
}

fn convert_fill_rule(fill_rule: piet::FillRule) -> FillRule {
    match fill_rule {
        piet::FillRule::NonZero => FillRule::NonZero,
        piet::FillRule::EvenOdd => FillRule::EvenOdd,
    }
}

impl<'a> RenderContext for WebRenderContext<'a> {
    /// stdweb doesn't have a native Point type, so use kurbo's.
    type Point = Vec2;
    type Coord = f64;
    type Brush = Brush;
    type StrokeStyle = StrokeStyle;

    fn clear(&mut self, _rgb: u32) {
        // TODO: we might need to know the size of the canvas to do this.
    }

    fn solid_brush(&mut self, rgba: u32) -> Brush {
        Brush::Solid(rgba)
    }

    fn fill(
        &mut self,
        shape: &impl Shape,
        brush: &Self::Brush,
        fill_rule: piet::FillRule,
    ) {
        self.set_path(shape);
        self.set_brush(brush, true);
        self.ctx.fill(convert_fill_rule(fill_rule));
    }

    fn stroke(
        &mut self,
        shape: &impl Shape,
        brush: &Self::Brush,
        width: impl RoundInto<Self::Coord>,
        style: Option<&Self::StrokeStyle>,
    ) {
        self.set_path(shape);
        self.set_stroke(width.round_into(), style);
        self.set_brush(brush, false);
        self.ctx.stroke();
    }
}

impl<'a> WebRenderContext<'a> {
    /// Set the source pattern to the brush.
    ///
    /// Cairo is super stateful, and we're trying to have more retained stuff.
    /// This is part of the impedance matching.
    fn set_brush(&mut self, brush: &Brush, is_fill: bool) {
        match *brush {
            Brush::Solid(rgba) => {
                let rgb = rgba >> 8;
                let a = rgba & 0xff;
                let color_str = if a == 0xff {
                    format!("#{:06x}", rgba >> 8)
                } else {
                    format!(
                        "rgba({},{},{},{:.3})",
                        (rgb >> 16) & 0xff,
                        (rgb >> 8) & 0xff,
                        rgb & 0xff,
                        byte_to_frac(a)
                    )
                };
                if is_fill {
                    self.ctx.set_fill_style_color(&color_str);
                } else {
                    self.ctx.set_stroke_style_color(&color_str);
                }
            }
        }
    }

    /// Set the stroke parameters.
    fn set_stroke(&mut self, width: f64, style: Option<&StrokeStyle>) {
        self.ctx.set_line_width(width);
        if let Some(style) = style {
            match style {
                // TODO: actual stroke style parameters
                StrokeStyle::Default => (),
            }
        }
    }

    fn set_path(&mut self, shape: &impl Shape) {
        // This shouldn't be necessary, we always leave the context in no-path
        // state. But just in case, and it should be harmless.
        self.ctx.begin_path();
        for el in shape.to_bez_path(1e-3) {
            match el {
                PathEl::Moveto(p) => self.ctx.move_to(p.x, p.y),
                PathEl::Lineto(p) => self.ctx.line_to(p.x, p.y),
                PathEl::Quadto(p1, p2) => self.ctx.quadratic_curve_to(p1.x, p1.y, p2.x, p2.y),
                PathEl::Curveto(p1, p2, p3) => {
                    self.ctx.bezier_curve_to(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y)
                }
                PathEl::Closepath => self.ctx.close_path(),
            }
        }
    }
}

fn byte_to_frac(byte: u32) -> f64 {
    ((byte & 255) as f64) * (1.0 / 255.0)
}
