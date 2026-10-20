//! Block structure parsing (Markdig Parsers/BlockProcessor.cs and the block parsers it
//! drives). The algorithm is the CommonMark one: each line is matched against the open
//! container blocks, then new block starts are tried, then the remaining text is added to
//! the tip (or lazily to an open paragraph).
use super::chars::*;
use super::inlines::Inline;
use super::Pipeline;

pub const CODE_INDENT: usize = 4;

#[derive(Clone, Debug, PartialEq)]
pub struct ListData {
    pub ordered: bool,
    /// '-', '*', '+' for bullet lists, '1', 'a', 'A', 'i', 'I' for ordered ones
    pub bullet_type: char,
    pub delimiter: char,
    pub ordered_start: String,
    pub padding: usize,
    pub marker_offset: usize,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Align {
    None,
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug)]
pub struct TableData {
    pub aligns: Vec<Align>,
    /// Cell texts, first row is the header
    pub rows: Vec<Vec<String>>,
    pub cells: Vec<Vec<Vec<Inline>>>,
}

#[derive(Clone, Debug)]
pub enum Kind {
    Document,
    BlockQuote,
    List {
        data: ListData,
        tight: bool,
    },
    Item(ListData),
    Paragraph,
    Heading(u8),
    ThematicBreak,
    CodeBlock {
        fenced: bool,
        fence_char: char,
        fence_length: usize,
        fence_offset: usize,
        info: String,
    },
    HtmlBlock(u8),
    Table(TableData),
}

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: Kind,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub open: bool,
    /// zero based line of the first line of the block
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub inlines: Vec<Inline>,
    pub classes: Vec<String>,
    pub last_line_blank: bool,
    pub removed: bool,
}

pub struct Document {
    pub nodes: Vec<Node>,
    pub refmap: std::collections::HashMap<String, (String, String)>,
}

impl Document {
    pub fn root(&self) -> usize {
        0
    }

    pub fn children(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        self.nodes[i]
            .children
            .iter()
            .copied()
            .filter(move |&c| !self.nodes[c].removed)
    }
}

struct Parser<'a> {
    doc: Document,
    pipeline: &'a Pipeline,
    line: Vec<char>,
    line_number: usize,
    offset: usize,
    column: usize,
    next_nonspace: usize,
    next_nonspace_column: usize,
    indent: usize,
    indented: bool,
    blank: bool,
    partially_consumed_tab: bool,
    all_closed: bool,
    last_matched_container: usize,
    tip: usize,
    oldtip: usize,
}

fn can_contain(parent: &Kind, child: &Kind) -> bool {
    match parent {
        Kind::Document | Kind::BlockQuote | Kind::Item(_) => !matches!(child, Kind::Item(_)),
        Kind::List { .. } => matches!(child, Kind::Item(_)),
        _ => false,
    }
}

fn accepts_lines(k: &Kind) -> bool {
    matches!(k, Kind::Paragraph | Kind::CodeBlock { .. } | Kind::HtmlBlock(_))
}

fn is_paragraph(k: &Kind) -> bool {
    matches!(k, Kind::Paragraph)
}

impl<'a> Parser<'a> {
    fn new(pipeline: &'a Pipeline) -> Self {
        let root = Node {
            kind: Kind::Document,
            parent: None,
            children: Vec::new(),
            open: true,
            start_line: 0,
            end_line: 0,
            content: String::new(),
            inlines: Vec::new(),
            classes: Vec::new(),
            last_line_blank: false,
            removed: false,
        };
        Parser {
            doc: Document {
                nodes: vec![root],
                refmap: Default::default(),
            },
            pipeline,
            line: Vec::new(),
            line_number: 0,
            offset: 0,
            column: 0,
            next_nonspace: 0,
            next_nonspace_column: 0,
            indent: 0,
            indented: false,
            blank: false,
            partially_consumed_tab: false,
            all_closed: true,
            last_matched_container: 0,
            tip: 0,
            oldtip: 0,
        }
    }

