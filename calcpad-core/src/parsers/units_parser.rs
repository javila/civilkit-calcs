//! Port of Parsers/UnitsParser.cs (target units after `|`).
use std::collections::HashMap;
use std::rc::Rc;

use crate::base_types::unit::{OptUnit, Unit, UnitRef};
use crate::exceptions;
use crate::extension_methods::trim;
use crate::math_parser_exception::CResult;
use crate::netfmt;
use crate::validator;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TokenTypes {
    None = 0,
    Constant = 1,
    Units = 2,
    Operator = 3,
    BracketLeft = 4,
    BracketRight = 5,
}

const DEFAULT_ORDER: i32 = -1;

#[derive(Clone)]
struct Token {
    order: i32,
    content: String,
    ttype: TokenTypes,
    unit: Option<UnitRef>,
    value: f64,
}

impl Token {
    fn new(content: &str, ttype: TokenTypes) -> Token {
        Token {
            order: DEFAULT_ORDER,
            content: content.to_string(),
            ttype,
            unit: None,
            value: 0.0,
        }
    }

    fn unit(u: UnitRef) -> Token {
        let mut content = u.text();
        if content.is_empty() {
            content = u.format_string_with_prefix().unwrap_or_default();
        }
        Token {
            order: DEFAULT_ORDER,
            content,
            ttype: TokenTypes::Units,
            unit: Some(u),
            value: 0.0,
        }
    }

    fn value(d: f64) -> Token {
        Token {
            order: DEFAULT_ORDER,
            content: netfmt::double_to_string(d),
            ttype: TokenTypes::Constant,
            unit: None,
            value: d,
        }
    }
}

const CORRECT_ORDER: [[bool; 6]; 6] = [
    [true, true, true, true, true, true],
    [true, false, false, true, false, true],
    [true, false, false, true, false, true],
    [true, true, true, false, true, false],
    [true, true, true, false, true, false],
    [true, false, false, true, false, true],
];

fn check(input: &[Token]) -> CResult<()> {
    let mut count_of_brackets = 0;
    let mut pt = Token::new("", TokenTypes::None);
    for t in input {
        if t.ttype == TokenTypes::BracketLeft {
            count_of_brackets += 1;
        } else if t.ttype == TokenTypes::BracketRight {
            count_of_brackets -= 1;
            if count_of_brackets < 0 {
                return Err(exceptions::InvalidUnits(&t.content));
            }
        }
        if !CORRECT_ORDER[pt.ttype as usize][t.ttype as usize] {
            return Err(exceptions::InvalidSyntax(&format!(
                "{} {}",
                pt.content, t.content
            )));
        }
        pt = t.clone();
    }
    if pt.ttype == TokenTypes::Operator || pt.ttype == TokenTypes::BracketLeft {
        return Err(exceptions::IncompleteExpression());
    }
    if count_of_brackets > 0 {
        return Err(exceptions::MissingRightBracket());
    }
    if count_of_brackets < 0 {
        return Err(exceptions::MissingLeftBracket());
    }
    Ok(())
}

fn operator_order(op: &str) -> i32 {
    match op.chars().next() {
        Some('^') => 0,
        Some('/') => 1,
        Some('*') => 2,
        _ => -1,
    }
}

/// `UnitsParser.Parse`
pub fn parse(
    expression: &str,
    units: Option<&HashMap<String, OptUnit>>,
    is_us: bool,
) -> CResult<OptUnit> {
    let input = get_input(expression, units, is_us)?;
    check(&input)?;
    let mut rpn = get_rpn(input);
    evaluate(&mut rpn)
}

