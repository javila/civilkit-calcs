//! Inline parsing (Markdig Parsers/InlineProcessor.cs and Parsers/Inlines/*, plus the
//! emphasis extras, task list and auto link extensions). Inlines are built in an arena of
//! linked nodes so that links and emphasis can take over a run of siblings, as in the
//! CommonMark reference algorithm that Markdig follows.
use std::collections::HashMap;

use super::chars::*;
use super::Pipeline;

#[derive(Clone, Debug, PartialEq)]
pub enum Inline {
    Text(String),
    Code(String),
    /// Raw inline HTML or an entity, written without escaping
    Html(String),
    SoftBreak,
    HardBreak,
    Emph {
        ch: char,
        count: usize,
        children: Vec<Inline>,
    },
    Link {
        url: String,
        title: String,
        image: bool,
        children: Vec<Inline>,
    },
    AutoLink {
        url: String,
        email: bool,
    },
    TaskList(bool),
}

#[derive(Clone, Debug)]
enum NK {
    Root,
    Text(String),
    Code(String),
    Html(String),
    Soft,
    Hard,
    Emph(char, usize),
    Link { url: String, title: String, image: bool },
    AutoLink { url: String, email: bool },
    Task(bool),
}

#[derive(Clone, Debug)]
struct N {
    kind: NK,
    parent: Option<usize>,
    first: Option<usize>,
    last: Option<usize>,
    prev: Option<usize>,
    next: Option<usize>,
}

struct Arena {
    n: Vec<N>,
}

impl Arena {
    fn new_node(&mut self, kind: NK) -> usize {
        self.n.push(N {
            kind,
            parent: None,
            first: None,
            last: None,
            prev: None,
            next: None,
        });
        self.n.len() - 1
    }

    fn append_child(&mut self, p: usize, c: usize) {
        self.unlink(c);
        self.n[c].parent = Some(p);
        if let Some(l) = self.n[p].last {
            self.n[l].next = Some(c);
            self.n[c].prev = Some(l);
            self.n[p].last = Some(c);
        } else {
            self.n[p].first = Some(c);
            self.n[p].last = Some(c);
        }
    }

    fn insert_after(&mut self, sib: usize, c: usize) {
        self.unlink(c);
        let p = self.n[sib].parent;
        let nx = self.n[sib].next;
        self.n[c].next = nx;
        if let Some(nx) = nx {
            self.n[nx].prev = Some(c);
        } else if let Some(p) = p {
            self.n[p].last = Some(c);
        }
        self.n[c].prev = Some(sib);
        self.n[sib].next = Some(c);
        self.n[c].parent = p;
    }

    fn unlink(&mut self, c: usize) {
        let (p, pv, nx) = (self.n[c].parent, self.n[c].prev, self.n[c].next);
        if let Some(pv) = pv {
            self.n[pv].next = nx;
        } else if let Some(p) = p {
            self.n[p].first = nx;
        }
        if let Some(nx) = nx {
            self.n[nx].prev = pv;
        } else if let Some(p) = p {
            self.n[p].last = pv;
        }
        self.n[c].parent = None;
        self.n[c].prev = None;
        self.n[c].next = None;
    }

    fn to_inlines(&self, p: usize) -> Vec<Inline> {
        let mut out: Vec<Inline> = Vec::new();
        let mut c = self.n[p].first;
        while let Some(i) = c {
            let item = match &self.n[i].kind {
                NK::Root => None,
                NK::Text(t) => {
                    if let Some(Inline::Text(prev)) = out.last_mut() {
                        prev.push_str(t);
                        None
                    } else if t.is_empty() {
                        None
                    } else {
                        Some(Inline::Text(t.clone()))
                    }
                }
                NK::Code(t) => Some(Inline::Code(t.clone())),
                NK::Html(t) => Some(Inline::Html(t.clone())),
                NK::Soft => Some(Inline::SoftBreak),
                NK::Hard => Some(Inline::HardBreak),
                NK::Emph(ch, count) => Some(Inline::Emph {
                    ch: *ch,
                    count: *count,
                    children: self.to_inlines(i),
                }),
                NK::Link { url, title, image } => Some(Inline::Link {
                    url: url.clone(),
                    title: title.clone(),
                    image: *image,
                    children: self.to_inlines(i),
                }),
                NK::AutoLink { url, email } => Some(Inline::AutoLink {
                    url: url.clone(),
                    email: *email,
                }),
                NK::Task(b) => Some(Inline::TaskList(*b)),
            };
            if let Some(item) = item {
                out.push(item);
            }
            c = self.n[i].next;
        }
        out
    }
}