    fn peek(&self, i: usize) -> Option<char> {
        self.line.get(i).copied()
    }

    fn last_child(&self, i: usize) -> Option<usize> {
        self.doc.nodes[i]
            .children
            .iter()
            .rev()
            .copied()
            .find(|&c| !self.doc.nodes[c].removed)
    }

    fn find_next_nonspace(&mut self) {
        let mut i = self.offset;
        let mut cols = self.column;
        let mut c = None;
        while let Some(ch) = self.peek(i) {
            if ch == ' ' {
                i += 1;
                cols += 1;
            } else if ch == '\t' {
                i += 1;
                cols += 4 - (cols % 4);
            } else {
                c = Some(ch);
                break;
            }
        }
        self.blank = matches!(c, None | Some('\n') | Some('\r'));
        self.next_nonspace = i;
        self.next_nonspace_column = cols;
        self.indent = self.next_nonspace_column - self.column;
        self.indented = self.indent >= CODE_INDENT;
    }

    fn advance_next_nonspace(&mut self) {
        self.offset = self.next_nonspace;
        self.column = self.next_nonspace_column;
        self.partially_consumed_tab = false;
    }

    fn advance_offset(&mut self, mut count: usize, columns: bool) {
        while count > 0 {
            let Some(c) = self.peek(self.offset) else { break };
            if c == '\t' {
                let chars_to_tab = 4 - (self.column % 4);
                if columns {
                    self.partially_consumed_tab = chars_to_tab > count;
                    let chars_to_advance = chars_to_tab.min(count);
                    self.column += chars_to_advance;
                    if !self.partially_consumed_tab {
                        self.offset += 1;
                    }
                    count -= chars_to_advance;
                } else {
                    self.partially_consumed_tab = false;
                    self.column += chars_to_tab;
                    self.offset += 1;
                    count -= 1;
                }
            } else {
                self.partially_consumed_tab = false;
                self.offset += 1;
                self.column += 1;
                count -= 1;
            }
        }
    }

    fn add_line(&mut self) {
        if self.partially_consumed_tab {
            self.offset += 1;
            let chars_to_tab = 4 - (self.column % 4);
            let tip = self.tip;
            self.doc.nodes[tip].content.push_str(&" ".repeat(chars_to_tab));
        }
        let rest: String = self.line[self.offset.min(self.line.len())..].iter().collect();
        let tip = self.tip;
        self.doc.nodes[tip].content.push_str(&rest);
        self.doc.nodes[tip].content.push('\n');
    }

    fn add_child(&mut self, kind: Kind) -> usize {
        while !can_contain(&self.doc.nodes[self.tip].kind, &kind) {
            let t = self.tip;
            self.finalize(t, self.line_number.saturating_sub(1));
        }
        let idx = self.doc.nodes.len();
        self.doc.nodes.push(Node {
            kind,
            parent: Some(self.tip),
            children: Vec::new(),
            open: true,
            start_line: self.line_number,
            end_line: self.line_number,
            content: String::new(),
            inlines: Vec::new(),
            classes: Vec::new(),
            last_line_blank: false,
            removed: false,
        });
        let tip = self.tip;
        self.doc.nodes[tip].children.push(idx);
        self.tip = idx;
        idx
    }

    fn close_unmatched_blocks(&mut self) {
        if !self.all_closed {
            while self.oldtip != self.last_matched_container {
                let parent = self.doc.nodes[self.oldtip].parent.unwrap();
                let o = self.oldtip;
                self.finalize(o, self.line_number.saturating_sub(1));
                self.oldtip = parent;
            }
            self.all_closed = true;
        }
    }