fn get_input(
    expression: &str,
    units: Option<&HashMap<String, OptUnit>>,
    is_us: bool,
) -> CResult<Vec<Token>> {
    let ex: Vec<char> = expression.chars().collect();
    let mut tokens: Vec<Token> = Vec::new();
    let mut pt = TokenTypes::None;
    let mut ts_start = 0usize;
    let mut ts_end = 0usize;
    let len = ex.len();
    let add_literal = |literal: &str, pt: TokenTypes, tokens: &mut Vec<Token>| -> CResult<()> {
        if pt == TokenTypes::Units {
            let mut u: OptUnit = None;
            let mut found_custom = false;
            if let Some(us) = units {
                if let Some(cu) = us.get(literal) {
                    found_custom = true;
                    u = cu.clone();
                }
            }
            if !found_custom {
                u = Unit::try_get(literal, is_us);
            }
            match u {
                None => return Err(exceptions::InvalidUnits(literal)),
                Some(u) => tokens.push(Token::unit(u)),
            }
        } else {
            match parse_double(literal) {
                Some(d) => tokens.push(Token::value(d)),
                None => return Err(exceptions::InvalidNumber(literal)),
            }
        }
        Ok(())
    };
    let mut i = 0;
    while i < len {
        let c = ex[i];
        let tt = if validator::is_letter(c) {
            TokenTypes::Units
        } else if c == '-' || validator::is_digit(c) {
            TokenTypes::Constant
        } else if c == '*' || c == '/' || c == '^' {
            TokenTypes::Operator
        } else if c == '(' {
            TokenTypes::BracketLeft
        } else if c == ')' {
            TokenTypes::BracketRight
        } else if c == ' ' || c == ':' {
            TokenTypes::None
        } else {
            return Err(exceptions::InvalidSymbol(c));
        };
        if tt == TokenTypes::Constant || tt == TokenTypes::Units {
            ts_end += 1;
            if ts_end - ts_start > 1 && (pt != tt || c == '-') {
                return Err(exceptions::InvalidSymbol(c));
            }
        } else {
            if ts_end > ts_start {
                let literal: String = ex[ts_start..ts_end].iter().collect();
                add_literal(&literal, pt, &mut tokens)?;
                ts_start = i + 1;
                ts_end = i + 1;
                if c == ':' {
                    let rest: String = ex[i + 1..].iter().collect();
                    let s = trim(&rest).to_string();
                    if validator::is_valid_format_string(&s) {
                        if !tokens.is_empty() {
                            tokens.push(Token::new("*", TokenTypes::Operator));
                        }
                        let fu = Unit::get_formatting_unit(&format!(":{}", s));
                        tokens.push(Token::unit(fu));
                        return Ok(tokens);
                    } else {
                        return Err(exceptions::InvalidFormatString(&s));
                    }
                }
            } else {
                ts_start = i + 1;
                ts_end = i + 1;
            }
            if tt != TokenTypes::None {
                tokens.push(Token::new(&c.to_string(), tt));
            }
        }
        pt = tt;
        i += 1;
    }
    if ts_end > ts_start {
        let literal: String = ex[ts_start..ts_end].iter().collect();
        add_literal(&literal, pt, &mut tokens)?;
    }
    Ok(tokens)
}

/// `double.TryParse(literal, out d)` (current culture en-US / invariant).
fn parse_double(s: &str) -> Option<f64> {
    let t = trim(s);
    if t.is_empty() || !t.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    if !t
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E' | ','))
    {
        return None;
    }
    t.replace(',', "").parse::<f64>().ok()
}

fn get_rpn(input: Vec<Token>) -> Vec<Token> {
    let mut output: Vec<Token> = Vec::new();
    let mut stack: Vec<Token> = Vec::new();
    for t in input {
        match t.ttype {
            TokenTypes::Constant | TokenTypes::Units => output.push(t),
            TokenTypes::Operator => {
                let t_order = operator_order(&t.content);
                while let Some(next) = stack.last_mut() {
                    next.order = operator_order(&next.content);
                    if next.ttype == TokenTypes::Operator && next.order > t_order
                        || next.ttype == TokenTypes::BracketLeft
                    {
                        break;
                    }
                    let mut n = stack.pop().unwrap();
                    n.order = operator_order(&n.content);
                    output.push(n);
                }
                let mut nt = t.clone();
                nt.order = t_order;
                stack.push(nt);
            }
            TokenTypes::BracketLeft => stack.push(t),
            TokenTypes::BracketRight => {
                while let Some(next) = stack.last() {
                    if next.ttype == TokenTypes::BracketLeft {
                        stack.pop();
                        break;
                    }
                    output.push(stack.pop().unwrap());
                }
            }
            _ => {}
        }
    }
    while let Some(t) = stack.pop() {
        output.push(t);
    }
    output
}

fn evaluate(rpn: &mut [Token]) -> CResult<OptUnit> {
    let mut stack: Vec<Token> = Vec::new();
    for t in rpn.iter() {
        match t.ttype {
            TokenTypes::Constant | TokenTypes::Units => stack.push(t.clone()),
            TokenTypes::Operator => {
                let b = stack.pop().ok_or_else(exceptions::MissingOperand)?;
                let a = stack.pop().ok_or_else(exceptions::MissingOperand)?;
                stack.push(evaluate_operator(t, a, b)?);
            }
            _ => {
                return Err(exceptions::InvalidLiteral(
                    &t.content,
                    &format!("{:?}", t.ttype),
                ))
            }
        }
    }
    let t = match stack.pop() {
        None => return Ok(None),
        Some(t) => t,
    };
    if t.ttype == TokenTypes::Units {
        let u = t.unit.clone().unwrap();
        let s = render_expression(rpn);
        if s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            u.set_text(&format!("\u{00B7}{}", s));
        } else {
            u.set_text(&s);
        }
        return Ok(Some(u));
    }
    Err(exceptions::ResultIsNotUnits())
}