/// `EmphasisDescriptor`: character, minimum and maximum run used, and whether it may
/// open or close inside a word.
#[derive(Clone, Copy)]
struct Descriptor {
    min: usize,
    max: usize,
    within_word: bool,
}

fn descriptor(p: &Pipeline, c: char) -> Option<Descriptor> {
    match c {
        '*' => Some(Descriptor {
            min: 1,
            max: 2,
            within_word: true,
        }),
        '_' => Some(Descriptor {
            min: 1,
            max: 2,
            within_word: false,
        }),
        '~' if p.emphasis_extras => Some(Descriptor {
            min: 1,
            max: 2,
            within_word: true,
        }),
        '^' if p.emphasis_extras => Some(Descriptor {
            min: 1,
            max: 1,
            within_word: true,
        }),
        '+' if p.emphasis_extras => Some(Descriptor {
            min: 2,
            max: 2,
            within_word: true,
        }),
        '=' if p.emphasis_extras => Some(Descriptor {
            min: 2,
            max: 2,
            within_word: true,
        }),
        _ => None,
    }
}

#[derive(Clone, Debug)]
struct Delim {
    node: usize,
    ch: char,
    count: usize,
    can_open: bool,
    can_close: bool,
}

#[derive(Clone, Debug)]
struct Bracket {
    node: usize,
    delim_bottom: usize,
    index: usize,
    image: bool,
    active: bool,
    bracket_after: bool,
}

struct InlineParser<'a> {
    s: Vec<char>,
    pos: usize,
    arena: Arena,
    root: usize,
    delims: Vec<Delim>,
    brackets: Vec<Bracket>,
    refmap: &'a HashMap<String, (String, String)>,
    pipeline: &'a Pipeline,
    in_list_item: bool,
}

fn is_autolink_start(c: char) -> bool {
    matches!(c, 'h' | 'f' | 'm' | 't' | 'w')
}

impl<'a> InlineParser<'a> {
    fn peek(&self) -> Option<char> {
        self.s.get(self.pos).copied()
    }

    fn peek_at(&self, i: usize) -> char {
        self.s.get(i).copied().unwrap_or('\0')
    }

    fn prev_char(&self, i: usize) -> char {
        if i == 0 {
            '\0'
        } else {
            self.s[i - 1]
        }
    }

    fn text(&mut self, t: &str) -> usize {
        let n = self.arena.new_node(NK::Text(t.to_string()));
        self.arena.append_child(self.root, n);
        n
    }

    fn append(&mut self, kind: NK) -> usize {
        let n = self.arena.new_node(kind);
        self.arena.append_child(self.root, n);
        n
    }

    fn is_special(&self, c: char) -> bool {
        matches!(c, '\n' | '\\' | '`' | '[' | ']' | '!' | '<' | '&' | '*' | '_')
            || (self.pipeline.emphasis_extras && matches!(c, '~' | '^' | '+' | '='))
    }

    fn autolink_possible_at(&self, i: usize) -> bool {
        if !self.pipeline.auto_links || !is_autolink_start(self.peek_at(i)) {
            return false;
        }
        let pc = self.prev_char(i);
        pc == '\0' || is_whitespace(pc) || "*_~(".contains(pc)
    }

