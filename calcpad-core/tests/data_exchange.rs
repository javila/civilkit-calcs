//! Transcribed by hand from CalcpadCE Calcpad.Tests/Data (0cf438f): WriteDirectiveTests.cs,
//! DataExchangeNameTests.cs and ReadDirectiveEmbeddingTests.cs. Each test works in its own
//! temporary folder through the default (disk) DataFiles.
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use calcpad_core::data_files::StdFiles;
use calcpad_core::parsers::expression_parser::{base64_encode, ExpressionParser};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A temporary folder, removed when dropped.
struct Dir(PathBuf);

impl Dir {
    fn new() -> Dir {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let p = std::env::temp_dir().join(format!("calcpad-rs-data-{}-{}", std::process::id(), n));
        std::fs::create_dir_all(&p).unwrap();
        Dir(p)
    }

    fn path(&self, name: &str) -> String {
        self.0.join(name).to_string_lossy().to_string()
    }

    fn run(&self, allow_write: bool, lines: &[&str]) -> String {
        let mut parser = ExpressionParser::new();
        parser.source_file_path = Some(self.path("worksheet.cpd"));
        parser.allow_data_write = allow_write;
        parser.parse(&lines.join("\n"), true);
        parser.html_result.clone()
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[track_caller]
fn contains(expected: &str, html: &str) {
    assert!(html.contains(expected), "expected {:?} in\n{}", expected, html);
}

#[track_caller]
fn not_contains(unexpected: &str, html: &str) {
    assert!(
        !html.contains(unexpected),
        "did not expect {:?} in\n{}",
        unexpected,
        html
    );
}

// ----- WriteDirectiveTests ---------------------------------------------------------------------

#[test]
fn by_default_the_file_is_written() {
    let d = Dir::new();
    let html = d.run(true, &["M = [1; 2|3; 4]", "#write M to out.csv"]);
    assert!(std::path::Path::new(&d.path("out.csv")).exists());
    contains("was successfully written to", &html);
}

#[test]
fn writing_off_leaves_no_file() {
    let d = Dir::new();
    let html = d.run(false, &["M = [1; 2|3; 4]", "#write M to out.csv"]);
    assert!(!std::path::Path::new(&d.path("out.csv")).exists());
    contains("will be written to", &html);
    not_contains("was successfully", &html);
}

#[test]
fn writing_off_does_not_link_the_target() {
    let d = Dir::new();
    let html = d.run(false, &["M = [1; 2]", "#write M to out.csv"]);
    not_contains("file:///", &html);
    contains("out.csv", &html);
}

#[test]
fn writing_off_still_appends_under_its_own_verb() {
    let d = Dir::new();
    contains(
        "will be appended to",
        &d.run(false, &["M = [1; 2]", "#append M to out.csv"]),
    );
}

#[test]
fn writing_off_leaves_an_existing_file_alone() {
    let d = Dir::new();
    std::fs::write(d.path("out.csv"), "keep me\n").unwrap();
    d.run(false, &["M = [1; 2|3; 4]", "#write M to out.csv"]);
    assert_eq!("keep me\n", std::fs::read_to_string(d.path("out.csv")).unwrap());
}

#[test]
fn an_undefined_variable_errors_either_way() {
    for allow in [true, false] {
        let d = Dir::new();
        contains("class=\"err\"", &d.run(allow, &["#write Missing to out.csv"]));
    }
}

#[test]
fn a_missing_target_errors_either_way() {
    for allow in [true, false] {
        let d = Dir::new();
        contains("class=\"err\"", &d.run(allow, &["M = [1; 2]", "#write M to"]));
    }
}

#[test]
fn a_missing_folder_errors_either_way() {
    for allow in [true, false] {
        let d = Dir::new();
        let html = d.run(allow, &["M = [1; 2]", "#write M to no_such_folder/out.csv"]);
        contains("class=\"err\"", &html);
        not_contains("written to", &html);
    }
}

#[test]
fn an_unsupported_excel_format_errors_either_way() {
    for allow in [true, false] {
        let d = Dir::new();
        contains(
            "class=\"err\"",
            &d.run(allow, &["M = [1; 2]", "#write M to out.xls@Sheet1"]),
        );
    }
}

#[test]
fn a_false_condition_skips_the_directive_either_way() {
    for allow in [true, false] {
        let d = Dir::new();
        let html = d.run(allow, &["M = [1; 2]", "#if 0", "#write M to out.csv", "#end if"]);
        assert!(!std::path::Path::new(&d.path("out.csv")).exists());
        not_contains("written to", &html);
    }
}

// ----- DataExchangeNameTests -------------------------------------------------------------------

fn data_dir() -> Dir {
    let d = Dir::new();
    std::fs::write(d.path("data.csv"), "1,2,3\n").unwrap();
    d
}

#[test]
fn read_accepts_a_valid_name() {
    let d = data_dir();
    not_contains("class=\"err\"", &d.run(true, &["#read v_max from data.csv type=V"]));
}

#[test]
fn read_rejects_an_invalid_name() {
    for name in ["a.b", "1abc", "a@b"] {
        let d = data_dir();
        let html = d.run(true, &[&format!("#read {} from data.csv type=V", name)]);
        contains(&format!("Invalid variable name: \"{}\".", name), &html);
    }
}

#[test]
fn write_rejects_an_invalid_name() {
    let d = data_dir();
    contains(
        "Invalid variable name: \"a.b\".",
        &d.run(true, &["#write a.b to out.csv"]),
    );
}

#[test]
fn append_rejects_an_invalid_name() {
    let d = data_dir();
    contains(
        "Invalid variable name: \"a.b\".",
        &d.run(true, &["#append a.b to out.csv"]),
    );
}

// ----- ReadDirectiveEmbeddingTests -------------------------------------------------------------

struct CsvDir(Dir);

impl CsvDir {
    fn new(csv: &str) -> CsvDir {
        let d = Dir::new();
        std::fs::write(d.path("data.csv"), csv).unwrap();
        CsvDir(d)
    }

    fn base64(&self) -> String {
        base64_encode(&std::fs::read(self.0.path("data.csv")).unwrap())
    }

    fn embed(&self, directive: &str) -> Result<Option<String>, String> {
        ExpressionParser::embed_read_directive(directive, Some(&self.0.path("worksheet.cpd")), &StdFiles)
            .map(|r| r.map(|(s, _)| s))
            .map_err(|e| e.message)
    }

    fn write(&self, lines: &[&str]) {
        not_contains("class=\"err\"", &self.0.run(true, lines));
    }

    /// What `expression` comes to once the embedded read has run.
    fn run(&self, directive: &str, expression: &str) -> f64 {
        let mut parser = ExpressionParser::new();
        let source = format!("{}\n{}", self.embed(directive).unwrap().unwrap(), expression);
        parser.parse(&source, true);
        let html = parser.html_result.clone();
        not_contains("class=\"err\"", &html);
        result(&html)
    }
}

/// The number the last line rendered to, read back out of the report.
fn result(html: &str) -> f64 {
    let end = html.rfind("</p>").unwrap();
    let mut text = String::new();
    let mut in_tag = false;
    for c in html[..end].chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    let i = text.rfind('=').unwrap();
    text[i + 1..].trim().replace('\u{2212}', "-").parse().unwrap()
}

#[test]
fn embedded_directive_keeps_everything_but_the_source() {
    let d = CsvDir::new("1,3\n2,4\n");
    assert_eq!(
        Some(format!("#read M from data:text/csv;base64,{} type=R_hp", d.base64())),
        d.embed("#read M from data.csv type=R_hp").unwrap()
    );
}

#[test]
fn embedded_directive_keeps_the_separator_and_the_range() {
    let d = CsvDir::new("1;2;3\n4;5;6\n7;8;9\n");
    assert_eq!(
        Some(format!(
            "#read M from data:text/csv;base64,{}@R2C2:R3C3 sep=';'",
            d.base64()
        )),
        d.embed("#read M from data.csv@R2C2:R3C3 sep=';'").unwrap()
    );
}

#[test]
fn embedded_read_assigns_what_the_original_did() {
    for (options, csv, cell, expected) in [
        ("", "1,2\n3,4\n", "M.(2; 1)", 3.0),
        ("type=V", "1,3\n2,4\n", "v.(2)", 3.0),
        ("type=C", "1\n2\n3\n", "M.(3; 1)", 3.0),
        ("type=D", "1,2,3\n", "M.(2; 2)", 2.0),
        ("type=L", "1\n2,3\n4,5,6\n", "M.(3; 2)", 5.0),
        ("type=U", "1,2,3\n4,5\n6\n", "M.(2; 3)", 5.0),
        ("type=S", "1,2\n3\n", "M.(2; 1)", 2.0),
        ("sep=';'", "1;2\n3;4\n", "M.(1; 2)", 2.0),
    ] {
        let d = CsvDir::new(csv);
        let name = if options == "type=V" { "v" } else { "M" };
        assert_eq!(
            expected,
            d.run(&format!("#read {} from data.csv {}", name, options), cell),
            "{}",
            options
        );
    }
}

#[test]
fn high_performance_read_stays_high_performance() {
    for options in ["type=R_hp", "type=S_hp", "type=V_hp"] {
        let d = CsvDir::new("1,2\n3,4\n");
        assert_eq!(
            1.0,
            d.run(&format!("#read M from data.csv {}", options), "ishp(M)"),
            "{}",
            options
        );
    }
}

#[test]
fn high_performance_read_keeps_its_shape() {
    let d = CsvDir::new("1,2\n3\n");
    assert_eq!(2.0, d.run("#read M from data.csv type=S_hp", "M.(2; 1)"));
}

#[test]
fn single_row_is_a_matrix_of_one_row() {
    let d = CsvDir::new("1,2,3\n");
    assert_eq!(3.0, d.run("#read M from data.csv", "n_cols(M)"));
}

#[test]
fn scientific_notation_is_read() {
    let d = CsvDir::new("7.5e-06,2e3\n");
    assert_eq!(7.5, d.run("#read M from data.csv", "M.(1; 1)*10^6"));
    assert_eq!(2000.0, d.run("#read M from data.csv", "M.(1; 2)"));
}

#[test]
fn units_on_cells_are_kept() {
    let d = CsvDir::new("1m,2m\n");
    assert_eq!(3.0, d.run("#read M from data.csv", "M.(1; 1)/1m + M.(1; 2)/1m"));
}

#[test]
fn empty_cells_read_as_zero() {
    let d = CsvDir::new("1,,3\n");
    assert_eq!(0.0, d.run("#read M from data.csv", "M.(1; 2)"));
}

#[test]
fn missing_file_throws() {
    let d = CsvDir::new("1\n");
    assert!(d.embed("#read M from absent.csv").is_err());
}

#[test]
fn excel_read_is_carried_as_a_workbook() {
    let d = CsvDir::new("1\n");
    d.write(&["M = [1; 2|3; 4]", "#write M to book.xlsx@Sheet1!A1:B2"]);
    let directive = d.embed("#read N from book.xlsx@Sheet1!A1:B2").unwrap().unwrap();
    assert!(directive
        .starts_with("#read N from data:application/vnd.openxmlformats-officedocument.spreadsheetml.sheet;base64,"));
    assert!(directive.ends_with("@Sheet1!A1:B2"));
    assert_eq!(3.0, d.run("#read N from book.xlsx@Sheet1!A1:B2", "N.(2; 1)"));
}

#[test]
fn an_embedded_source_is_not_a_path() {
    assert!(ExpressionParser::try_get_data_path("#read M from data:text/csv;base64,MSwyCg== type=R").is_none());
}

#[test]
fn a_write_to_an_embedded_target_throws() {
    let mut calc = ExpressionParser::new();
    calc.parse("M = [1; 2]\n#write M to data:text/csv;base64,MSwyCg==", true);
    contains("Invalid syntax", &calc.html_result);
}

#[test]
fn an_unknown_media_type_throws() {
    let mut calc = ExpressionParser::new();
    calc.parse("#read M from data:image/png;base64,MSwyCg==", true);
    contains("not supported", &calc.html_result);
}