    fn finalize(&mut self, block: usize, line_number: usize) {
        let above = self.doc.nodes[block].parent;
        self.doc.nodes[block].open = false;
        self.doc.nodes[block].end_line = line_number;
        let kind = self.doc.nodes[block].kind.clone();
        match kind {
            Kind::Paragraph => self.finalize_paragraph(block),
            Kind::CodeBlock {
                fenced,
                fence_char,
                fence_length,
                fence_offset,
                ..
            } => {
                let content = std::mem::take(&mut self.doc.nodes[block].content);
                if fenced {
                    let (first, rest) = match content.find('\n') {
                        Some(p) => (&content[..p], &content[p + 1..]),
                        None => (&content[..], ""),
                    };
                    let info = super::inlines::unescape_string(first.trim());
                    self.doc.nodes[block].kind = Kind::CodeBlock {
                        fenced,
                        fence_char,
                        fence_length,
                        fence_offset,
                        info,
                    };
                    self.doc.nodes[block].content = rest.to_string();
                } else {
                    // strip trailing blank lines
                    let mut lines: Vec<&str> = content.split('\n').collect();
                    if lines.last() == Some(&"") {
                        lines.pop();
                    }
                    while lines.last().is_some_and(|l| l.chars().all(|c| c == ' ')) {
                        lines.pop();
                    }
                    let mut s = lines.join("\n");
                    if !s.is_empty() {
                        s.push('\n');
                    }
                    self.doc.nodes[block].content = s;
                }
            }
            Kind::Item(_) => {
                let last = self.last_child(block);
                if let Some(l) = last {
                    self.doc.nodes[block].end_line = self.doc.nodes[l].end_line;
                } else {
                    self.doc.nodes[block].end_line = self.doc.nodes[block].start_line;
                }
            }
            Kind::List { data, .. } => {
                let mut tight = true;
                let items: Vec<usize> = self.doc.children(block).collect();
                'outer: for (k, &item) in items.iter().enumerate() {
                    let has_next = k + 1 < items.len();
                    if has_next && self.ends_with_blank_line(item, items[k + 1]) {
                        tight = false;
                        break;
                    }
                    let subs: Vec<usize> = self.doc.children(item).collect();
                    for (m, &sub) in subs.iter().enumerate() {
                        if m + 1 < subs.len() && self.ends_with_blank_line(sub, subs[m + 1]) {
                            tight = false;
                            break 'outer;
                        }
                    }
                }
                if let Some(l) = self.last_child(block) {
                    self.doc.nodes[block].end_line = self.doc.nodes[l].end_line;
                }
                self.doc.nodes[block].kind = Kind::List { data, tight };
            }
            _ => {}
        }
        if let Some(a) = above {
            self.tip = a;
        }
    }

    fn ends_with_blank_line(&self, block: usize, next: usize) -> bool {
        self.doc.nodes[block].end_line + 1 != self.doc.nodes[next].start_line
    }

    fn finalize_paragraph(&mut self, block: usize) {
        let mut content = std::mem::take(&mut self.doc.nodes[block].content);
        let mut has_refs = false;
        while content.starts_with('[') {
            match super::inlines::parse_reference(&content, &mut self.doc.refmap) {
                Some(pos) => {
                    content = content[pos..].to_string();
                    has_refs = true;
                }
                None => break,
            }
        }
        if has_refs && content.trim().is_empty() {
            self.doc.nodes[block].removed = true;
            return;
        }
        if self.pipeline.pipe_tables {
            if let Some(t) = super::table::try_parse_table(&content) {
                self.doc.nodes[block].kind = Kind::Table(t);
            }
        }
        self.doc.nodes[block].content = content;
    }

    // ----- continuation ------------------------------------------------------------

    /// 0 matched, 1 failed, 2 line consumed
    fn continue_block(&mut self, container: usize) -> u8 {
        let kind = self.doc.nodes[container].kind.clone();
        match kind {
            Kind::Document | Kind::List { .. } => 0,
            Kind::BlockQuote => {
                if !self.indented && self.peek(self.next_nonspace) == Some('>') {
                    self.advance_next_nonspace();
                    self.advance_offset(1, false);
                    if self.peek(self.offset).is_some_and(is_space_or_tab) {
                        self.advance_offset(1, true);
                    }
                    0
                } else {
                    1
                }
            }
            Kind::Item(data) => {
                if self.blank {
                    if self.last_child(container).is_none() {
                        return 1;
                    }
                    self.advance_next_nonspace();
                } else if self.indent >= data.marker_offset + data.padding {
                    self.advance_offset(data.marker_offset + data.padding, true);
                } else {
                    return 1;
                }
                0
            }
            Kind::Heading(_) | Kind::ThematicBreak | Kind::Table(_) => 1,
            Kind::CodeBlock {
                fenced,
                fence_char,
                fence_length,
                fence_offset,
                ..
            } => {
                if fenced {
                    let rest = &self.line[self.next_nonspace.min(self.line.len())..];
                    let mut n = 0;
                    while n < rest.len() && rest[n] == fence_char {
                        n += 1;
                    }
                    let closing = self.indent <= 3
                        && n >= 3
                        && n >= fence_length
                        && rest[n..].iter().all(|&c| is_space_or_tab(c));
                    if closing {
                        let ln = self.line_number;
                        self.finalize(container, ln);
                        return 2;
                    }
                    let mut i = fence_offset;
                    while i > 0 && self.peek(self.offset).is_some_and(is_space_or_tab) {
                        self.advance_offset(1, true);
                        i -= 1;
                    }
                    0
                } else if self.indent >= CODE_INDENT {
                    self.advance_offset(CODE_INDENT, true);
                    0
                } else if self.blank {
                    self.advance_next_nonspace();
                    0
                } else {
                    1
                }
            }
            Kind::HtmlBlock(t) => {
                if self.blank && (t == 6 || t == 7) {
                    1
                } else {
                    0
                }
            }
            Kind::Paragraph => {
                if self.blank {
                    1
                } else {
                    0
                }
            }
        }
    }

    // ----- block starts ------------------------------------------------------------

    /// 0 no match, 1 matched container, 2 matched leaf
    fn try_starts(&mut self, container: usize) -> u8 {
        let rest: Vec<char> = self.line[self.next_nonspace.min(self.line.len())..].to_vec();
        let first = rest.first().copied();
        // block quote
        if !self.indented && first == Some('>') {
            self.advance_next_nonspace();
            self.advance_offset(1, false);
            if self.peek(self.offset).is_some_and(is_space_or_tab) {
                self.advance_offset(1, true);
            }
            self.close_unmatched_blocks();
            self.add_child(Kind::BlockQuote);
            return 1;
        }
        // ATX heading
        if !self.indented && first == Some('#') {
            let mut n = 0;
            while n < rest.len() && rest[n] == '#' {
                n += 1;
            }
            if n <= 6 && (n == rest.len() || is_space_or_tab(rest[n])) {
                let mut m = n;
                while m < rest.len() && is_space_or_tab(rest[m]) {
                    m += 1;
                }
                self.advance_next_nonspace();
                self.advance_offset(m, false);
                self.close_unmatched_blocks();
                let h = self.add_child(Kind::Heading(n as u8));
                let text: String = self.line[self.offset.min(self.line.len())..].iter().collect();
                self.doc.nodes[h].content = strip_closing_hashes(&text);
                self.offset = self.line.len();
                return 2;
            }
        }
        // fenced code
        if !self.indented && matches!(first, Some('`') | Some('~')) {
            let fc = first.unwrap();
            let mut n = 0;
            while n < rest.len() && rest[n] == fc {
                n += 1;
            }
            if n >= 3 && !(fc == '`' && rest[n..].contains(&'`')) {
                self.close_unmatched_blocks();
                let indent = self.indent;
                self.add_child(Kind::CodeBlock {
                    fenced: true,
                    fence_char: fc,
                    fence_length: n,
                    fence_offset: indent,
                    info: String::new(),
                });
                self.advance_next_nonspace();
                self.advance_offset(n, false);
                return 2;
            }
        }
        // HTML block
        if !self.indented && first == Some('<') {
            if let Some(t) = html_block_start(&rest) {
                let lazy_para = !self.all_closed && !self.blank && is_paragraph(&self.doc.nodes[self.tip].kind);
                if t < 7 || (!is_paragraph(&self.doc.nodes[container].kind) && !lazy_para) {
                    self.close_unmatched_blocks();
                    self.add_child(Kind::HtmlBlock(t));
                    return 2;
                }
            }
        }
        // setext heading
        if !self.indented && is_paragraph(&self.doc.nodes[container].kind) && matches!(first, Some('=') | Some('-')) {
            let c = first.unwrap();
            let mut n = 0;
            while n < rest.len() && rest[n] == c {
                n += 1;
            }
            if rest[n..].iter().all(|&x| is_space_or_tab(x)) {
                self.close_unmatched_blocks();
                let mut content = self.doc.nodes[container].content.clone();
                while content.starts_with('[') {
                    match super::inlines::parse_reference(&content, &mut self.doc.refmap) {
                        Some(pos) => content = content[pos..].to_string(),
                        None => break,
                    }
                }
                if !content.is_empty() {
                    let level = if c == '=' { 1 } else { 2 };
                    self.doc.nodes[container].kind = Kind::Heading(level);
                    self.doc.nodes[container].content = content;
                    self.tip = container;
                    self.offset = self.line.len();
                    return 2;
                }
            }
        }
        // thematic break
        if !self.indented && is_thematic_break(&rest) {
            self.close_unmatched_blocks();
            self.add_child(Kind::ThematicBreak);
            self.offset = self.line.len();
            return 2;
        }
        // list item
        let in_list = matches!(self.doc.nodes[container].kind, Kind::List { .. });
        if !self.indented || in_list {
            if let Some(data) = self.parse_list_marker(container) {
                self.close_unmatched_blocks();
                let tip_matches = match &self.doc.nodes[self.tip].kind {
                    Kind::List { data: d, .. } => {
                        d.ordered == data.ordered && d.delimiter == data.delimiter && d.bullet_type == data.bullet_type
                    }
                    _ => false,
                };
                if !tip_matches {
                    self.add_child(Kind::List {
                        data: data.clone(),
                        tight: true,
                    });
                }
                self.add_child(Kind::Item(data));
                return 1;
            }
        }
        // indented code
        if self.indented && !is_paragraph(&self.doc.nodes[self.tip].kind) && !self.blank {
            self.advance_offset(CODE_INDENT, true);
            self.close_unmatched_blocks();
            self.add_child(Kind::CodeBlock {
                fenced: false,
                fence_char: ' ',
                fence_length: 0,
                fence_offset: 0,
                info: String::new(),
            });
            return 2;
        }
        0
    }

    fn parse_list_marker(&mut self, container: usize) -> Option<ListData> {
        if self.indent >= 4 {
            return None;
        }
        let rest: Vec<char> = self.line[self.next_nonspace.min(self.line.len())..].to_vec();
        let c = *rest.first()?;
        let pending = match &self.doc.nodes[container].kind {
            Kind::List { data, .. } => data.bullet_type,
            _ => '\0',
        };
        let in_paragraph = is_paragraph(&self.doc.nodes[container].kind);
        let (marker_len, mut data) = if matches!(c, '-' | '*' | '+') {
            (
                1,
                ListData {
                    ordered: false,
                    bullet_type: c,
                    delimiter: '\0',
                    ordered_start: String::new(),
                    padding: 0,
                    marker_offset: self.indent,
                },
            )
        } else if c.is_ascii_digit() {
            let mut n = 0;
            while n < rest.len() && rest[n].is_ascii_digit() {
                n += 1;
            }
            if n > 9 || !matches!(rest.get(n), Some('.') | Some(')')) {
                return None;
            }
            let digits: String = rest[..n].iter().collect();
            let trimmed = digits.trim_start_matches('0');
            let start = if trimmed.is_empty() {
                "0".to_string()
            } else {
                trimmed.to_string()
            };
            (
                n + 1,
                ListData {
                    ordered: true,
                    bullet_type: '1',
                    delimiter: rest[n],
                    ordered_start: start,
                    padding: 0,
                    marker_offset: self.indent,
                },
            )
        } else if self.pipeline.list_extras && c.is_ascii_alphabetic() {
            let roman_low = matches!(c, 'i' | 'v' | 'x');
            let roman_up = !roman_low && matches!(c, 'I' | 'V' | 'X');
            let (n, bullet, start) = if (roman_low || roman_up) && matches!(pending, '\0' | 'i' | 'I') {
                let mut n = 0;
                while n < rest.len()
                    && (if roman_low {
                        matches!(rest[n], 'i' | 'v' | 'x')
                    } else {
                        matches!(rest[n], 'I' | 'V' | 'X')
                    })
                {
                    n += 1;
                }
                let s: String = rest[..n].iter().collect();
                (n, if roman_low { 'i' } else { 'I' }, roman_to_arabic(&s).to_string())
            } else {
                let upper = c.is_ascii_uppercase();
                (
                    1,
                    if upper { 'A' } else { 'a' },
                    ((c.to_ascii_lowercase() as u32 - 'a' as u32) + 1).to_string(),
                )
            };
            if !matches!(rest.get(n), Some('.') | Some(')')) {
                return None;
            }
            (
                n + 1,
                ListData {
                    ordered: true,
                    bullet_type: bullet,
                    delimiter: rest[n],
                    ordered_start: start,
                    padding: 0,
                    marker_offset: self.indent,
                },
            )
        } else {
            return None;
        };
        // make sure we have spaces after
        match rest.get(marker_len) {
            None | Some(' ') | Some('\t') => {}
            _ => return None,
        }
        let blank_after = rest[marker_len..].iter().all(|&x| is_space_or_tab(x));
        if in_paragraph && (blank_after || (data.bullet_type == '1' && data.ordered_start != "1")) {
            return None;
        }
        self.advance_next_nonspace();
        self.advance_offset(marker_len, true);
        let spaces_start_col = self.column;
        let spaces_start_offset = self.offset;
        loop {
            self.advance_offset(1, true);
            let nextc = self.peek(self.offset);
            if !(self.column - spaces_start_col < 5 && nextc.is_some_and(is_space_or_tab)) {
                break;
            }
        }
        let blank_item = self.peek(self.offset).is_none();
        let spaces_after_marker = self.column - spaces_start_col;
        if spaces_after_marker >= 5 || spaces_after_marker < 1 || blank_item {
            data.padding = marker_len + 1;
            self.column = spaces_start_col;
            self.offset = spaces_start_offset;
            if self.peek(self.offset).is_some_and(is_space_or_tab) {
                self.advance_offset(1, true);
            }
        } else {
            data.padding = marker_len + spaces_after_marker;
        }
        Some(data)
    }

    fn incorporate_line(&mut self, ln: &str) {
        let mut all_matched = true;
        let mut container = 0usize;
        self.oldtip = self.tip;
        self.offset = 0;
        self.column = 0;
        self.blank = false;
        self.partially_consumed_tab = false;
        self.line = ln.chars().map(|c| if c == '\0' { '\u{FFFD}' } else { c }).collect();

        while let Some(last) = self.last_child(container) {
            if !self.doc.nodes[last].open {
                break;
            }
            container = last;
            self.find_next_nonspace();
            match self.continue_block(container) {
                0 => {}
                1 => {
                    all_matched = false;
                    container = self.doc.nodes[container].parent.unwrap();
                    break;
                }
                _ => return,
            }
        }
        let _ = all_matched;
        self.all_closed = container == self.oldtip;
        self.last_matched_container = container;

        let mut matched_leaf =
            !is_paragraph(&self.doc.nodes[container].kind) && accepts_lines(&self.doc.nodes[container].kind);
        while !matched_leaf {
            self.find_next_nonspace();
            let c = self.peek(self.next_nonspace);
            let maybe_special = c.is_some_and(|c| {
                matches!(c, '#' | '`' | '~' | '*' | '+' | '_' | '=' | '<' | '>' | '-') || c.is_ascii_digit()
            }) || (self.pipeline.list_extras && c.is_some_and(|c| c.is_ascii_alphabetic()));
            if !self.indented && !maybe_special {
                self.advance_next_nonspace();
                break;
            }
            match self.try_starts(container) {
                1 => container = self.tip,
                2 => {
                    container = self.tip;
                    matched_leaf = true;
                }
                _ => {
                    self.advance_next_nonspace();
                    break;
                }
            }
        }

        if !self.all_closed && !self.blank && is_paragraph(&self.doc.nodes[self.tip].kind) {
            self.add_line();
        } else {
            self.close_unmatched_blocks();
            if self.blank {
                if let Some(lc) = self.last_child(container) {
                    self.doc.nodes[lc].last_line_blank = true;
                }
            }
            let kind = self.doc.nodes[container].kind.clone();
            let last_line_blank = self.blank
                && !(matches!(kind, Kind::BlockQuote)
                    || matches!(kind, Kind::CodeBlock { fenced: true, .. })
                    || (matches!(kind, Kind::Item(_))
                        && self.last_child(container).is_none()
                        && self.doc.nodes[container].start_line == self.line_number));
            let mut cont = Some(container);
            while let Some(c) = cont {
                self.doc.nodes[c].last_line_blank = last_line_blank;
                cont = self.doc.nodes[c].parent;
            }
            if accepts_lines(&kind) {
                self.add_line();
                if let Kind::HtmlBlock(t) = kind {
                    if (1..=5).contains(&t) && html_block_end(t, &self.line[self.offset.min(self.line.len())..]) {
                        let ln = self.line_number;
                        self.finalize(container, ln);
                    }
                }
            } else if self.offset < self.line.len() && !self.blank {
                self.add_child(Kind::Paragraph);
                self.advance_next_nonspace();
                self.add_line();
            }
        }
    }
}

