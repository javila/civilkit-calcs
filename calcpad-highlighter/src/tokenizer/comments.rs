//! CalcpadTokenizer.Comments.cs: comment and HTML tag parsing.
use super::models::TokenType;
use super::{CalcpadTokenizer, SpecialContentType, TagState};
use crate::text::is_digit;

impl CalcpadTokenizer {
    /// True while inside an unclosed `<!--` block, carried over from a previous line or opened by
    /// the current buffer, which starts at the `<` of the opener.
    pub(super) fn in_open_html_comment(&self) -> bool {
        self.in_html_comment
            || (self.builder.len() >= 4
                && self.builder[0] == '<'
                && self.builder[1] == '!'
                && self.builder[2] == '-'
                && self.builder[3] == '-')
    }

    pub(super) fn reset_comment_state(&mut self) {
        self.in_html_comment = false;
        self.current_tag_name = String::new();
        self.tag_state = TagState::None;
        self.continue_text_comment = '\0';
        self.continue_tag_comment = '\0';
    }

    /// The token type of comment text for the current special content / HTML state.
    pub(super) fn content_type(&self) -> TokenType {
        match self.state.in_special_content {
            SpecialContentType::Script => TokenType::JavaScript,
            SpecialContentType::Style => TokenType::Css,
            SpecialContentType::Svg => TokenType::Svg,
            SpecialContentType::None => {
                if self.state.has_html_content {
                    TokenType::HtmlContent
                } else {
                    TokenType::Comment
                }
            }
        }
    }

    pub(super) fn parse_comment(&mut self, c: char) {
        // Skip comment parsing in Include mode (a quoted filename is not a comment) and in
        // SettingsJson mode (quotes belong to the JSON payload).
        if self.state.current_type == TokenType::Include || self.state.current_type == TokenType::SettingsJson {
            return;
        }

        // Cyrillic auto-comment
        if ('А'..='я').contains(&c) {
            if self.state.text_comment == '\0' && !self.state.is_subscript {
                self.append(self.state.current_type);
                self.state.text_comment = '\'';
                self.builder.clear();
                self.builder.push(self.state.text_comment);
                self.state.current_type = TokenType::Comment;
            }
        } else if c == '\'' || c == '"' {
            if self.state.text_comment == '\0' {
                self.state.is_units = false;
                self.state.text_comment = c;
                self.state.tag_comment = if c == '\'' { '"' } else { '\'' };
                // Only reset HTML state if not continuing from the previous line
                if self.state.in_special_content == SpecialContentType::None {
                    self.state.has_html_content = false;
                }
                self.append(self.state.current_type);
                if self.state.is_tag {
                    if self.tag_state == TagState::Starting {
                        self.state.current_type = TokenType::Tag;
                    } else {
                        self.state.current_type = self.content_type();
                        self.state.is_tag = false;
                    }
                } else {
                    self.state.current_type = self.content_type();
                }
            } else if c == self.state.text_comment {
                self.state.text_comment = '\0';
                let prev_type = self.state.current_type;
                self.state.current_type = TokenType::Comment;

                if prev_type == TokenType::Comment {
                    // Regular comment: check for HTML comment markers before emitting
                    self.check_html_comment();
                    self.builder.push(c);
                    self.append(self.state.current_type);
                } else {
                    // JS/CSS/SVG/HtmlContent: flush content with its real type, then emit the
                    // closing quote as Comment
                    self.append(prev_type);
                    self.builder.push(c);
                    self.append(TokenType::Comment);
                }

                self.state.current_type = TokenType::Comment;
                // Reset has_html_content only when not inside a special content block
                if self.state.in_special_content == SpecialContentType::None {
                    self.state.has_html_content = false;
                }
            } else if self.state.is_tag {
                if self.state.is_tag_comment {
                    self.state.current_type = TokenType::Comment;
                } else {
                    self.builder.push(c);
                }

                self.append(self.state.current_type);
                if self.state.is_tag_comment && c == self.state.tag_comment {
                    self.builder.push(c);
                    self.append(TokenType::Tag);
                }
                self.state.is_tag_comment = !self.state.is_tag_comment;
            }
        }
    }

