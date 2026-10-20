//! Port of Plotter/MapPlotter.cs ($Map).
//!
//! In vector mode CalcpadCE draws the colour map into an SKBitmap pixel by pixel
//! (`SetBitmapBits`) and embeds it in the SVG as a base64 PNG. The pixel bytes are
//! computed exactly as in C#; see `png.rs` for the encoding.
use std::rc::Rc;

use super::svg_drawing::SvgDrawing;
use super::svg_point::SvgPoint;
use super::{Node, PlotBox, Plotter};
use crate::base_types::unit::{OptUnit, Unit};
use crate::base_types::variable::VarRef;
use crate::exceptions::Items;
use crate::math_parser_exception::CResult;
use crate::parsers::math_parser::compiler::Expr;
use crate::parsers::math_parser::MathParser;
use crate::settings::{ColorScales, LightDirections, PlotSettings};

const N_COLORS: usize = 12;

struct Map {
    points: Vec<Vec<Node>>,
    vertices: Vec<Vec<Node>>,
    nx: usize,
    ny: usize,
    min: f64,
    max: f64,
}

impl Map {
    fn new(points: Vec<Vec<Node>>, settings: &PlotSettings) -> Map {
        let nx = points.len();
        let ny = points[0].len();
        let mut m = Map {
            vertices: vec![vec![Node::default(); ny]; nx],
            points,
            nx,
            ny,
            min: f64::MAX,
            max: f64::MIN,
        };
        m.get_bounds();
        if settings.shadows() {
            m.get_shadows();
        }
        m
    }

    fn get_bounds(&mut self) {
        for i in 0..self.nx {
            for j in 0..self.ny {
                let z = self.points[i][j].z;
                if !(z.is_nan() || z.is_infinite()) {
                    if z < self.min {
                        self.min = z;
                    }
                    if z > self.max {
                        self.max = z;
                    }
                }
            }
        }
    }

    fn get_shadows(&mut self) {
        let (nx, ny) = (self.nx - 1, self.ny - 1);
        let p = &self.points;
        let mut dx = (p[nx][0].x - p[0][0].x).abs();
        let mut dy = (p[0][ny].y - p[0][0].y).abs();
        let mut dz = 0.0;
        for i in 0..nx {
            let i1 = if i == 0 { 0 } else { i - 1 };
            let i2 = if i == nx { i } else { i + 1 };
            for j in 0..ny {
                let j1 = if j == 0 { 0 } else { j - 1 };
                let j2 = if j == ny { j } else { j + 1 };
                dz += (p[i1][j].z - p[i2][j].z).abs() + (p[i][j1].z - p[i][j2].z).abs();
            }
        }
        let mut k = 1.0;
        if dz > 0.0 {
            k = 0.5 * (self.ny as f64 * dx + self.nx as f64 * dy) / dz;
        }
        for i in 0..self.nx {
            let i1 = if i == 0 { 0 } else { i - 1 };
            let i2 = if i == nx { i } else { i + 1 };
            dx = k / (p[i2][0].x - p[i1][0].x);
            for j in 0..self.ny {
                let j1 = if j == 0 { 0 } else { j - 1 };
                let j2 = if j == ny { j } else { j + 1 };
                dy = k / (p[i][j2].y - p[i][j1].y);
                let x = (p[i1][j].z - p[i2][j].z) * dx;
                let y = (p[i][j1].z - p[i][j2].z) * dy;
                self.vertices[i][j] = Node::new(x, y, 0.0);
            }
        }
    }
}

pub struct MapPlotter<'a> {
    base: Plotter<'a>,
    nx: usize,
    ny: usize,
    size: i32,
    colors: [Node; N_COLORS],
    k: f64,
}