    fn parse_inline(&mut self) {
        let c = self.peek().unwrap();
        let handled = match c {
            '\n' => {
                self.parse_newline();
                true
            }
            '\\' => {
                self.parse_backslash();
                true
            }
            '`' => {
                self.parse_backticks();
                true
            }
            '[' => self.parse_open_bracket(),
            '!' => {
                self.parse_bang();
                true
            }
            ']' => {
                self.parse_close_bracket();
                true
            }
            '<' => self.parse_autolink() || self.parse_html_tag(),
            '&' => self.parse_entity(),
            _ if descriptor(self.pipeline, c).is_some() => self.handle_delim(c),
            _ if self.autolink_possible_at(self.pos) => self.parse_bare_autolink(),
            _ => {
                self.parse_string();
                true
            }
        };
        if !handled {
            self.pos += 1;
            self.text(&c.to_string());
        }
    }

    fn parse_string(&mut self) {
        let start = self.pos;
        self.pos += 1;
        while let Some(c) = self.peek() {
            if self.is_special(c) || self.autolink_possible_at(self.pos) {
                break;
            }
            self.pos += 1;
        }
        let t: String = self.s[start..self.pos].iter().collect();
        self.text(&t);
    }

    fn parse_newline(&mut self) {
        self.pos += 1;
        let root = self.root;
        let mut hard = false;
        let mut soft = true;
        if let Some(l) = self.arena.n[root].last {
            if let NK::Text(t) = &mut self.arena.n[l].kind {
                if t.ends_with(' ') {
                    let trimmed = t.trim_end_matches(' ');
                    hard = t.len() - trimmed.len() >= 2;
                    *t = trimmed.to_string();
                }
            }
        }
        if hard {
            soft = false;
        }
        self.append(if soft { NK::Soft } else { NK::Hard });
        while self.peek() == Some(' ') {
            self.pos += 1;
        }
    }

    fn parse_backslash(&mut self) {
        self.pos += 1;
        match self.peek() {
            Some('\n') => {
                self.pos += 1;
                self.append(NK::Hard);
                while self.peek() == Some(' ') {
                    self.pos += 1;
                }
            }
            Some(c) if is_escapable(c) => {
                self.pos += 1;
                self.text(&c.to_string());
            }
            _ => {
                self.text("\\");
            }
        }
    }

    fn parse_backticks(&mut self) {
        let start = self.pos;
        while self.peek() == Some('`') {
            self.pos += 1;
        }
        let ticks = self.pos - start;
        let after_open = self.pos;
        let mut i = self.pos;
        while i < self.s.len() {
            if self.s[i] == '`' {
                let run_start = i;
                while i < self.s.len() && self.s[i] == '`' {
                    i += 1;
                }
                if i - run_start == ticks {
                    let contents: String = self.s[after_open..run_start]
                        .iter()
                        .map(|&c| if c == '\n' { ' ' } else { c })
                        .collect();
                    let lit = if !contents.is_empty()
                        && contents.chars().any(|c| c != ' ')
                        && contents.starts_with(' ')
                        && contents.ends_with(' ')
                    {
                        contents[1..contents.len() - 1].to_string()
                    } else {
                        contents
                    };
                    self.pos = i;
                    self.append(NK::Code(lit));
                    return;
                }
            } else {
                i += 1;
            }
        }
        self.pos = after_open;
        let t: String = "`".repeat(ticks);
        self.text(&t);
    }

    fn handle_delim(&mut self, c: char) -> bool {
        let desc = descriptor(self.pipeline, c).unwrap();
        let start = self.pos;
        let pc = self.prev_char(start);
        let mut n = 0;
        while self.peek_at(start + n) == c {
            n += 1;
        }
        if n < desc.min {
            self.pos += n;
            let t: String = std::iter::repeat_n(c, n).collect();
            self.text(&t);
            return true;
        }
        let nc = self.peek_at(start + n);
        let (can_open, can_close) = check_open_close_delimiter(pc, nc, desc.within_word);
        self.pos += n;
        let t: String = std::iter::repeat_n(c, n).collect();
        let node = self.text(&t);
        if can_open || can_close {
            self.delims.push(Delim {
                node,
                ch: c,
                count: n,
                can_open,
                can_close,
            });
        }
        true
    }