fn strip_closing_hashes(text: &str) -> String {
    let t = text.trim_end_matches([' ', '\t']);
    // whole content is hashes
    if !t.is_empty() && t.chars().all(|c| c == '#') {
        return String::new();
    }
    let without = t.trim_end_matches('#');
    if without.len() != t.len() && (without.ends_with(' ') || without.ends_with('\t')) {
        return without.trim_end_matches([' ', '\t']).to_string();
    }
    t.to_string()
}

fn is_thematic_break(rest: &[char]) -> bool {
    let Some(&c) = rest.first() else { return false };
    if !matches!(c, '*' | '_' | '-') {
        return false;
    }
    let mut n = 0;
    for &x in rest {
        if x == c {
            n += 1;
        } else if !is_space_or_tab(x) {
            return false;
        }
    }
    n >= 3
}

fn roman_to_arabic(s: &str) -> i32 {
    let v = |c: char| match c.to_ascii_lowercase() {
        'i' => 1,
        'v' => 5,
        'x' => 10,
        'l' => 50,
        'c' => 100,
        'd' => 500,
        'm' => 1000,
        _ => 0,
    };
    let cs: Vec<char> = s.chars().collect();
    let mut result = 0;
    for i in 0..cs.len() {
        let a = v(cs[i]);
        if i + 1 < cs.len() && a < v(cs[i + 1]) {
            result -= a;
        } else {
            result += a;
        }
    }
    result
}

/// Parses `text` into a block tree. Inline content is parsed afterwards.
pub fn parse_blocks(text: &str, pipeline: &Pipeline) -> Document {
    let mut p = Parser::new(pipeline);
    let mut lines: Vec<&str> = text.split('\n').collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    for (i, l) in lines.iter().enumerate() {
        p.line_number = i;
        let l = l.strip_suffix('\r').unwrap_or(l);
        p.incorporate_line(l);
    }
    let len = lines.len().saturating_sub(1);
    while p.tip != 0 || p.doc.nodes[0].open {
        let t = p.tip;
        p.finalize(t, len);
        if t == 0 {
            break;
        }
    }
    p.doc
}
