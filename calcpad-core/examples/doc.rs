use calcpad_core::parsers::expression_parser::ExpressionParser;

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let mut p = ExpressionParser::new();
    p.debug = std::env::var("DEBUG").is_ok();
    p.parse(&src, true);
    println!("{}", p.html_result);
    for e in &p.errors_list {
        println!("ERR {:?}", e);
    }
}