    fn remove_chars(&mut self, node: usize, n: usize, from_end: bool) {
        if let NK::Text(t) = &mut self.arena.n[node].kind {
            let cs: Vec<char> = t.chars().collect();
            let keep = cs.len().saturating_sub(n);
            *t = if from_end {
                cs[..keep].iter().collect()
            } else {
                cs[n.min(cs.len())..].iter().collect()
            };
        }
    }

    fn process_emphasis(&mut self, bottom: usize) {
        let mut i = bottom;
        while i < self.delims.len() {
            let ch = self.delims[i].ch;
            let desc = descriptor(self.pipeline, ch).unwrap();
            if self.delims[i].can_close {
                while self.delims[i].count >= desc.min {
                    let mut opener = None;
                    for j in (bottom..i).rev() {
                        let o = &self.delims[j];
                        let c = &self.delims[i];
                        let is_odd = (c.can_open || o.can_close)
                            && o.count != c.count
                            && (o.count + c.count) % 3 == 0
                            && (o.count % 3 != 0 || c.count % 3 != 0);
                        if o.ch == c.ch && o.can_open && o.count >= desc.min && !is_odd {
                            opener = Some(j);
                            break;
                        }
                    }
                    let Some(j) = opener else {
                        if !self.delims[i].can_open {
                            self.delims.remove(i);
                            i = i.wrapping_sub(1);
                        }
                        break;
                    };
                    let mut closed = false;
                    loop {
                        let delta = self.delims[j].count.min(self.delims[i].count).min(desc.max);
                        let emph = self.arena.new_node(NK::Emph(ch, delta));
                        let (on, cn) = (self.delims[j].node, self.delims[i].node);
                        self.remove_chars(on, delta, true);
                        self.remove_chars(cn, delta, false);
                        let mut tmp = self.arena.n[on].next;
                        while let Some(t) = tmp {
                            if t == cn {
                                break;
                            }
                            let nx = self.arena.n[t].next;
                            self.arena.append_child(emph, t);
                            tmp = nx;
                        }
                        self.arena.insert_after(on, emph);
                        self.delims[j].count -= delta;
                        self.delims[i].count -= delta;
                        // intermediate delimiters become literal
                        if i > j + 1 {
                            self.delims.drain(j + 1..i);
                            i = j + 1;
                        }
                        if self.delims[i].count == 0 {
                            self.arena.unlink(cn);
                            self.delims.remove(i);
                            i = i.wrapping_sub(1);
                            if self.delims[j].count == 0 {
                                self.arena.unlink(on);
                                self.delims.remove(j);
                                i = i.wrapping_sub(1);
                            }
                            closed = true;
                            break;
                        }
                        if self.delims[j].count >= desc.min && self.delims[i].count >= desc.min {
                            continue;
                        } else if self.delims[j].count > 0 {
                            self.delims.remove(j);
                            i -= 1;
                        } else {
                            self.arena.unlink(on);
                            self.delims.remove(j);
                            i -= 1;
                        }
                        break;
                    }
                    if closed {
                        break;
                    }
                }
            }
            i = i.wrapping_add(1);
        }
        self.delims.truncate(bottom);
    }

    fn parse_open_bracket(&mut self) -> bool {
        if self.in_list_item && self.pipeline.task_lists {
            let c1 = self.peek_at(self.pos + 1);
            if (c1 == ' ' || c1 == 'x' || c1 == 'X') && self.peek_at(self.pos + 2) == ']' {
                self.pos += 3;
                self.append(NK::Task(c1 != ' '));
                return true;
            }
        }
        let start = self.pos;
        self.pos += 1;
        let node = self.text("[");
        self.add_bracket(node, start + 1, false);
        true
    }

    fn add_bracket(&mut self, node: usize, index: usize, image: bool) {
        if let Some(b) = self.brackets.last_mut() {
            b.bracket_after = true;
        }
        self.brackets.push(Bracket {
            node,
            delim_bottom: self.delims.len(),
            index,
            image,
            active: true,
            bracket_after: false,
        });
    }

    fn parse_bang(&mut self) {
        let start = self.pos;
        self.pos += 1;
        if self.peek() == Some('[') {
            self.pos += 1;
            let node = self.text("![");
            self.add_bracket(node, start + 2, true);
        } else {
            self.text("!");
        }
    }