impl<'a> MapPlotter<'a> {
    pub fn new(parser: &'a MathParser, settings: &'a PlotSettings) -> MapPlotter<'a> {
        MapPlotter {
            base: Plotter::new(parser, settings),
            nx: 0,
            ny: 0,
            size: 0,
            colors: [Node::default(); N_COLORS],
            k: 1.5f64.sqrt(),
        }
    }

    fn get_image_size(&mut self) -> CResult<()> {
        let b = &mut self.base;
        let ssf = b.screen_scale_factor;
        let p = b.parser;
        let w = ((p.get_settings_variable("PlotWidth", b.settings.width as f64)? as i32) as f32 * ssf) as i32;
        let h = ((p.get_settings_variable("PlotHeight", b.settings.height as f64)? as i32) as f32 * ssf) as i32;
        self.size = ((p.get_settings_variable("PlotStep", b.settings.step as f64)? as i32) as f32 * ssf) as i32;
        let d = (((w * w + h * h) as f64).sqrt()) as i32;
        if self.size == 0 {
            self.size = if d >= 800 {
                8
            } else if d >= 400 {
                4
            } else {
                2
            };
        }
        if self.size <= 0 {
            return Err(crate::math_parser_exception::MathParserException::unexpected(
                "Attempted to divide by zero.",
            ));
        }
        self.nx = (w / self.size).max(0) as usize;
        self.ny = (h / self.size).max(0) as usize;
        let w = self.nx as i32 * self.size;
        let h = self.ny as i32 * self.size;
        b.left = (1.5 * b.margin as f64) as i32;
        b.right = (2.5 * b.margin as f64) as i32;
        b.width = w + b.left + b.right;
        b.height = h + 2 * b.margin;
        Ok(())
    }

    /// `Plot(function, varX, varY, startX, endX, startY, endY, xUnits, yUnits)`
    #[allow(clippy::too_many_arguments)]
    pub fn plot(
        &mut self,
        function: &Option<Rc<Expr>>,
        var_x: &VarRef,
        var_y: &VarRef,
        start_x: f64,
        end_x: f64,
        start_y: f64,
        end_y: f64,
        x_units: &OptUnit,
        y_units: &OptUnit,
    ) -> CResult<String> {
        self.get_color_scale();
        self.get_image_size()?;
        let pts = self.calculate(function, var_x, var_y, start_x, end_x, start_y, end_y, x_units, y_units)?;
        let mut m = Map::new(pts, self.base.settings);
        Plotter::fix_bounds(&mut m.min, &mut m.max);
        let b = &self.base;
        let xs = (b.width - b.left - b.right) as f64 / (end_x - start_x);
        let x0 = b.left as f64 - ((start_x * xs) as i32) as f64;
        let ys = (b.height - 2 * b.margin) as f64 / (end_y - start_y);
        let y0 = (b.height - b.margin) as f64 + ((start_y * ys) as i32) as f64;
        let bounds = PlotBox::new(start_x, start_y, end_x, end_y);
        if !b.is_vector()? {
            return Err(super::raster_not_supported(b.parser));
        }
        if !b.settings.image_path.is_empty() {
            return Err(super::image_files_not_supported(b.parser));
        }
        // GetSvgPoints: computed in C# but not used by DrawSvg.
        let _svg_points: Vec<Vec<SvgPoint>> = m
            .points
            .iter()
            .map(|r| r.iter().map(|p| SvgPoint::new(x0 + p.x * xs, y0 - p.y * ys)).collect())
            .collect();
        self.draw_svg(&m, x0, y0, xs, ys, &bounds)
    }

    #[allow(clippy::too_many_arguments)]
    fn calculate(
        &self,
        function: &Option<Rc<Expr>>,
        var_x: &VarRef,
        var_y: &VarRef,
        start_x: f64,
        end_x: f64,
        start_y: f64,
        end_y: f64,
        x_units: &OptUnit,
        y_units: &OptUnit,
    ) -> CResult<Vec<Vec<Node>>> {
        let (nx, ny) = (self.nx, self.ny);
        let mut n = vec![vec![Node::default(); ny + 1]; nx + 1];
        let sx = (end_x - start_x) / nx as f64;
        let sy = (end_y - start_y) / ny as f64;
        let mut x = start_x;
        let mut d = 1.0;
        if let Some(yu) = y_units {
            if Unit::is_consistent_opt(x_units, y_units) {
                d = yu.convert_to(x_units.as_ref().unwrap());
            }
        }
        var_x.set_units(x_units.clone());
        var_y.set_units(y_units.clone());
        let parser = self.base.parser;
        for col in n.iter_mut() {
            parser.break_if_canceled()?;
            var_x.set_number(x);
            let mut y = start_y;
            for node in col.iter_mut() {
                var_y.set_number(y);
                let value = match function {
                    Some(f) => parser.eval_expr(f)?,
                    None => return Err(crate::vector::null_reference()),
                };
                let z = value.as_value(Items::Argument)?.re();
                *node = Node::new(x, y * d, z);
                y += sy;
            }
            x += sx;
        }
        Ok(n)
    }

    fn get_color(&self, value: f64, lightness: f64) -> (u8, u8, u8) {
        let s = self.base.settings;
        let k = if s.shadows() { 75.0 + 180.0 * lightness } else { 255.0 };
        if s.smooth_scale {
            let (dr, dg, db) = self.get_rgb(value);
            if s.color_scale == ColorScales::Gray {
                let v = dr as u8;
                (v, v, v)
            } else {
                ((k * dr) as u8, (k * dg) as u8, (k * db) as u8)
            }
        } else {
            let mut i = (value * N_COLORS as f64).floor() as i64;
            if i >= N_COLORS as i64 {
                i = N_COLORS as i64 - 1;
            }
            // C# indexes the array with a negative value for value < 0 and throws; the
            // interpolated values are always within [0, 1].
            let c = self.colors[i.max(0) as usize];
            if s.color_scale == ColorScales::Gray {
                let v = c.x as u8;
                (v, v, v)
            } else if s.color_scale == ColorScales::None {
                let v = k as u8;
                (v, v, v)
            } else {
                ((k * c.x) as u8, (k * c.y) as u8, (k * c.z) as u8)
            }
        }
    }

    fn get_color_scale(&mut self) {
        for i in 0..N_COLORS {
            let value = i as f64 / (N_COLORS - 1) as f64;
            let (r, g, b) = self.get_rgb(value);
            self.colors[i] = Node::new(r, g, b);
        }
    }

    fn f(&self, x: f64) -> f64 {
        self.k * x / (0.5 + x * x).sqrt()
    }

    fn get_rgb(&self, value: f64) -> (f64, f64, f64) {
        const D1: f64 = 3.0 / 5.0;
        const D2: f64 = 5.0 / 3.0;
        let mut value = value;
        match self.base.settings.color_scale {
            ColorScales::Gray => {
                let v = 75.0 + 180.0 * value;
                (v, v, v)
            }
            ColorScales::Rainbow => {
                value *= 4.0;
                let n = value.floor() as i32;
                value -= n as f64;
                match n {
                    0 => (0.0, value.powf(D1), 1.0),
                    1 => (0.0, 1.0, 1.0 - value.powf(D2)),
                    2 => (value.powf(D1), 1.0, 0.0),
                    3 => (1.0, 1.0 - value.powf(D2), 0.0),
                    _ => (1.0 - 0.5 * value, 0.0, 0.0),
                }
            }
            ColorScales::Terrain => {
                value *= 3.0;
                let n = value.floor() as i32;
                value -= n as f64;
                match n {
                    0 => (0.0, 0.5 * (1.0 + value), 0.5 * (1.0 - value)),
                    1 => (value, 1.0, 0.0),
                    2 => (1.0, 1.0 - 0.5 * value, 0.0),
                    _ => (1.0, 0.5 * (1.0 - value), 0.0),
                }
            }
            ColorScales::VioletToYellow => (0.6 + 0.4 * value, 0.2 + 0.8 * value, 0.8 - 0.8 * value),
            ColorScales::GreenToYellow => (value, 0.4 + 0.6 * self.f(value), 0.4 - 0.2 * value * value),
            ColorScales::Blues => (0.1 + 0.6 * value, 0.2 + 0.75 * value, 0.8 + 0.2 * value),
            ColorScales::BlueToYellow => {
                let v2 = value * value;
                (0.15 + 0.85 * v2, 0.3 + 0.65 * self.f(value), 0.15 + 0.65 * (1.0 - v2))
            }
            ColorScales::BlueToRed => {
                let r = 0.8 * self.f(value) + 0.2 * (1.0 - value.powf(4.0) - (1.0 - value).powf(4.0));
                let g = value - 0.45;
                let g = 2.0 * (0.4 - g * g);
                (r, g, 1.0 - 0.8 * value * value)
            }
            ColorScales::PurpleToYellow => {
                let v6 = (1.0 - value).powf(6.0);
                (
                    0.4 + 0.6 * value - 0.4 * v6,
                    0.9 * value * value * value,
                    1.2 * (1.0 - value) - 0.6 * v6,
                )
            }
            ColorScales::None => (1.0, 1.0, 1.0),
        }
    }

    fn draw_color_scale_svg(&self, g: &mut SvgDrawing, m: &Map) {
        let b = &self.base;
        let x0 = (b.width - b.right + b.margin / 2) as f64;
        let y0 = (b.height - b.margin) as f64;
        let w = b.margin / 4;
        let h = b.height - 2 * b.margin;
        let mut n = N_COLORS as i32;
        if b.settings.smooth_scale {
            n = h / self.size + 1;
        }
        let mut dh = h as f64 / n as f64;
        let ssf = b.screen_scale_factor as f64;
        let th = 10.0 * ssf;
        let th05 = th / 2.0;
        for i in 0..n {
            let (r, gg, bb) = self.get_color((i as f64 + 0.5) / n as f64, 1.0);
            g.fill_rectangle(
                x0,
                y0 - i as f64 * dh - dh,
                w as f64,
                dh,
                &format!("#{:02x}{:02x}{:02x}", r, gg, bb),
            );
        }
        g.draw_rectangle(x0, b.margin as f64, w as f64, h as f64, "Axis");
        let dy = (m.max - m.min) / N_COLORS as f64;
        dh = h as f64 / N_COLORS as f64;
        let a = (5.0 * b.screen_scale_factor) as f64;
        for i in 0..=N_COLORS {
            let y = y0 - i as f64 * dh;
            g.draw_line(x0, y, x0 + w as f64, y, "Grid");
            let s = b.writer_fmt(m.min + i as f64 * dy);
            g.draw_text(&s, x0 + w as f64 + a, y + th05, "");
        }
    }

    /// `Interpolate(Map m)`: values[i][j] = [scaled z, shadow x, shadow y]
    fn interpolate(&self, m: &Map) -> Vec<Vec<[f64; 3]>> {
        let delta = m.max - m.min;
        let delta = if delta == 0.0 { 1.0 } else { 1.0 / delta };
        let size = self.size as usize;
        let factor = 1.0 / size as f64;
        let (nxs, nys) = (self.nx * size, self.ny * size);
        let mut d = vec![vec![[0.0f64; 3]; nys + 1]; nxs + 1];
        let shadows = self.base.settings.shadows();
        let (mut i0, mut i1) = (0usize, 0usize);
        let mut i = 0;
        while i <= nxs {
            let (mut j0, mut j1) = (0usize, 0usize);
            let mut j = 0;
            while j <= nys {
                d[i][j][0] = (m.points[i1][j1].z - m.min) * delta;
                d[i][j][1] = m.vertices[i1][j1].x;
                d[i][j][2] = m.vertices[i1][j1].y;
                if j > 0 && size > 1 {
                    let mut d0 = d[i][j0][0];
                    let d1 = (d[i][j][0] - d0) * factor;
                    for k in 1..size {
                        d0 += d1;
                        d[i][j0 + k][0] = d0;
                    }
                    if shadows {
                        for p in 1..=2 {
                            let mut d0 = d[i][j0][p];
                            let d1 = (d[i][j][p] - d0) * factor;
                            for k in 1..size {
                                d0 += d1;
                                d[i][j0 + k][p] = d0;
                            }
                        }
                    }
                }
                j0 = j;
                j1 += 1;
                j += size;
            }
            if i > 0 && size > 1 {
                for j in 0..nys {
                    let mut d0 = d[i0][j][0];
                    let d1 = (d0 - d[i][j][0]) * factor;
                    for k in 1..size {
                        d0 -= d1;
                        d[i0 + k][j][0] = d0;
                    }
                    if shadows {
                        for p in 1..=2 {
                            let mut d0 = d[i0][j][p];
                            let d1 = (d0 - d[i][j][p]) * factor;
                            for k in 1..size {
                                d0 -= d1;
                                d[i0 + k][j][p] = d0;
                            }
                        }
                    }
                }
            }
            i0 = i;
            i1 += 1;
            i += size;
        }
        d
    }

    /// `SetBitmapBits(bitmap, values)`: the BGRA bytes of a `w` by `h` bitmap.
    fn bitmap_bits(&self, w: usize, h: usize, values: &[Vec<[f64; 3]>]) -> Vec<u8> {
        let s = self.base.settings;
        let mut light = Node::default();
        let mut spec = Node::default();
        let mut ks = 0.5;
        let shadows = s.shadows();
        if shadows {
            const SQR2: f64 = std::f64::consts::FRAC_1_SQRT_2;
            const SQR3: f64 = 0.577_350_269_189_625_764_509_148_780_501_96;
            if s.color_scale <= ColorScales::Terrain {
                ks = 0.7;
            }
            light = match s.light_direction {
                LightDirections::North => Node::new(0.0, SQR2, SQR2),
                LightDirections::East => Node::new(SQR2, 0.0, SQR2),
                LightDirections::South => Node::new(0.0, -SQR2, SQR2),
                LightDirections::West => Node::new(-SQR2, 0.0, SQR2),
                LightDirections::NorthEast => Node::new(SQR3, SQR3, SQR3),
                LightDirections::SouthEast => Node::new(SQR3, -SQR3, SQR3),
                LightDirections::SouthWest => Node::new(-SQR3, -SQR3, SQR3),
                LightDirections::NorthWest => Node::new(-SQR3, SQR3, SQR3),
            };
            let z = light.z + 1.0;
            let length = (light.x * light.x + light.y * light.y + z * z).sqrt();
            spec = Node::new(light.x / length, light.y / length, z / length);
        }
        let nx = values.len() - 1;
        let ny = values[0].len() - 1;
        let mt = (h as i64 - ny as i64 + 1) / 2;
        let ml = (mt as f64 * self.base.left as f64 / self.base.margin as f64) as i64;
        let mut bytes = vec![0u8; 4 * w * h];
        for i in 0..ny {
            let i_str = (mt + i as i64) * w as i64 + ml;
            for j in 0..nx {
                let n1 = ny - i - 1;
                let d = values[j][n1][0];
                if !d.is_infinite() && !d.is_nan() {
                    let (mut p, mut sv) = (0.0, 0.0);
                    if shadows {
                        let x = values[j][n1][1];
                        let y = values[j][n1][2];
                        let mut length = (x * x + y * y + 1.0).sqrt();
                        p = (x * light.x + y * light.y + light.z) / length;
                        if s.color_scale > ColorScales::Gray {
                            length = (x * spec.x + y * spec.y + spec.z) / length;
                            if length.abs() > 0.98 {
                                sv = length.powf(200.0) * ks;
                            }
                        }
                    }
                    if p < 0.0 {
                        p = 0.0;
                    }
                    let (r, g, b) = self.get_color(d, p);
                    let j_str = (4 * (i_str + j as i64)) as usize;
                    bytes[j_str + 3] = 255;
                    bytes[j_str + 2] = (r as f64 + (255.0 - r as f64) * sv) as u8;
                    bytes[j_str + 1] = (g as f64 + (255.0 - g as f64) * sv) as u8;
                    bytes[j_str] = (b as f64 + (255.0 - b as f64) * sv) as u8;
                }
            }
        }
        bytes
    }

    fn draw_svg(&self, m: &Map, x0: f64, y0: f64, xs: f64, ys: f64, bounds: &PlotBox) -> CResult<String> {
        let b = &self.base;
        let mut g = SvgDrawing::new(b.width as f64, b.height as f64, b.screen_scale_factor as f64, 0)?;
        let values = self.interpolate(m);
        let w = values.len() - 1;
        let h = values[0].len() - 1;
        let bgra = self.bitmap_bits(w, h, &values);
        // The SKBitmap is BGRA (premultiplied; alpha is 0 or 255), the PNG is RGBA.
        let mut rgba = bgra;
        for px in rgba.chunks_mut(4) {
            px.swap(0, 2);
        }
        let png = super::png::encode_rgba(w, h, &rgba);
        let src = format!("data:image/png;base64,{}", super::png::base64(&png));
        g.draw_image(b.left as f64, b.margin as f64, w as f64, h as f64, &src);
        b.draw_grid_svg(&mut g, x0, y0, xs, ys, bounds);
        self.draw_color_scale_svg(&mut g, m);
        let d = 0.75 / b.screen_scale_factor as f64;
        let dw = super::round(d * b.width as f64);
        let dh = super::round(d * b.height as f64);
        let s = g.to_string();
        let svg_text = format!("<svg data-plot=\"svg\" {}", &s[4..]);
        Ok(format!(
            "<div class=\"plot\" style=\"width:{}pt; height:{}pt;\">{}</div>",
            crate::netfmt::double_to_string(dw),
            crate::netfmt::double_to_string(dh),
            svg_text
        ))
    }
}
