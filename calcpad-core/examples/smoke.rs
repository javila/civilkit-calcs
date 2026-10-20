use std::cell::RefCell;
use std::rc::Rc;

use calcpad_core::{MathParser, MathSettings};

fn main() {
    let settings = Rc::new(RefCell::new(MathSettings::default()));
    let p = MathParser::new(settings);
    p.is_calculation.set(true);
    for e in std::env::args().skip(1) {
        let r = p
            .parse(&e, true)
            .and_then(|_| p.calculate(true, -1))
            .and_then(|_| p.to_html());
        match r {
            Ok(h) => println!("{} => {} | {}", e, p.result_as_string(), h),
            Err(err) => println!("{} => ERROR {}", e, err),
        }
    }
}