    fn spnl(&mut self) {
        while self.peek().is_some_and(|c| c == ' ' || c == '\t') {
            self.pos += 1;
        }
        if self.peek() == Some('\n') {
            self.pos += 1;
        }
        while self.peek().is_some_and(|c| c == ' ' || c == '\t') {
            self.pos += 1;
        }
    }

    fn parse_close_bracket(&mut self) {
        let startpos = self.pos;
        self.pos += 1;
        let Some(opener) = self.brackets.last().cloned() else {
            self.text("]");
            return;
        };
        if !opener.active {
            self.text("]");
            self.brackets.pop();
            return;
        }
        let is_image = opener.image;
        let savepos = self.pos;
        let mut matched = false;
        let mut dest = String::new();
        let mut title = String::new();
        if self.peek() == Some('(') {
            self.pos += 1;
            self.spnl();
            if let Some(d) = self.parse_link_destination() {
                dest = d;
                let before_title = self.pos;
                self.spnl();
                if self.pos > before_title {
                    if let Some(t) = self.parse_link_title() {
                        title = t;
                    } else {
                        self.pos = before_title;
                    }
                }
                self.spnl();
                if self.peek() == Some(')') {
                    self.pos += 1;
                    matched = true;
                }
            }
            if !matched {
                self.pos = savepos;
            }
        }
        if !matched {
            let beforelabel = self.pos;
            let n = self.parse_link_label();
            let reflabel = if n > 2 {
                Some(self.s[beforelabel..beforelabel + n].iter().collect::<String>())
            } else if !opener.bracket_after {
                Some(self.s[opener.index - 1..startpos + 1].iter().collect::<String>())
            } else {
                None
            };
            if n == 0 {
                self.pos = savepos;
            }
            if let Some(label) = reflabel {
                if let Some((d, t)) = self.refmap.get(&normalize_reference(&label)) {
                    dest = d.clone();
                    title = t.clone();
                    matched = true;
                }
            }
        }
        if matched {
            let node = self.arena.new_node(NK::Link {
                url: dest,
                title,
                image: is_image,
            });
            let mut tmp = self.arena.n[opener.node].next;
            while let Some(t) = tmp {
                let nx = self.arena.n[t].next;
                self.arena.append_child(node, t);
                tmp = nx;
            }
            let root = self.root;
            self.arena.append_child(root, node);
            self.process_emphasis(opener.delim_bottom);
            self.brackets.pop();
            self.arena.unlink(opener.node);
            if !is_image {
                for b in self.brackets.iter_mut() {
                    if !b.image {
                        b.active = false;
                    }
                }
            }
        } else {
            self.brackets.pop();
            self.pos = startpos + 1;
            self.text("]");
        }
    }

    fn parse_link_label(&mut self) -> usize {
        if self.peek() != Some('[') {
            return 0;
        }
        let start = self.pos;
        let mut i = start + 1;
        while i < self.s.len() {
            match self.s[i] {
                '\\' if i + 1 < self.s.len() && is_escapable(self.s[i + 1]) => i += 2,
                '[' => return 0,
                ']' => {
                    if i - start > 1000 {
                        return 0;
                    }
                    self.pos = i + 1;
                    return i + 1 - start;
                }
                _ => i += 1,
            }
        }
        0
    }

    fn parse_link_destination(&mut self) -> Option<String> {
        if self.peek() == Some('<') {
            let mut i = self.pos + 1;
            let mut out = String::new();
            while i < self.s.len() {
                let c = self.s[i];
                if c == '\\' && i + 1 < self.s.len() && is_escapable(self.s[i + 1]) {
                    out.push(self.s[i + 1]);
                    i += 2;
                    continue;
                }
                if c == '>' {
                    self.pos = i + 1;
                    return Some(out);
                }
                if c == '<' || c == '\n' {
                    return None;
                }
                out.push(c);
                i += 1;
            }
            None
        } else {
            let start = self.pos;
            let mut i = self.pos;
            let mut depth = 0i32;
            let mut out = String::new();
            while i < self.s.len() {
                let c = self.s[i];
                if c == '\\' && i + 1 < self.s.len() && is_escapable(self.s[i + 1]) {
                    out.push(self.s[i + 1]);
                    i += 2;
                    continue;
                }
                if c == '(' {
                    depth += 1;
                    if depth > 32 {
                        return None;
                    }
                } else if c == ')' {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                } else if c.is_whitespace() || c.is_control() {
                    break;
                }
                out.push(c);
                i += 1;
            }
            if i == start && self.peek() != Some(')') {
                return None;
            }
            if depth != 0 {
                return None;
            }
            self.pos = i;
            Some(out)
        }
    }

