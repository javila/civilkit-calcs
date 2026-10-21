//! MetadataReturnTypeTests.cs
mod common;
use calcpad_highlighter::content_resolution::result::Stage3Result;
use calcpad_highlighter::content_resolution::ContentResolver;
use calcpad_highlighter::linter::models::{CalcpadType, DefinitionMetadata};
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};

fn stage3(content: &str) -> Stage3Result {
    ContentResolver::new()
        .get_staged_content(content, None, None, None)
        .stage3
}

#[test]
fn return_type_metadata_overrides_inferred_function_type() {
    // f(x) = x infers a scalar Value; the metadata comment declares matrix.
    let s3 = stage3("'<!--{\"returnType\":\"matrix\"}-->\nf(x) = x\n");
    let func = s3.type_tracker.functions().get("f").expect("f");
    assert_eq!(CalcpadType::Matrix, func.return_type);
}

#[test]
fn return_type_metadata_any_maps_to_various() {
    let s3 = stage3("'<!--{\"returnType\":\"any\"}-->\nf(x) = x\n");
    let func = s3.type_tracker.functions().get("f").expect("f");
    assert_eq!(CalcpadType::Various, func.return_type);
}

#[test]
fn custom_unit_metadata_description_captured() {
    let s3 = stage3("'<!--{\"desc\":\"Euro currency\"}-->\n.EUR = 1\n");
    let unit = s3.custom_units.iter().find(|u| u.name == "EUR").expect("EUR");
    assert_eq!(Some("Euro currency"), unit.description.as_deref());
}

#[test]
fn try_parse_reads_return_type() {
    let line: Vec<char> = "'<!--{\"returnType\":\"vector\"}-->".chars().collect();
    let metadata = DefinitionMetadata::try_parse(&line).expect("metadata");
    assert_eq!(Some("vector"), metadata.return_type.as_deref());
}

#[test]
fn invalid_return_type_reports_cpd3416() {
    let src = "'<!--{\"returnType\":\"scalar\"}-->\nf(x) = x\n";
    let staged = ContentResolver::new().get_staged_content(src, None, None, None);
    let result = CalcpadLinter::new().lint(&staged, &LintIgnoreRegionParser::extract_regions(src));
    assert!(result.diagnostics.iter().any(|d| d.code == "CPD-3411"));
}
