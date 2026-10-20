//! Port of Parsers/MathParser/MathParser.CustomFunction.cs and Container.cs.
//!
//! C# has CustomFunction1/2/3/N subclasses whose only behavioural difference is a result
//! cache. The cache is transparent (it is cleared whenever a dependency changes), so one
//! struct covers all arities here.
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use super::compiler::Expr;
use super::tokens::{Rpn, TokenTypes};
use crate::base_types::unit::OptUnit;
use crate::base_types::variable::{ChangeEvent, ChangeListener, Parameter};

pub struct CustomFunction {
    pub on_change: ChangeEvent,
    pub rpn: RefCell<Rpn>,
    pub units: RefCell<OptUnit>,
    pub parameter_count: Cell<usize>,
    pub is_recursion: Cell<bool>,
    pub function: RefCell<Option<Rc<Expr>>>,
    pub is_read_only: Cell<bool>,
    pub defaults: RefCell<Option<Vec<Option<Rpn>>>>,
    pub default_functions: RefCell<Option<Vec<Option<Rc<Expr>>>>>,
    parameters: RefCell<Vec<Parameter>>,
}

pub type CustomFunctionRef = Rc<CustomFunction>;

impl ChangeListener for CustomFunction {
    fn on_change(&self) {
        self.clear();
    }
}

impl CustomFunction {
    pub fn new(is_read_only: bool) -> CustomFunctionRef {
        Rc::new(CustomFunction {
            on_change: ChangeEvent::default(),
            rpn: RefCell::new(Rc::new(Vec::new())),
            units: RefCell::new(None),
            parameter_count: Cell::new(0),
            is_recursion: Cell::new(false),
            function: RefCell::new(None),
            is_read_only: Cell::new(is_read_only),
            defaults: RefCell::new(None),
            default_functions: RefCell::new(None),
            parameters: RefCell::new(Vec::new()),
        })
    }

    pub fn parameter_count(&self) -> usize {
        self.parameter_count.get()
    }

    pub fn add_parameters(&self, parameters: &[String]) {
        self.parameter_count.set(parameters.len());
        *self.parameters.borrow_mut() = parameters.iter().map(|p| Parameter::new(p)).collect();
    }

    pub fn parameters(&self) -> Vec<Parameter> {
        self.parameters.borrow().clone()
    }

    pub fn parameter_name(&self, index: usize) -> String {
        self.parameters.borrow()[index].name.clone()
    }

    pub fn index_of_parameter(&self, name: &str) -> Option<usize> {
        self.parameters.borrow().iter().position(|p| p.name == name)
    }

    pub fn clear_cache(&self) {
        *self.function.borrow_mut() = None;
    }

    pub fn clear(&self) {
        self.clear_cache();
        self.on_change.invoke();
    }

    /// `CheckRecursion(f, functions)`. Errors are reported as `Err(())` (C# catches and maps).
    pub fn check_recursion(
        self: &Rc<Self>,
        f: Option<&Rc<CustomFunction>>,
        functions: &Container,
    ) -> Result<bool, ()> {
        if let Some(f) = f {
            if Rc::ptr_eq(f, self) {
                return Ok(true);
            }
        }
        let f = f.cloned().unwrap_or_else(|| self.clone());
        if check_recursion_rpn(&self.rpn.borrow(), &f, functions)? {
            return Ok(true);
        }
        if let Some(defaults) = &*self.defaults.borrow() {
            for rpn in defaults.iter().flatten() {
                if check_recursion_rpn(rpn, &f, functions)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

fn check_recursion_rpn(
    rpn: &Rpn,
    f: &Rc<CustomFunction>,
    functions: &Container,
) -> Result<bool, ()> {
    for t in rpn.iter() {
        let (tt, index) = {
            let t = t.borrow();
            (t.ttype, t.index)
        };
        if tt == TokenTypes::CustomFunction {
            let cf = match functions.get(index) {
                Some(cf) => cf,
                None => return Ok(true),
            };
            match cf.check_recursion(Some(f), functions) {
                Ok(true) => return Ok(true),
                Ok(false) => {}
                Err(_) => return Ok(true),
            }
        }
    }
    Ok(false)
}

/// Port of Container.cs specialised for custom functions.
#[derive(Default)]
pub struct Container {
    index: HashMap<String, usize>,
    values: Vec<CustomFunctionRef>,
    pub last_name: String,
}

impl Container {
    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn add(&mut self, name: &str, value: CustomFunctionRef) -> usize {
        let i = if let Some(&i) = self.index.get(name) {
            self.values[i] = value;
            i
        } else {
            let i = self.values.len();
            self.index.insert(name.to_string(), i);
            self.values.push(value);
            i
        };
        self.last_name = name.to_string();
        i
    }

    pub fn index_of(&self, name: &str) -> i64 {
        self.index.get(name).map(|&i| i as i64).unwrap_or(-1)
    }

    pub fn get(&self, index: i64) -> Option<CustomFunctionRef> {
        if index < 0 {
            return None;
        }
        self.values.get(index as usize).cloned()
    }

    pub fn all(&self) -> Vec<CustomFunctionRef> {
        self.values.clone()
    }
}