    fn parse_link_title(&mut self) -> Option<String> {
        let open = self.peek()?;
        let close = match open {
            '"' => '"',
            '\'' => '\'',
            '(' => ')',
            _ => return None,
        };
        let mut i = self.pos + 1;
        let mut out = String::new();
        while i < self.s.len() {
            let c = self.s[i];
            if c == '\\' && i + 1 < self.s.len() && is_escapable(self.s[i + 1]) {
                out.push(self.s[i + 1]);
                i += 2;
                continue;
            }
            if c == close {
                self.pos = i + 1;
                return Some(out);
            }
            if open == '(' && c == '(' {
                return None;
            }
            out.push(c);
            i += 1;
        }
        None
    }

    fn parse_autolink(&mut self) -> bool {
        let rest = &self.s[self.pos..];
        // URI autolink
        let mut i = 1;
        while i < rest.len() && (rest[i].is_ascii_alphanumeric() || matches!(rest[i], '.' | '+' | '-')) {
            i += 1;
        }
        let scheme_len = i - 1;
        if rest.get(1).is_some_and(|c| c.is_ascii_alphabetic())
            && (2..=32).contains(&scheme_len)
            && rest.get(i) == Some(&':')
        {
            let mut j = i + 1;
            while j < rest.len() && !matches!(rest[j], '<' | '>') && rest[j] as u32 > 0x20 {
                j += 1;
            }
            if rest.get(j) == Some(&'>') {
                let url: String = rest[1..j].iter().collect();
                self.pos += j + 1;
                self.append(NK::AutoLink { url, email: false });
                return true;
            }
        }
        // email autolink
        let mut j = 1;
        while j < rest.len() && (rest[j].is_ascii_alphanumeric() || ".!#$%&'*+/=?^_`{|}~-".contains(rest[j])) {
            j += 1;
        }
        if j > 1 && rest.get(j) == Some(&'@') {
            let mut k = j + 1;
            let dstart = k;
            while k < rest.len() && (rest[k].is_ascii_alphanumeric() || matches!(rest[k], '-' | '.')) {
                k += 1;
            }
            if k > dstart && rest.get(k) == Some(&'>') {
                let domain: String = rest[dstart..k].iter().collect();
                let ok = domain
                    .split('.')
                    .all(|l| !l.is_empty() && l.len() <= 63 && !l.starts_with('-') && !l.ends_with('-'));
                if ok {
                    let url: String = rest[1..k].iter().collect();
                    self.pos += k + 1;
                    self.append(NK::AutoLink { url, email: true });
                    return true;
                }
            }
        }
        false
    }

    fn parse_html_tag(&mut self) -> bool {
        match match_html_tag(&self.s[self.pos..]) {
            Some(n) => {
                let t: String = self.s[self.pos..self.pos + n].iter().collect();
                self.pos += n;
                self.append(NK::Html(t));
                true
            }
            None => false,
        }
    }

    fn parse_entity(&mut self) -> bool {
        match match_entity(&self.s[self.pos..]) {
            Some(n) => {
                let t: String = self.s[self.pos..self.pos + n].iter().collect();
                self.pos += n;
                self.append(NK::Html(t));
                true
            }
            None => false,
        }
    }

