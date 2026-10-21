//! HtmlComment/: extraction of the JSON payloads of `'<!--{...}-->` metadata comments and the
//! PDF settings payload.
pub mod html_comment_parser;
pub mod pdf_settings_dto;

pub use html_comment_parser::{HtmlCommentBlock, HtmlCommentParseStatus, HtmlCommentParser};
pub use pdf_settings_dto::{PdfPaperFormat, PdfSettingKey, PdfSettingsDefaults, PdfSettingsDto};
