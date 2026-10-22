//! Native Excel charts (DrawingML chart parts) for the workbook export: an XY scatter chart
//! with straight lines, drawn from the data of a `$Plot`. The element order follows the
//! ECMA-376 schema (CT_ChartSpace, CT_Chart, CT_PlotArea, CT_ScatterChart, CT_ScatterSer,
//! CT_ValAx), because Excel asks to repair a part whose children are out of order.
use calcpad_core::openxml::xml::escape;

/// One curve; the values live in the data sheet and are cached here.
pub struct ChartSeries {
    pub name: String,
    /// `RRGGBB`
    pub color: String,
    /// Absolute references such as `'Plot data'!$A$3:$A$514`.
    pub x_ref: String,
    pub y_ref: String,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    /// A single point, drawn as a marker instead of a line.
    pub dot: bool,
}

pub struct Chart {
    /// Zero-based column and row of the top left corner.
    pub col: u32,
    pub row: u32,
    /// Size in pixels as shown.
    pub width: u32,
    pub height: u32,
    pub name: String,
    pub x_title: String,
    pub y_title: String,
    pub series: Vec<ChartSeries>,
    /// Axis limits as the plot drew them: left, bottom, right, top.
    pub bounds: (f64, f64, f64, f64),
}

fn num(n: f64) -> String {
    if n == n.trunc() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{:?}", n)
    }
}

fn rich_title(text: &str, size: u32) -> String {
    format!(
        "<c:title><c:tx><c:rich><a:bodyPr/><a:lstStyle/><a:p><a:pPr><a:defRPr sz=\"{sz}\" b=\"0\"/></a:pPr><a:r><a:rPr lang=\"en-US\" sz=\"{sz}\" b=\"0\"/><a:t>{}</a:t></a:r></a:p></c:rich></c:tx><c:overlay val=\"0\"/></c:title>",
        escape(text, false),
        sz = size
    )
}

fn num_ref(f: &str, values: &[f64]) -> String {
    let mut x = format!(
        "<c:numRef><c:f>{}</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val=\"{}\"/>",
        escape(f, false),
        values.len()
    );
    for (i, v) in values.iter().enumerate() {
        x.push_str(&format!("<c:pt idx=\"{}\"><c:v>{}</c:v></c:pt>", i, num(*v)));
    }
    x.push_str("</c:numCache></c:numRef>");
    x
}

/// A limit without the float noise of the plotter's arithmetic: zero when it is nothing next
/// to the span (1.2e-17 beside a span of 1), otherwise ten significant digits.
fn clean(v: f64, span: f64) -> f64 {
    if v.abs() < span.abs() * 1e-9 {
        return 0.0;
    }
    format!("{:.9e}", v).parse().unwrap_or(v)
}

/// A major unit of 1, 2, 2.5 or 5 times a power of ten (the steps the plot's own grid uses) that
/// gives at most ten divisions.
fn major_unit(span: f64) -> f64 {
    if !(span.is_finite() && span > 0.0) {
        return 1.0;
    }
    let base = 10f64.powf((span / 10.0).log10().floor());
    for k in [1.0, 2.0, 2.5, 5.0, 10.0] {
        if span / (base * k) <= 10.0 {
            return clean(base * k, span);
        }
    }
    base * 10.0
}

fn val_ax(id: u32, cross: u32, pos: &str, title: &str, min: f64, max: f64) -> String {
    let span = max - min;
    let (min, max) = (clean(min, span), clean(max, span));
    let unit = major_unit(max - min);
    // CT_ValAx: axId, scaling, delete, axPos, majorGridlines, title, numFmt, majorTickMark,
    // minorTickMark, tickLblPos, spPr, crossAx, crosses, crossBetween.
    let mut x = format!(
        "<c:valAx><c:axId val=\"{}\"/><c:scaling><c:orientation val=\"minMax\"/><c:max val=\"{}\"/><c:min val=\"{}\"/></c:scaling><c:delete val=\"0\"/><c:axPos val=\"{}\"/><c:majorGridlines><c:spPr><a:ln w=\"6350\"><a:solidFill><a:srgbClr val=\"D9D9D9\"/></a:solidFill></a:ln></c:spPr></c:majorGridlines>",
        id,
        num(max),
        num(min),
        pos
    );
    if !title.is_empty() {
        x.push_str(&rich_title(title, 1000));
    }
    x.push_str(&format!(
        "<c:numFmt formatCode=\"General\" sourceLinked=\"0\"/><c:majorTickMark val=\"out\"/><c:minorTickMark val=\"none\"/><c:tickLblPos val=\"low\"/><c:spPr><a:ln w=\"9525\"><a:solidFill><a:srgbClr val=\"595959\"/></a:solidFill></a:ln></c:spPr><c:crossAx val=\"{}\"/><c:crosses val=\"autoZero\"/><c:crossBetween val=\"midCat\"/><c:majorUnit val=\"{}\"/></c:valAx>",
        cross,
        num(unit)
    ));
    x
}