fn evaluate_operator(t: &Token, a: Token, b: Token) -> CResult<Token> {
    let c = t.content.chars().next().unwrap_or('\0');
    match (a.ttype == TokenTypes::Units, b.ttype == TokenTypes::Units) {
        (true, true) => {
            let ub = b.unit.clone().unwrap();
            if c == '*' && ub.is_dimensionless() {
                return Ok(a);
            }
            let ua = a.unit.clone().unwrap();
            let (u, d) = match c {
                '*' => Unit::multiply(&Some(ua), &Some(ub), false),
                '/' => Unit::divide(&Some(ua), &Some(ub), false),
                '^' => return Err(exceptions::PowerNotUnitless()),
                _ => return Err(exceptions::InvalidOperator(c)),
            };
            Ok(match u {
                None => Token::value(d),
                Some(u) => Token::unit(Unit::mul_f(&u, d)),
            })
        }
        (true, false) => {
            let ua = a.unit.clone().unwrap();
            Ok(match c {
                '*' => Token::unit(Unit::mul_f(&ua, b.value)),
                '/' => Token::unit(Unit::div_f(&ua, b.value)),
                '^' => match ua.pow(b.value as f32) {
                    Some(u) => Token::unit(u),
                    None => {
                        return Err(
                            crate::math_parser_exception::MathParserException::unexpected(
                                "Object reference not set to an instance of an object.",
                            ),
                        )
                    }
                },
                _ => return Err(exceptions::InvalidOperator(c)),
            })
        }
        (false, true) => {
            let ub = b.unit.clone().unwrap();
            Ok(match c {
                '*' => Token::unit(Unit::mul_f(&ub, a.value)),
                '/' => match Unit::f_div(a.value, &ub) {
                    Some(u) => Token::unit(u),
                    None => {
                        return Err(
                            crate::math_parser_exception::MathParserException::unexpected(
                                "Object reference not set to an instance of an object.",
                            ),
                        )
                    }
                },
                '^' => return Err(exceptions::PowerNotUnitless()),
                _ => return Err(exceptions::InvalidOperator(c)),
            })
        }
        (false, false) => Ok(match c {
            '*' => Token::value(a.value * b.value),
            '/' => Token::value(a.value / b.value),
            '^' => Token::value(a.value.powf(b.value)),
            _ => return Err(exceptions::InvalidOperator(c)),
        }),
    }
}

fn is_negative(t: &Token) -> bool {
    t.order == DEFAULT_ORDER && t.content.starts_with('-')
}

fn render_expression(rpn: &mut [Token]) -> String {
    let mut stack: Vec<Token> = Vec::new();
    for idx in 0..rpn.len() {
        let t = rpn[idx].clone();
        if t.ttype == TokenTypes::Constant || t.ttype == TokenTypes::Units {
            stack.push(t);
        } else {
            let b = match stack.pop() {
                Some(b) => b,
                None => continue,
            };
            let mut a = stack
                .pop()
                .unwrap_or_else(|| Token::new("", TokenTypes::None));
            if a.order > t.order {
                a.content = format!("({})", a.content);
            }
            let tc = t.content.chars().next().unwrap_or('\0');
            let mut sa = a.content.clone();
            let mut sb = b.content.clone();
            if tc == '^' {
                if is_negative(&a) {
                    sa = format!("({})", sa);
                }
                if is_negative(&b) || b.order != DEFAULT_ORDER {
                    sb = format!("({})", sb);
                }
            } else if b.order > t.order || b.order == t.order && tc == '/' || is_negative(&b) {
                sb = format!("({})", sb);
            }
            let s = if sb.starts_with(':') {
                format!("{}{}", sa, sb)
            } else {
                format!("{}{}{}", sa, tc, sb)
            };
            let mut c = Token::new(&s, t.ttype);
            c.order = t.order;
            stack.push(c);
        }
    }
    let _ = Rc::new(());
    stack.pop().map(|t| t.content).unwrap_or_default()
}
