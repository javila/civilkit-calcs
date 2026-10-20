//! HTML rendering (Markdig Renderers/HtmlRenderer.cs and Renderers/Html/*, plus the table
//! and task list renderers). `EnsureLine`, `ImplicitParagraph` and the first/last in
//! container flags follow Markdig, so the text nodes come out with the same line breaks.
use super::blocks::{Align, Document, Kind};
use super::chars::{escape_html, escape_url};
use super::inlines::Inline;

pub struct HtmlRenderer {
    pub out: String,
    previous_was_line: bool,
    pub implicit_paragraph: bool,
    enable_html_for_inline: bool,
    is_first_in_container: bool,
    is_last_in_container: bool,
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlRenderer {
    pub fn new() -> Self {
        HtmlRenderer {
            out: String::new(),
            previous_was_line: true,
            implicit_paragraph: false,
            enable_html_for_inline: true,
            is_first_in_container: false,
            is_last_in_container: false,
        }
    }

    fn write(&mut self, s: &str) {
        self.previous_was_line = false;
        self.out.push_str(s);
    }

    fn write_line(&mut self, s: &str) {
        self.out.push_str(s);
        self.out.push('\n');
        self.previous_was_line = true;
    }

    fn ensure_line(&mut self) {
        if !self.previous_was_line {
            self.previous_was_line = true;
            self.out.push('\n');
        }
    }

    fn write_escape(&mut self, s: &str) {
        self.previous_was_line = false;
        escape_html(s, &mut self.out);
    }

    fn write_escape_url(&mut self, s: &str) {
        self.previous_was_line = false;
        escape_url(s, &mut self.out);
    }

    fn write_class_attr(&mut self, classes: &[String]) {
        if !classes.is_empty() {
            self.write(" class=\"");
            let joined = classes.join(" ");
            self.write_escape(&joined);
            self.write("\"");
        }
    }

    fn write_children(&mut self, doc: &Document, block: usize) {
        let children: Vec<usize> = doc.children(block).collect();
        let (sf, sl) = (self.is_first_in_container, self.is_last_in_container);
        for (i, &c) in children.iter().enumerate() {
            self.is_first_in_container = i == 0;
            self.is_last_in_container = i + 1 == children.len();
            self.render_block(doc, c);
        }
        self.is_first_in_container = sf;
        self.is_last_in_container = sl;
    }

    pub fn render_block(&mut self, doc: &Document, b: usize) {
        let node = &doc.nodes[b];
        match &node.kind {
            Kind::Document => self.write_children(doc, b),
            Kind::Paragraph => {
                if !self.implicit_paragraph {
                    if !self.is_first_in_container {
                        self.ensure_line();
                    }
                    self.write("<p>");
                }
                self.write_inlines(&node.inlines);
                if !self.implicit_paragraph {
                    self.write_line("</p>");
                    self.ensure_line();
                }
            }
            Kind::Heading(level) => {
                self.write(&format!("<h{}>", level));
                self.write_inlines(&node.inlines);
                self.write_line(&format!("</h{}>", level));
                self.ensure_line();
            }
            Kind::ThematicBreak => self.write_line("<hr />"),
            Kind::BlockQuote => {
                self.ensure_line();
                self.write_line("<blockquote>");
                let saved = self.implicit_paragraph;
                self.implicit_paragraph = false;
                self.write_children(doc, b);
                self.implicit_paragraph = saved;
                self.write_line("</blockquote>");
                self.ensure_line();
            }
            Kind::List { data, tight } => {
                self.ensure_line();
                if data.ordered {
                    self.write("<ol");
                    if data.bullet_type != '1' {
                        self.write(&format!(" type=\"{}\"", data.bullet_type));
                    }
                    if data.ordered_start != "1" {
                        self.write(&format!(" start=\"{}\"", data.ordered_start));
                    }
                    self.write_class_attr(&node.classes);
                    self.write_line(">");
                } else {
                    self.write("<ul");
                    self.write_class_attr(&node.classes);
                    self.write_line(">");
                }
                let items: Vec<usize> = doc.children(b).collect();
                for item in items {
                    let previous_implicit = self.implicit_paragraph;
                    self.implicit_paragraph = *tight;
                    self.ensure_line();
                    self.write("<li");
                    let classes = doc.nodes[item].classes.clone();
                    self.write_class_attr(&classes);
                    self.write(">");
                    self.write_children(doc, item);
                    self.write_line("</li>");
                    self.ensure_line();
                    self.implicit_paragraph = previous_implicit;
                }
                self.write_line(if data.ordered { "</ol>" } else { "</ul>" });
                self.ensure_line();
            }
            Kind::Item(_) => self.write_children(doc, b),
            Kind::CodeBlock { info, .. } => {
                self.ensure_line();
                self.write("<pre><code");
                let lang = info.split_whitespace().next().unwrap_or("");
                if !lang.is_empty() {
                    self.write(" class=\"");
                    self.write_escape(&format!("language-{}", lang));
                    self.write("\"");
                }
                self.write(">");
                let content = node.content.clone();
                for line in content.split_inclusive('\n') {
                    self.write_escape(line.trim_end_matches('\n'));
                    self.write_line("");
                }
                self.write_line("</code></pre>");
                self.ensure_line();
            }
            Kind::HtmlBlock(_) => {
                let content = node.content.clone();
                for line in content.split_inclusive('\n') {
                    self.write(line.trim_end_matches('\n'));
                    self.write_line("");
                }
            }
            Kind::Table(t) => {
                self.ensure_line();
                self.write_line("<table>");
                let mut has_body = false;
                let mut is_header_open = false;
                for (r, row) in t.cells.iter().enumerate() {
                    let header = r == 0;
                    if header {
                        self.write_line("<thead>");
                        is_header_open = true;
                    } else if !has_body {
                        if is_header_open {
                            self.write_line("</thead>");
                            is_header_open = false;
                        }
                        self.write_line("<tbody>");
                        has_body = true;
                    }
                    self.write_line("<tr>");
                    for (i, cell) in row.iter().enumerate() {
                        self.ensure_line();
                        self.write(if header { "<th" } else { "<td" });
                        if !t.aligns.is_empty() {
                            let ci = i.min(t.aligns.len() - 1);
                            match t.aligns[ci] {
                                Align::Center => self.write(" style=\"text-align: center;\""),
                                Align::Right => self.write(" style=\"text-align: right;\""),
                                Align::Left => self.write(" style=\"text-align: left;\""),
                                Align::None => {}
                            }
                        }
                        self.write(">");
                        self.write_inlines(cell);
                        self.write_line(if header { "</th>" } else { "</td>" });
                    }
                    self.write_line("</tr>");
                }
                if has_body {
                    self.write_line("</tbody>");
                } else if is_header_open {
                    self.write_line("</thead>");
                }
                self.write_line("</table>");
            }
        }
    }

    fn write_inlines(&mut self, inlines: &[Inline]) {
        let (sf, sl) = (self.is_first_in_container, self.is_last_in_container);
        for (i, inl) in inlines.iter().enumerate() {
            self.is_first_in_container = i == 0;
            self.is_last_in_container = i + 1 == inlines.len();
            self.write_inline(inl);
        }
        self.is_first_in_container = sf;
        self.is_last_in_container = sl;
    }

    fn write_inline(&mut self, inl: &Inline) {
        match inl {
            Inline::Text(t) => self.write_escape(t),
            Inline::Code(t) => {
                if self.enable_html_for_inline {
                    self.write("<code>");
                }
                self.write_escape(t);
                if self.enable_html_for_inline {
                    self.write("</code>");
                }
            }
            Inline::Html(t) => {
                if self.enable_html_for_inline {
                    self.write(t);
                }
            }
            Inline::SoftBreak | Inline::HardBreak => {
                if self.is_last_in_container {
                    return;
                }
                if self.enable_html_for_inline && *inl == Inline::HardBreak {
                    self.write_line("<br />");
                }
                self.ensure_line();
            }
            Inline::Emph { ch, count, children } => {
                let tag = match (ch, count) {
                    ('~', 2) => "del",
                    ('~', _) => "sub",
                    ('^', _) => "sup",
                    ('+', _) => "ins",
                    ('=', _) => "mark",
                    (_, 2) => "strong",
                    _ => "em",
                };
                if self.enable_html_for_inline {
                    self.write(&format!("<{}>", tag));
                }
                self.write_inlines(children);
                if self.enable_html_for_inline {
                    self.write(&format!("</{}>", tag));
                }
            }
            Inline::Link {
                url,
                title,
                image,
                children,
            } => {
                if self.enable_html_for_inline {
                    self.write(if *image { "<img src=\"" } else { "<a href=\"" });
                    self.write_escape_url(url);
                    self.write("\"");
                }
                if *image {
                    if self.enable_html_for_inline {
                        self.write(" alt=\"");
                    }
                    let was = self.enable_html_for_inline;
                    self.enable_html_for_inline = false;
                    self.write_inlines(children);
                    self.enable_html_for_inline = was;
                    if self.enable_html_for_inline {
                        self.write("\"");
                    }
                }
                if self.enable_html_for_inline && !title.is_empty() {
                    self.write(" title=\"");
                    self.write_escape(title);
                    self.write("\"");
                }
                if *image {
                    if self.enable_html_for_inline {
                        self.write(" />");
                    }
                } else {
                    if self.enable_html_for_inline {
                        self.write(">");
                    }
                    self.write_inlines(children);
                    if self.enable_html_for_inline {
                        self.write("</a>");
                    }
                }
            }
            Inline::AutoLink { url, email } => {
                if self.enable_html_for_inline {
                    self.write(if *email { "<a href=\"mailto:" } else { "<a href=\"" });
                    self.write_escape_url(url);
                    self.write("\">");
                }
                self.write_escape(url);
                if self.enable_html_for_inline {
                    self.write("</a>");
                }
            }
            Inline::TaskList(checked) => {
                if self.enable_html_for_inline {
                    self.write("<input disabled=\"disabled\" type=\"checkbox\"");
                    if *checked {
                        self.write(" checked=\"checked\"");
                    }
                    self.write(" />");
                } else {
                    self.write(if *checked { "[x]" } else { "[ ]" });
                }
            }
        }
    }
}
