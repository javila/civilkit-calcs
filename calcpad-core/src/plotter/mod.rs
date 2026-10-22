//! Port of Plotter/Plotter.cs: the shared part of the chart ($Plot) and map ($Map)
//! plotters.
//!
//! Only the vector (SVG) output path is ported. CalcpadCE draws raster plots with
//! SkiaSharp (font rendering, anti aliasing) and writes plot images to files when
//! `PlotSettings.ImagePath` is set; neither can be reproduced here, and both are reported
//! as out of scope. The colour map of `$Map` is computed pixel by pixel and embedded as a
//! PNG, which is ported (see `png.rs`).
pub mod capture;
pub mod chart_plotter;
pub mod map_plotter;
pub mod png;
pub mod svg_drawing;
pub mod svg_point;

use crate::math_parser_exception::{CResult, MathParserException};
use crate::netfmt;
use crate::output::text_writer::TextWriter;
use crate::output::OutputWriter;
use crate::parsers::math_parser::MathParser;
use crate::settings::PlotSettings;
use svg_drawing::SvgDrawing;

const STEPS: [f64; 4] = [1.0, 2.0, 2.5, 5.0];

/// `Math.Round(x)` (banker's rounding)
#[inline]
pub(crate) fn round(x: f64) -> f64 {
    x.round_ties_even()
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Node {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Node {
    pub fn new(x: f64, y: f64, z: f64) -> Node {
        Node { x, y, z }
    }
}

/// `Plotter.Box`
#[derive(Clone, Copy, Debug)]
pub struct PlotBox {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

impl PlotBox {
    pub fn empty() -> PlotBox {
        PlotBox {
            left: f64::MAX,
            bottom: f64::MAX,
            right: f64::MIN,
            top: f64::MIN,
        }
    }

    pub fn new(left: f64, bottom: f64, right: f64, top: f64) -> PlotBox {
        PlotBox {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn width(&self) -> f64 {
        self.right - self.left
    }

    pub fn height(&self) -> f64 {
        self.top - self.bottom
    }

    pub fn expand(&mut self, node: &Node) {
        if node.x < self.left {
            self.left = node.x;
        }
        if node.x > self.right {
            self.right = node.x;
        }
        if node.y < self.bottom {
            self.bottom = node.y;
        }
        if node.y > self.top {
            self.top = node.y;
        }
    }

    pub fn expand_box(&mut self, b: &PlotBox) {
        if b.left < self.left {
            self.left = b.left;
        }
        if b.right > self.right {
            self.right = b.right;
        }
        if b.bottom < self.bottom {
            self.bottom = b.bottom;
        }
        if b.top > self.top {
            self.top = b.top;
        }
    }
}

/// `Math.Clamp(value, min, max)`, which throws when `min > max`.
pub(crate) fn clamp(value: f64, min: f64, max: f64) -> CResult<f64> {
    if min > max {
        return Err(MathParserException::unexpected(format!(
            "'{}' cannot be greater than {}.",
            netfmt::double_to_string(min),
            netfmt::double_to_string(max)
        )));
    }
    Ok(if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    })
}

/// The members of the abstract `Plotter` class.
pub struct Plotter<'a> {
    pub screen_scale_factor: f32,
    pub width: i32,
    pub height: i32,
    pub margin: i32,
    pub left: i32,
    pub right: i32,
    pub settings: &'a PlotSettings,
    pub parser: &'a MathParser,
    pub writer: TextWriter,
}

impl<'a> Plotter<'a> {
    pub fn new(parser: &'a MathParser, settings: &'a PlotSettings) -> Plotter<'a> {
        let ssf = 2.0 * settings.screen_scale_factor as f32;
        let margin = (30.0 * ssf) as i32;
        Plotter {
            screen_scale_factor: ssf,
            width: 500,
            height: 350,
            margin,
            left: margin,
            right: margin,
            settings,
            parser,
            writer: TextWriter::new(None, parser.phasor.get()),
        }
    }

    /// `IsVector`
    pub fn is_vector(&self) -> CResult<bool> {
        let v = self.parser.get_settings_variable("PlotSVG", f64::NAN)?;
        Ok(if v.is_nan() {
            self.settings.vector_graphics
        } else {
            v != 0.0
        })
    }

    /// `IsAdaptive`
    pub fn is_adaptive(&self) -> CResult<bool> {
        let v = self.parser.get_settings_variable("PlotAdaptive", f64::NAN)?;
        Ok(if v.is_nan() {
            self.settings.is_adaptive
        } else {
            v != 0.0
        })
    }

    /// `FixBounds(ref min, ref max)`
    pub fn fix_bounds(min: &mut f64, max: &mut f64) {
        if *min > *max {
            std::mem::swap(min, max);
        }
        let mut d = min.abs().max(max.abs());
        d = if d < 1.0 { *max - *min } else { (*max - *min) / d };
        if d >= 1e-14 {
            return;
        }
        let mut mid = (*min + *max) / 2.0;
        let span = 10f64.powf(round(mid.abs().log10()) - 2.0);
        if span != 0.0 {
            mid = round(*min / span) * span;
        }
        if d == 0.0 {
            d = 0.1;
        }
        if d < span {
            d = span;
        }
        *min = mid - d;
        *max = mid + d;
    }

    pub(crate) fn writer_fmt(&self, d: f64) -> String {
        self.writer.format_number_helper(d, None)
    }

    /// `DrawGridSvg(canvas, x0, y0, xs, ys, bounds)`
    pub fn draw_grid_svg(&self, canvas: &mut SvgDrawing, x0: f64, y0: f64, xs: f64, ys: f64, bounds: &PlotBox) {
        let ssf = self.screen_scale_factor as f64;
        let tw = 5.5 * ssf;
        let th = 4.5 * ssf;
        let left = self.left as f64;
        let margin = self.margin as f64;
        let xn = (self.width - self.right) as f64;
        let yn = (self.height - self.margin) as f64;
        let mut max_steps = (15.0 * yn / xn).min((yn - margin) / (2.5 * th)) as i32;
        let mut delta = bounds.top - bounds.bottom;
        let mut step_y = get_grid_step(delta, max_steps);
        if step_y * ys > 8.0 * th && delta < 7.0 * step_y {
            step_y /= 2.0;
        }
        let mut tol = delta * 0.02;
        let mut yg = round(bounds.bottom / step_y) * step_y;
        let mut min = bounds.bottom - tol;
        if yg < min {
            yg += step_y;
        }
        let mut max = bounds.top + tol;
        let mut is_scientific = bounds.top.abs() + bounds.bottom.abs() >= 20000.0;
        let mut a: f32 = 4.0 * self.screen_scale_factor;
        let xt = left - tw / 2.0 - a as f64 / 2.0;
        canvas.draw_rectangle(left, margin, xn - left, yn - margin, "Frame");
        let mut guard = 0;
        while (yg < max) == (step_y > 0.0) {
            let y = y0 - yg * ys;
            if y.abs() > 1e8 || guard > 100_000 {
                break;
            }
            guard += 1;
            if yg >= bounds.bottom && yg <= bounds.top {
                canvas.draw_line(left, y, xn, y, "Grid");
            }
            let s = if yg.abs() < step_y * 1e-8 {
                "0".to_string()
            } else if is_scientific {
                netfmt::format_custom(yg, "0.##E+0")
            } else {
                self.writer_fmt(yg)
            };
            canvas.draw_text(&s, xt, y + th, "sm end");
            yg += step_y;
        }
        let sx0 = self.writer_fmt(bounds.left);
        let sx1 = self.writer_fmt(bounds.right);
        let mut n = sx0.chars().count().max(sx1.chars().count()) + 3;
        max_steps = 15.min(((xn - left) / (tw * n as f64)) as i32);
        delta = bounds.right - bounds.left;
        let mut step_x = get_grid_step(delta, max_steps);
        let mut mid_line = step_x * xs > 1.5 * step_y * ys;
        if mid_line {
            step_x /= 2.0;
        }
        tol = delta * 0.02;
        min = bounds.left - tol;
        let mut xg = round(min / step_x) * step_x;
        if xg < min {
            xg += step_x;
        }
        max = bounds.right + tol;
        let yt = yn + 2.0 * th + a as f64;
        is_scientific = bounds.right.abs() + bounds.left.abs() >= 20000.0;
        if mid_line {
            let n1 = self.writer_fmt(round(max / step_x) * step_x).chars().count();
            let n2 = self.writer_fmt(xg + step_x).chars().count();
            n = n1.max(n2) + 1;
            if tw * (n as f64) < step_x * xs {
                mid_line = false;
            }
        }
        let mut even = true;
        guard = 0;
        while (xg < max) == (step_x > 0.0) {
            let x = x0 + xg * xs;
            if x.abs() > 1e8 || guard > 100_000 {
                break;
            }
            guard += 1;
            if xg >= bounds.left && xg <= bounds.right {
                canvas.draw_line(x, margin, x, yn, "Grid");
            }
            if even {
                let s = if xg.abs() < step_x * 1e-8 {
                    "0".to_string()
                } else if is_scientific {
                    netfmt::format_custom(xg, "0.##E+0")
                } else {
                    self.writer_fmt(xg)
                };
                canvas.draw_text(&s, x, yt, "sm mid");
            }
            if mid_line {
                even = !even;
            }
            xg += step_x;
        }
        if y0 >= margin - 0.1 && y0 <= yn + 0.1 {
            canvas.draw_line(left, y0, xn, y0, "Axis");
            canvas.draw_text("x", xn + tw, y0 - th, "");
        }
        if x0 >= left - 0.1 && x0 <= xn + 0.1 {
            canvas.draw_line(x0, margin, x0, yn, "Axis");
            canvas.draw_text("y", x0, margin - 2.0 * th, "mid");
        }
        let sy0 = self.writer_fmt(bounds.bottom);
        let sy1 = self.writer_fmt(bounds.top);
        a /= 2.0;
        let a = a as f64;
        canvas.draw_text(&format!("[{}; {}]", sx0, sy0), a, self.height as f64 - 0.5 * th - a, "");
        canvas.draw_text(
            &format!("[{}; {}]", sx1, sy1),
            self.width as f64 - a,
            2.0 * th + a,
            "end",
        );
    }

    /// `HtmlImg(string src)`
    pub fn html_img(&self, src: &str) -> String {
        let w = round(0.75 * self.width as f64 / self.screen_scale_factor as f64);
        let data_attr = if src.starts_with("data:image/png") {
            " data-plot=\"png\""
        } else {
            ""
        };
        format!(
            "<img class=\"plot\"{} src=\"{}\" alt=\"Plot\" style=\"width:{}pt;\">",
            data_attr,
            src,
            netfmt::double_to_string(w)
        )
    }
}

/// `GetGridStep(double span, int maxSteps)`
fn get_grid_step(span: f64, max_steps: i32) -> f64 {
    let a = span.abs();
    let n = round(a.log10()) as i32;
    let k = 10f64.powf(n as f64);
    let st = a / k;
    let mut d_min = 10.0;
    let mut max_steps = max_steps;
    if max_steps < 2 {
        max_steps = 2;
    }
    if max_steps > 15 {
        max_steps = 15;
    }
    let mut step = 2.5 * round(4.0 * st / max_steps as f64);
    for s in STEPS {
        let d = (st - s).abs();
        if d < d_min && (10.0 * st / s).floor() <= max_steps as f64 {
            d_min = d;
            step = s;
        }
    }
    (step * (k / 10.0)).copysign(span)
}

/// Raster (PNG) plots are drawn with SkiaSharp in CalcpadCE and cannot be reproduced.
pub(crate) fn raster_not_supported(parser: &MathParser) -> MathParserException {
    parser.note_out_of_scope("raster plots (SkiaSharp)");
    MathParserException::new("Raster (PNG) plots are not supported by calcpad-rs: CalcpadCE draws them with SkiaSharp.")
}

/// Plot image files (`PlotSettings.ImagePath`) need file output, which the core does not do.
pub(crate) fn image_files_not_supported(parser: &MathParser) -> MathParserException {
    parser.note_out_of_scope("plot image files");
    MathParserException::new("Plot image files are not supported by calcpad-rs.")
}
