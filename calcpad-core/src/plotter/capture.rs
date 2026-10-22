//! The data of a `$Plot`, recorded for the Excel export (calcpad-rs's own feature): the series
//! the plotter calculated, with the axis limits, the units and the colours of the SVG, so that
//! the workbook can hold an editable chart. `$Map` colour maps have no such data.

/// One plotted curve.
#[derive(Clone, Debug, PartialEq)]
pub struct CapturedSeries {
    /// The expression as written (`-M(x)`, or `x(t) | y(t)` for a parametric curve).
    pub name: String,
    /// `RRGGBB`, the colour of the SVG's `SeriesN` class.
    pub color: String,
    pub points: Vec<(f64, f64)>,
    /// The curve is a single point, drawn as a dot.
    pub dot: bool,
    /// The SVG fills the area under the curve (`y = f(x)` plots); a chart cannot.
    pub fill: bool,
}

/// A `$Plot` as the plotter drew it.
#[derive(Clone, Debug, PartialEq)]
pub struct CapturedChart {
    /// The name of the plot variable.
    pub x_name: String,
    pub x_unit: String,
    pub y_unit: String,
    pub series: Vec<CapturedSeries>,
    /// The plotted limits: left, bottom, right, top.
    pub bounds: (f64, f64, f64, f64),
}

/// The colours of `Series1` to `Series10` (SvgDrawing.AddStyle) as `RRGGBB`.
pub const SERIES_COLORS: [&str; 10] = [
    "FF6347", "9ACD32", "6495ED", "F0D000", "C71585", "00FA9A", "8A2BE2", "FFA07A", "FF1493", "00CED1",
];

impl CapturedChart {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "xName": self.x_name,
            "xUnit": self.x_unit,
            "yUnit": self.y_unit,
            "bounds": [self.bounds.0, self.bounds.1, self.bounds.2, self.bounds.3],
            "series": self.series.iter().map(|s| serde_json::json!({
                "name": s.name,
                "color": s.color,
                "dot": s.dot,
                "fill": s.fill,
                "points": s.points.iter().map(|p| [p.0, p.1]).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })
    }

    pub fn from_json(v: &serde_json::Value) -> Option<CapturedChart> {
        let text = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
        let b: Vec<f64> = v.get("bounds")?.as_array()?.iter().filter_map(|x| x.as_f64()).collect();
        if b.len() != 4 {
            return None;
        }
        let series = v
            .get("series")?
            .as_array()?
            .iter()
            .map(|s| {
                Some(CapturedSeries {
                    name: s.get("name")?.as_str()?.to_string(),
                    color: s.get("color")?.as_str()?.to_string(),
                    dot: s.get("dot").and_then(|x| x.as_bool()).unwrap_or(false),
                    fill: s.get("fill").and_then(|x| x.as_bool()).unwrap_or(false),
                    points: s
                        .get("points")?
                        .as_array()?
                        .iter()
                        .filter_map(|p| {
                            let p = p.as_array()?;
                            Some((p.first()?.as_f64()?, p.get(1)?.as_f64()?))
                        })
                        .collect(),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(CapturedChart {
            x_name: text("xName"),
            x_unit: text("xUnit"),
            y_unit: text("yUnit"),
            series,
            bounds: (b[0], b[1], b[2], b[3]),
        })
    }
}

pub fn charts_to_json(charts: &[CapturedChart]) -> serde_json::Value {
    serde_json::Value::Array(charts.iter().map(|c| c.to_json()).collect())
}

pub fn charts_from_json(v: &serde_json::Value) -> Vec<CapturedChart> {
    v.as_array()
        .map(|a| a.iter().filter_map(CapturedChart::from_json).collect())
        .unwrap_or_default()
}
