//! A port of the subset of Markdig 0.43.0 that CalcpadCE uses: CommonMark blocks and
//! inlines, rendered to HTML, with the emphasis extras (`~~del~~`, `~sub~`, `^sup^`,
//! `++ins++`, `==mark==`), list extras (`a.`, `A.`, `i.`, `I.` lists), pipe tables, task
//! lists and bare URL auto links. Markdig is Copyright (c) 2018-2019, Alexandre Mutel,
//! BSD 2-Clause licence (see NOTICE).
pub mod blocks;
pub mod chars;
pub mod html;
pub mod inlines;
pub mod table;

use blocks::{Document, Kind};

/// The extensions of a `MarkdownPipeline`.
#[derive(Clone, Debug, Default)]
pub struct Pipeline {
    pub emphasis_extras: bool,
    pub list_extras: bool,
    pub pipe_tables: bool,
    pub task_lists: bool,
    pub auto_links: bool,
}

impl Pipeline {
    /// `new MarkdownPipelineBuilder().UseEmphasisExtras().UseListExtras().Build()`,
    /// used for `#md on` comments.
    pub fn comments() -> Self {
        Pipeline {
            emphasis_extras: true,
            list_extras: true,
            ..Default::default()
        }
    }

    /// The pipeline of `#markdown` blocks (ExpressionParser.ParseModes.cs).
    pub fn markdown_mode() -> Self {
        Pipeline {
            emphasis_extras: true,
            list_extras: true,
            pipe_tables: true,
            task_lists: true,
            auto_links: true,
        }
    }
}

fn contains_task(inlines: &[inlines::Inline]) -> bool {
    inlines.iter().any(|i| match i {
        inlines::Inline::TaskList(_) => true,
        inlines::Inline::Emph { children, .. } | inlines::Inline::Link { children, .. } => contains_task(children),
        _ => false,
    })
}

/// `Markdown.Parse(text, pipeline)`: block structure plus inline content.
pub fn parse(text: &str, pipeline: &Pipeline) -> Document {
    let mut doc = blocks::parse_blocks(text, pipeline);
    let refmap = doc.refmap.clone();
    for i in 0..doc.nodes.len() {
        if doc.nodes[i].removed {
            continue;
        }
        let kind = doc.nodes[i].kind.clone();
        match kind {
            Kind::Paragraph | Kind::Heading(_) => {
                let parent = doc.nodes[i].parent;
                let in_item = parent.is_some_and(|p| matches!(doc.nodes[p].kind, Kind::Item(_)));
                let content = std::mem::take(&mut doc.nodes[i].content);
                let inl = inlines::parse_inlines(&content, &refmap, pipeline, in_item);
                if in_item && contains_task(&inl) {
                    let item = parent.unwrap();
                    if !doc.nodes[item].classes.iter().any(|c| c == "task-list-item") {
                        doc.nodes[item].classes.push("task-list-item".to_string());
                    }
                    let list = doc.nodes[item].parent.unwrap();
                    if !doc.nodes[list].classes.iter().any(|c| c == "contains-task-list") {
                        doc.nodes[list].classes.push("contains-task-list".to_string());
                    }
                }
                doc.nodes[i].inlines = inl;
            }
            Kind::Table(mut t) => {
                t.cells = t
                    .rows
                    .iter()
                    .map(|r| {
                        r.iter()
                            .map(|c| inlines::parse_inlines(c, &refmap, pipeline, false))
                            .collect()
                    })
                    .collect();
                doc.nodes[i].kind = Kind::Table(t);
            }
            _ => {}
        }
    }
    doc
}

/// Renders a whole document, as `HtmlRenderer.Render(document)`.
pub fn render(doc: &Document, implicit_paragraph: bool) -> String {
    let mut r = html::HtmlRenderer::new();
    r.implicit_paragraph = implicit_paragraph;
    r.render_block(doc, doc.root());
    r.out
}