    pub(super) fn parse_tag_in_comment(&mut self, c: char) {
        if c == '>' {
            self.builder.push(c);
            if self.state.is_tag {
                self.state.current_type = TokenType::Tag;
                self.state.has_html_content = true;

                // Use the stored tag name (populated when the tag started) instead of extracting
                // from the builder, which may only contain ">" if the tag has attributes
                let tag_name = if !self.current_tag_name.is_empty() {
                    self.current_tag_name.clone()
                } else {
                    self.get_tag_name()
                };
                let lower = tag_name.to_lowercase();
                if self.tag_state != TagState::Closing && self.tag_state != TagState::SelfClosing {
                    let new_special = match lower.as_str() {
                        "script" => SpecialContentType::Script,
                        "style" => SpecialContentType::Style,
                        "svg" => SpecialContentType::Svg,
                        _ => self.state.in_special_content,
                    };
                    // Invalid nesting (a style tag inside a script tag) keeps the outer context.
                    // Upstream also prints a console warning here; the port does not.
                    if new_special == self.state.in_special_content
                        || self.state.in_special_content == SpecialContentType::None
                    {
                        self.state.in_special_content = new_special;
                    }
                } else if self.tag_state == TagState::Closing {
                    let closing_matches = match lower.as_str() {
                        "script" => self.state.in_special_content == SpecialContentType::Script,
                        "style" => self.state.in_special_content == SpecialContentType::Style,
                        "svg" => self.state.in_special_content == SpecialContentType::Svg,
                        _ => false,
                    };
                    if closing_matches {
                        self.state.in_special_content = SpecialContentType::None;
                        self.state.has_html_content = false;
                    }
                }
            } else {
                self.check_html_comment();
                if self.in_html_comment {
                    return;
                }
            }

            self.append(self.state.current_type);

            // Token type for content after the tag
            self.state.current_type = self.content_type();

            self.state.is_tag = false;
            self.current_tag_name = String::new();
        } else if c == '<' {
            if self.in_open_html_comment() {
                self.builder.push(c);
                return;
            }

            self.tag_state = TagState::None;
            self.append(self.state.current_type);
            self.builder.push(c);
            self.state.is_tag = true;
            self.state.has_html_content = true;
            self.current_tag_name = String::new();
        } else {
            if self.state.is_tag {
                self.state.is_tag = self.check_tag(c);
            }

            if !(self.state.is_tag && c == self.state.tag_comment) {
                if self.state.current_type == TokenType::Macro {
                    if c == '(' || c == ')' {
                        self.parse_brackets(c, -1);
                    } else {
                        self.builder.push(c);
                    }

                    if c != '(' {
                        self.state.current_type = self.content_type();
                    }
                } else {
                    self.builder.push(c);
                }

                if c == '$' {
                    self.parse_macro_in_comment(self.state.current_type);
                }
            }
        }
    }

    fn check_tag(&mut self, c: char) -> bool {
        // Handle tags in comments and special content (SVG, CSS, JavaScript)
        if matches!(
            self.state.current_type,
            TokenType::Comment | TokenType::Svg | TokenType::Css | TokenType::JavaScript | TokenType::HtmlContent
        ) {
            if self.tag_state == TagState::None {
                if c == ' ' {
                    if self.builder.len() == 1 {
                        return false;
                    }
                    self.tag_state = TagState::Starting;
                    // Extract and store the tag name when the space after it is reached
                    self.current_tag_name = self.get_tag_name();
                } else if c == '/' {
                    self.tag_state = if self.builder.len() == 1 {
                        TagState::Closing
                    } else {
                        TagState::SelfClosing
                    };
                } else if !(c.is_ascii_alphabetic()
                    || (is_digit(c) && self.builder.len() == 2 && self.builder[1] == 'h'))
                {
                    return false;
                }
            } else {
                self.state.current_type = TokenType::Tag;
            }
        } else if self.tag_state == TagState::SelfClosing
            || (self.tag_state == TagState::Closing && !c.is_ascii_alphabetic())
        {
            return false;
        }
        true
    }

    fn get_tag_name(&self) -> String {
        // "<script>" -> "script", "</div>" -> "div"
        let len = self.builder.len();
        if len < 2 {
            return String::new();
        }
        let start = if self.builder[0] == '<' && self.builder[1] == '/' {
            2
        } else {
            1
        };
        let mut end = len;
        for i in start..len {
            let ch = self.builder[i];
            if ch == ' ' || ch == '/' || ch == '>' {
                end = i;
                break;
            }
        }
        if end <= start {
            return String::new();
        }
        self.builder[start..end].iter().collect()
    }

    pub(super) fn check_html_comment(&mut self) {
        let len = self.builder.len();
        let b = &self.builder;
        let starts_open = len >= 4 && b[0] == '<' && b[1] == '!' && b[2] == '-' && b[3] == '-';
        let ends_close = len >= 3 && b[len - 1] == '>' && b[len - 2] == '-' && b[len - 3] == '-';

        if starts_open || ends_close || self.in_html_comment {
            self.state.current_type = TokenType::HtmlComment;
        }

        // Track the open/close state across lines for multi-line HTML comments
        if starts_open && !ends_close {
            self.in_html_comment = true;
        } else if ends_close {
            self.in_html_comment = false;
        }
    }
}