    /// `AutoLinkParser` (bare http://, https://, ftp://, mailto:, tel: and www. links)
    fn parse_bare_autolink(&mut self) -> bool {
        let rest: String = self.s[self.pos..].iter().take(8).collect();
        let c = self.s[self.pos];
        let may_be_valid = match c {
            'h' => rest.starts_with("https://") || rest.starts_with("http://"),
            'w' => rest.starts_with("www."),
            'f' => rest.starts_with("ftp://"),
            'm' => rest.starts_with("mailto:"),
            _ => rest.starts_with("tel:"),
        };
        if !may_be_valid || !self.autolink_valid_in_context() {
            self.parse_string_one();
            return true;
        }
        // LinkHelper.TryParseUrl with isAutoLink
        let mut i = self.pos;
        let mut opened = 0i32;
        let mut link = String::new();
        loop {
            let ch = self.peek_at(i);
            if ch == '(' {
                opened += 1;
            }
            if ch == ')' {
                opened -= 1;
                if opened < 0 {
                    break;
                }
            }
            if ch == '\0' || ch == ' ' || ch == '\t' || ch.is_control() || ch == '<' {
                break;
            }
            if ch == '&' && match_entity(&self.s[i..]).is_some() {
                break;
            }
            let next = self.peek_at(i + 1);
            if matches!(ch, '?' | '!' | '.' | ',' | ':' | '*' | '_' | '~')
                && (next == '\0' || next == ' ' || next == '\t' || next.is_control() || next == '<')
            {
                break;
            }
            link.push(ch);
            i += 1;
        }
        if opened > 0 {
            self.parse_string_one();
            return true;
        }
        // trailing pending emphasis characters are not part of the link
        let pending: Vec<char> = self.delims.iter().map(|d| d.ch).collect();
        if !pending.is_empty() {
            while link.ends_with(|x| pending.contains(&x)) {
                link.pop();
            }
        }
        let lc: Vec<char> = link.chars().collect();
        let domain_offset = match c {
            'h' => {
                if link == "http://" || link == "https://" {
                    self.parse_string_one();
                    return true;
                }
                if lc[4] == 's' {
                    8
                } else {
                    7
                }
            }
            'w' => 4,
            'f' => {
                if link == "ftp://" {
                    self.parse_string_one();
                    return true;
                }
                6
            }
            't' => 0,
            _ => match lc.iter().position(|&x| x == '@') {
                None | Some(7) => {
                    self.parse_string_one();
                    return true;
                }
                Some(a) => a + 1,
            },
        };
        if c != 't' && !is_valid_domain(&lc, domain_offset) {
            self.parse_string_one();
            return true;
        }
        self.pos += lc.len();
        let url = if c == 'w' {
            format!("http://{}", link)
        } else {
            link.clone()
        };
        let skip = match c {
            'm' => 7,
            't' => 4,
            _ => 0,
        };
        let text: String = lc[skip..].iter().collect();
        let node = self.arena.new_node(NK::Link {
            url,
            title: String::new(),
            image: false,
        });
        let t = self.arena.new_node(NK::Text(text));
        self.arena.append_child(node, t);
        let root = self.root;
        self.arena.append_child(root, node);
        true
    }

    fn autolink_valid_in_context(&self) -> bool {
        // a pending <a> tag
        let mut c = self.arena.n[self.root].last;
        while let Some(i) = c {
            if let NK::Html(t) = &self.arena.n[i].kind {
                let l = t.to_ascii_lowercase();
                if l.starts_with("</a") {
                    break;
                }
                if l.starts_with("<a") {
                    return false;
                }
            }
            c = self.arena.n[i].prev;
        }
        // pending link brackets
        !self.brackets.iter().any(|b| b.active)
    }

    fn parse_string_one(&mut self) {
        let c = self.s[self.pos];
        self.pos += 1;
        self.text(&c.to_string());
    }
}

