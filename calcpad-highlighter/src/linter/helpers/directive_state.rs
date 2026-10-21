//! DirectiveState.cs
use super::cs;
use crate::linter::models::ParseMode;
use crate::text::trim;
use crate::tokenizer::parse_mode_tracker::ParseModeTracker;

/// Output-value mode set by #equ / #val / #noc.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputMode {
    Equations,
    Values,
    NoCalculation,
}

/// Scope mode set by #local / #global.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeMode {
    Global,
    Local,
}

/// Visibility mode set by #show / #hide / #pre / #post.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisibilityMode {
    Shown,
    Hidden,
    ScreenOnly,
    PrintOnly,
}

/// Tracks the running state of Calcpad's mode directives as lines are visited in order.
#[derive(Debug)]
pub struct DirectiveState {
    output_stack: Vec<OutputMode>,
    visibility_stack: Vec<VisibilityMode>,
    parse_modes: ParseModeTracker,
    pub output: OutputMode,
    pub visibility: VisibilityMode,
    pub scope: ScopeMode,
    pub is_markdown_on: bool,
}

impl Default for DirectiveState {
    fn default() -> Self {
        DirectiveState {
            output_stack: Vec::new(),
            visibility_stack: Vec::new(),
            parse_modes: ParseModeTracker::new(),
            output: OutputMode::Equations,
            visibility: VisibilityMode::Shown,
            scope: ScopeMode::Global,
            is_markdown_on: false,
        }
    }
}

fn matches(line: &[char], directive: &str) -> bool {
    cs::starts_with_ic(line, directive)
}

impl DirectiveState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_mode(&self) -> ParseMode {
        self.parse_modes.mode()
    }

    /// Updates the tracked state from a trimmed directive line.
    pub fn apply(&mut self, trimmed_line: &[char]) {
        let l = trimmed_line;
        if self.parse_modes.apply(l) {
            return;
        }
        if matches(l, "#end val") || matches(l, "#end equ") || matches(l, "#end noc") {
            self.output = self.output_stack.pop().unwrap_or(OutputMode::Equations);
        } else if matches(l, "#equ") {
            self.output_stack.push(self.output);
            self.output = OutputMode::Equations;
        } else if matches(l, "#val") {
            self.output_stack.push(self.output);
            self.output = OutputMode::Values;
        } else if matches(l, "#noc") {
            self.output_stack.push(self.output);
            self.output = OutputMode::NoCalculation;
        } else if matches(l, "#end hide")
            || matches(l, "#end show")
            || matches(l, "#end pre")
            || matches(l, "#end post")
        {
            self.visibility = self.visibility_stack.pop().unwrap_or(VisibilityMode::Shown);
        } else if matches(l, "#hide") {
            self.visibility_stack.push(self.visibility);
            self.visibility = VisibilityMode::Hidden;
        } else if matches(l, "#show") {
            self.visibility_stack.push(self.visibility);
            self.visibility = VisibilityMode::Shown;
        } else if matches(l, "#pre") {
            self.visibility_stack.push(self.visibility);
            self.visibility = VisibilityMode::ScreenOnly;
        } else if matches(l, "#post") {
            self.visibility_stack.push(self.visibility);
            self.visibility = VisibilityMode::PrintOnly;
        } else if matches(l, "#global") {
            self.scope = ScopeMode::Global;
        } else if matches(l, "#local") {
            self.scope = ScopeMode::Local;
        } else if matches(l, "#md") {
            self.apply_markdown(l);
        }
    }

    fn apply_markdown(&mut self, l: &[char]) {
        let arg: &[char] = if l.len() > 3 { trim(&l[3..]) } else { &[] };
        if cs::equals_ic(arg, "off") {
            self.is_markdown_on = false;
        } else if arg.is_empty() || cs::equals_ic(arg, "on") {
            self.is_markdown_on = true;
        }
    }
}