/// Renders each top level block separately, as `FlushMarkdown` does, returning the HTML
/// of every block with the zero based line it starts on.
pub fn render_blocks(doc: &Document) -> Vec<(usize, String)> {
    let mut r = html::HtmlRenderer::new();
    let mut result = Vec::new();
    let children: Vec<usize> = doc.children(doc.root()).collect();
    for c in children {
        r.render_block(doc, c);
        let html = std::mem::take(&mut r.out);
        result.push((doc.nodes[c].start_line, html));
    }
    result
}

/// Markdown to HTML with a whole document render (used by tests and examples).
pub fn to_html(text: &str, pipeline: &Pipeline) -> String {
    render(&parse(text, pipeline), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn md(s: &str) -> String {
        to_html(s, &Pipeline::markdown_mode())
    }

    #[test]
    fn basic_blocks() {
        assert_eq!(md("# Title\n\nparagraph"), "<h1>Title</h1>\n<p>paragraph</p>\n");
        assert_eq!(md("---"), "<hr />\n");
        assert_eq!(md("> quote"), "<blockquote>\n<p>quote</p>\n</blockquote>\n");
        assert_eq!(
            md("```text\na\n```"),
            "<pre><code class=\"language-text\">a\n</code></pre>\n"
        );
    }

    #[test]
    fn lists() {
        assert_eq!(
            md("- one\n  - two\n- three"),
            "<ul>\n<li>one\n<ul>\n<li>two</li>\n</ul>\n</li>\n<li>three</li>\n</ul>\n"
        );
        assert_eq!(md("1. a\n2. b"), "<ol>\n<li>a</li>\n<li>b</li>\n</ol>\n");
        assert_eq!(
            md("- a\n\n- b"),
            "<ul>\n<li><p>a</p>\n</li>\n<li><p>b</p>\n</li>\n</ul>\n"
        );
        assert_eq!(md("a. x\nb. y"), "<ol type=\"a\">\n<li>x</li>\n<li>y</li>\n</ol>\n");
    }

    #[test]
    fn emphasis() {
        assert_eq!(md("***bi***"), "<p><em><strong>bi</strong></em></p>\n");
        assert_eq!(
            md("++ins++ H~2~O x^2^ ~~del~~ ==m=="),
            "<p><ins>ins</ins> H<sub>2</sub>O x<sup>2</sup> <del>del</del> <mark>m</mark></p>\n"
        );
        assert_eq!(md("*a **b** c*"), "<p><em>a <strong>b</strong> c</em></p>\n");
        assert_eq!(md("a_b_c"), "<p>a_b_c</p>\n");
    }

    #[test]
    fn links_and_code() {
        assert_eq!(md("`c = a + b`"), "<p><code>c = a + b</code></p>\n");
        assert_eq!(
            md("[t](http://x.org \"T\")"),
            "<p><a href=\"http://x.org\" title=\"T\">t</a></p>\n"
        );
        assert_eq!(md("![alt *x*](a.svg)"), "<p><img src=\"a.svg\" alt=\"alt x\" /></p>\n");
        assert_eq!(
            md("<https://a.org>"),
            "<p><a href=\"https://a.org\">https://a.org</a></p>\n"
        );
        assert_eq!(
            md("see https://example.com."),
            "<p>see <a href=\"https://example.com\">https://example.com</a>.</p>\n"
        );
    }

    #[test]
    fn tables_and_tasks() {
        assert_eq!(
            md("| a | b |\n|---|:-:|\n| 1 | 2 |"),
            "<table>\n<thead>\n<tr>\n<th>a</th>\n<th style=\"text-align: center;\">b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>1</td>\n<td style=\"text-align: center;\">2</td>\n</tr>\n</tbody>\n</table>\n"
        );
        assert_eq!(
            md("- [x] done\n- [ ] todo"),
            "<ul class=\"contains-task-list\">\n<li class=\"task-list-item\"><input disabled=\"disabled\" type=\"checkbox\" checked=\"checked\" /> done</li>\n<li class=\"task-list-item\"><input disabled=\"disabled\" type=\"checkbox\" /> todo</li>\n</ul>\n"
        );
    }

    #[test]
    fn html_blocks() {
        assert_eq!(md("<div>\n*x*\n</div>"), "<div>\n*x*\n</div>\n");
        assert_eq!(md("<!-- c -->"), "<!-- c -->\n");
    }
}
