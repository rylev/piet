//! The Cairo backend for the Piet 2D graphics abstraction.

use cairo::{Context, LineCap, LineJoin};

use kurbo::{PathEl, QuadBez, Shape, Vec2};

use piet::{FillRule, RenderContext, RoundInto};

pub struct CairoRenderContext<'a> {
    // Cairo has this as Clone and with &self methods, but we do this to avoid
    // concurrency problems.
    ctx: &'a mut Context,
}

impl<'a> CairoRenderContext<'a> {
    pub fn new(ctx: &mut Context) -> CairoRenderContext {
        CairoRenderContext { ctx }
    }
}

pub enum Brush {
    Solid(u32),
}

// TODO: This cannot be used yet because the `piet::RenderContext` trait
// needs to expose a way to create stroke styles.
pub struct StrokeStyle {
    line_join: Option<LineJoin>,
    line_cap: Option<LineCap>,
    dash: Option<(Vec<f64>, f64)>,
    miter_limit: Option<f64>,
}

impl StrokeStyle {
    pub fn new() -> StrokeStyle {
        StrokeStyle {
            line_join: None,
            line_cap: None,
            dash: None,
            miter_limit: None,
        }
    }

    pub fn line_join(mut self, line_join: LineJoin) -> Self {
        self.line_join = Some(line_join);
        self
    }

    pub fn line_cap(mut self, line_cap: LineCap) -> Self {
        self.line_cap = Some(line_cap);
        self
    }

    pub fn dash(mut self, dashes: Vec<f64>, offset: f64) -> Self {
        self.dash = Some((dashes, offset));
        self
    }

    pub fn miter_limit(mut self, miter_limit: f64) -> Self {
        self.miter_limit = Some(miter_limit);
        self
    }
}

fn convert_fill_rule(fill_rule: piet::FillRule) -> cairo::FillRule {
    match fill_rule {
        piet::FillRule::NonZero => cairo::FillRule::Winding,
        piet::FillRule::EvenOdd => cairo::FillRule::EvenOdd,
    }
}

impl<'a> RenderContext for CairoRenderContext<'a> {
    /// Cairo mostly uses raw f64, so this is as convenient as anything.
    type Point = Vec2;
    type Coord = f64;
    type Brush = Brush;
    type StrokeStyle = StrokeStyle;

    fn clear(&mut self, rgb: u32) {
        self.ctx.set_source_rgb(
            byte_to_frac(rgb >> 16),
            byte_to_frac(rgb >> 8),
            byte_to_frac(rgb),
        );
        self.ctx.paint();
    }

    fn solid_brush(&mut self, rgba: u32) -> Brush {
        Brush::Solid(rgba)
    }

    fn fill(
        &mut self,
        shape: &impl Shape,
        brush: &Self::Brush,
        fill_rule: FillRule,
    ) {
        self.set_path(shape);
        self.set_brush(brush);
        self.ctx.set_fill_rule(convert_fill_rule(fill_rule));
        self.ctx.fill();
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
        self.set_brush(brush);
        self.ctx.stroke();
    }
}

impl<'a> CairoRenderContext<'a> {
    /// Set the source pattern to the brush.
    ///
    /// Cairo is super stateful, and we're trying to have more retained stuff.
    /// This is part of the impedance matching.
    fn set_brush(&mut self, brush: &Brush) {
        match *brush {
            Brush::Solid(rgba) => self.ctx.set_source_rgba(
                byte_to_frac(rgba >> 24),
                byte_to_frac(rgba >> 16),
                byte_to_frac(rgba >> 8),
                byte_to_frac(rgba),
            ),
        }
    }

    /// Set the stroke parameters.
    fn set_stroke(&mut self, width: f64, style: Option<&StrokeStyle>) {
        self.ctx.set_line_width(width);

        let line_join = style
            .and_then(|style| style.line_join)
            .unwrap_or(LineJoin::Miter);
        self.ctx.set_line_join(line_join);

        let line_cap = style
            .and_then(|style| style.line_cap)
            .unwrap_or(LineCap::Butt);
        self.ctx.set_line_cap(line_cap);

        let miter_limit = style.and_then(|style| style.miter_limit).unwrap_or(10.0);
        self.ctx.set_miter_limit(miter_limit);

        match style.and_then(|style| style.dash.as_ref()) {
            None => self.ctx.set_dash(&[], 0.0),
            Some((dashes, offset)) => self.ctx.set_dash(dashes, *offset),
        }
    }

    fn set_path(&mut self, shape: &impl Shape) {
        // This shouldn't be necessary, we always leave the context in no-path
        // state. But just in case, and it should be harmless.
        self.ctx.new_path();
        let mut last = Vec2::default();
        for el in shape.to_bez_path(1e-3) {
            match el {
                PathEl::Moveto(p) => {
                    self.ctx.move_to(p.x, p.y);
                    last = p;
                }
                PathEl::Lineto(p) => {
                    self.ctx.line_to(p.x, p.y);
                    last = p;
                }
                PathEl::Quadto(p1, p2) => {
                    let q = QuadBez::new(last, p1, p2);
                    let c = q.raise();
                    self.ctx
                        .curve_to(c.p1.x, c.p1.y, c.p2.x, c.p2.y, p2.x, p2.y);
                    last = p2;
                }
                PathEl::Curveto(p1, p2, p3) => {
                    self.ctx.curve_to(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
                    last = p3;
                }
                PathEl::Closepath => self.ctx.close_path(),
            }
        }
    }
}

fn byte_to_frac(byte: u32) -> f64 {
    ((byte & 255) as f64) * (1.0 / 255.0)
}