fn is_valid_domain(link: &[char], prefix: usize) -> bool {
    let mut segment_count = 1;
    let mut segment_has_chars = false;
    let mut last_underscore_segment: i32 = -1;
    let mut i = prefix;
    while i < link.len() {
        let c = link[i];
        if !c.is_alphanumeric() {
            if c == '.' {
                if !segment_has_chars {
                    return false;
                }
                segment_count += 1;
                segment_has_chars = false;
                i += 1;
                continue;
            }
            if matches!(c, '/' | '?' | '#' | ':') {
                break;
            }
            if c == '_' {
                last_underscore_segment = segment_count;
            } else if c != '-' && (c.is_whitespace() || c.is_ascii_punctuation() || c == '\0') {
                return false;
            }
        }
        segment_has_chars = true;
        i += 1;
    }
    segment_count != 1 && segment_has_chars && segment_count - last_underscore_segment >= 2
}

pub fn normalize_reference(label: &str) -> String {
    let inner = label.trim_start_matches('[').trim_end_matches(']');
    inner.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Removes backslash escapes (used for the info string of fenced code blocks).
pub fn unescape_string(s: &str) -> String {
    let cs: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '\\' && i + 1 < cs.len() && is_escapable(cs[i + 1]) {
            out.push(cs[i + 1]);
            i += 2;
        } else {
            out.push(cs[i]);
            i += 1;
        }
    }
    out
}

/// Parses a link reference definition at the start of `s`, storing it in `refmap`.
/// Returns the number of bytes consumed.
pub fn parse_reference(s: &str, refmap: &mut HashMap<String, (String, String)>) -> Option<usize> {
    let pipeline = Pipeline::default();
    let empty = HashMap::new();
    let mut p = InlineParser {
        s: s.chars().collect(),
        pos: 0,
        arena: Arena { n: Vec::new() },
        root: 0,
        delims: Vec::new(),
        brackets: Vec::new(),
        refmap: &empty,
        pipeline: &pipeline,
        in_list_item: false,
    };
    p.root = p.arena.new_node(NK::Root);
    let n = p.parse_link_label();
    if n <= 2 {
        return None;
    }
    let label: String = p.s[..n].iter().collect();
    if p.peek() != Some(':') {
        return None;
    }
    p.pos += 1;
    p.spnl();
    let dest = p.parse_link_destination()?;
    if dest.is_empty() && !p.s.get(p.pos.saturating_sub(1)).is_some_and(|&c| c == '>') {
        return None;
    }
    let before_title = p.pos;
    p.spnl();
    let mut title = String::new();
    if p.pos != before_title {
        if let Some(t) = p.parse_link_title() {
            title = t;
        } else {
            p.pos = before_title;
        }
    }
    // rest of the line must be blank
    let mut at_line_end = false;
    while p.peek().is_some_and(|c| c == ' ' || c == '\t') {
        p.pos += 1;
    }
    if matches!(p.peek(), None | Some('\n')) {
        at_line_end = true;
    }
    if !at_line_end {
        if title.is_empty() {
            return None;
        }
        // the title is not followed by a line end: retry without the title
        p.pos = before_title;
        while p.peek().is_some_and(|c| c == ' ' || c == '\t') {
            p.pos += 1;
        }
        if !matches!(p.peek(), None | Some('\n')) {
            return None;
        }
        title.clear();
    }
    if p.peek() == Some('\n') {
        p.pos += 1;
    }
    let key = normalize_reference(&label);
    if key.is_empty() {
        return None;
    }
    refmap.entry(key).or_insert((dest, title));
    let consumed: String = p.s[..p.pos].iter().collect();
    Some(consumed.len())
}

/// Parses the inline content of a paragraph, heading or table cell.
pub fn parse_inlines(
    text: &str,
    refmap: &HashMap<String, (String, String)>,
    pipeline: &Pipeline,
    in_list_item: bool,
) -> Vec<Inline> {
    let trimmed = text.trim_matches(|c: char| c == ' ' || c == '\t' || c == '\n' || c == '\r');
    let mut p = InlineParser {
        s: trimmed.chars().collect(),
        pos: 0,
        arena: Arena { n: Vec::new() },
        root: 0,
        delims: Vec::new(),
        brackets: Vec::new(),
        refmap,
        pipeline,
        in_list_item,
    };
    p.root = p.arena.new_node(NK::Root);
    while p.pos < p.s.len() {
        p.parse_inline();
    }
    p.process_emphasis(0);
    p.arena.to_inlines(p.root)
}