/// The chart part (`xl/charts/chartN.xml`).
pub fn chart_xml(c: &Chart) -> String {
    let mut x = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><c:roundedCorners val=\"0\"/><c:chart><c:autoTitleDeleted val=\"1\"/><c:plotArea><c:layout/><c:scatterChart><c:scatterStyle val=\"lineMarker\"/><c:varyColors val=\"0\"/>",
    );
    for (i, s) in c.series.iter().enumerate() {
        // CT_ScatterSer: idx, order, tx, spPr, marker, xVal, yVal, smooth.
        x.push_str(&format!(
            "<c:ser><c:idx val=\"{i}\"/><c:order val=\"{i}\"/><c:tx><c:v>{}</c:v></c:tx>",
            escape(&s.name, false),
            i = i
        ));
        if s.dot {
            x.push_str(&format!(
                "<c:spPr><a:ln w=\"19050\"><a:noFill/></a:ln></c:spPr><c:marker><c:symbol val=\"circle\"/><c:size val=\"7\"/><c:spPr><a:solidFill><a:srgbClr val=\"{c}\"/></a:solidFill><a:ln><a:solidFill><a:srgbClr val=\"{c}\"/></a:solidFill></a:ln></c:spPr></c:marker>",
                c = s.color
            ));
        } else {
            x.push_str(&format!(
                "<c:spPr><a:ln w=\"28575\" cap=\"rnd\"><a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill><a:round/></a:ln></c:spPr><c:marker><c:symbol val=\"none\"/></c:marker>",
                s.color
            ));
        }
        x.push_str("<c:xVal>");
        x.push_str(&num_ref(&s.x_ref, &s.xs));
        x.push_str("</c:xVal><c:yVal>");
        x.push_str(&num_ref(&s.y_ref, &s.ys));
        x.push_str("</c:yVal><c:smooth val=\"0\"/></c:ser>");
    }
    x.push_str("<c:axId val=\"500000001\"/><c:axId val=\"500000002\"/></c:scatterChart>");
    let (l, b, r, t) = c.bounds;
    x.push_str(&val_ax(500000001, 500000002, "b", &c.x_title, l, r));
    x.push_str(&val_ax(500000002, 500000001, "l", &c.y_title, b, t));
    x.push_str("<c:spPr><a:noFill/></c:spPr></c:plotArea>");
    if c.series.len() > 1 {
        x.push_str("<c:legend><c:legendPos val=\"b\"/><c:overlay val=\"0\"/></c:legend>");
    }
    x.push_str("<c:plotVisOnly val=\"1\"/><c:dispBlanksAs val=\"gap\"/></c:chart><c:spPr><a:solidFill><a:srgbClr val=\"FFFFFF\"/></a:solidFill><a:ln w=\"9525\"><a:solidFill><a:srgbClr val=\"D9D9D9\"/></a:solidFill></a:ln></c:spPr></c:chartSpace>");
    x
}

/// The `xdr:oneCellAnchor` that places chart part `rid` in a drawing.
pub fn anchor_xml(c: &Chart, rid: usize, shape_id: usize) -> String {
    let (cx, cy) = (c.width as u64 * 9525, c.height as u64 * 9525);
    format!(
        "<xdr:oneCellAnchor><xdr:from><xdr:col>{}</xdr:col><xdr:colOff>0</xdr:colOff><xdr:row>{}</xdr:row><xdr:rowOff>0</xdr:rowOff></xdr:from><xdr:ext cx=\"{}\" cy=\"{}\"/><xdr:graphicFrame macro=\"\"><xdr:nvGraphicFramePr><xdr:cNvPr id=\"{}\" name=\"{}\"/><xdr:cNvGraphicFramePr/></xdr:nvGraphicFramePr><xdr:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{}\" cy=\"{}\"/></xdr:xfrm><a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/chart\"><c:chart xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" r:id=\"rId{}\"/></a:graphicData></a:graphic></xdr:graphicFrame><xdr:clientData/></xdr:oneCellAnchor>",
        c.col,
        c.row,
        cx,
        cy,
        shape_id,
        escape(&c.name, true),
        cx,
        cy,
        rid
    )
}
