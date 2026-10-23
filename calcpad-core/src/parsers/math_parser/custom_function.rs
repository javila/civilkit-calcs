//! Port of Parsers/MathParser/MathParser.CustomFunction.cs and Container.cs.
//!
//! C# has CustomFunction1/2/3/N subclasses whose only behavioural difference is the result
//! cache of the one and two parameter functions (CustomFunction1/2): results are kept per
//! scalar argument value, up to 1000 entries, and cleared whenever a dependency changes.
//! One struct covers all arities here.
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::rc::Rc;

use super::compiler::Expr;
use super::tokens::{Rpn, TokenTypes};
use crate::base_types::unit::{OptUnit, Unit};
use crate::base_types::value::Value;
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
    cache: RefCell<ResultCache>,
}

/// `MaxCacheSize`
pub const MAX_CACHE_SIZE: usize = 1000;

/// A cached result; `None` when the function threw while computing it (C# leaves the
/// default, null, in the dictionary).
pub type CachedResult = Option<Value>;

/// Hasher for the cache's `u64` keys (already a mix of the argument bits): a multiply and two
/// shifts instead of SipHash. The map is only looked up, never iterated, so the order of its
/// entries cannot show in any output.
#[derive(Default)]
pub struct MixHasher(u64);

impl Hasher for MixHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_u64(b as u64);
        }
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        let mut x = self.0 ^ i;
        x ^= x >> 32;
        x = x.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        self.0 = x ^ (x >> 29);
    }
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// The `Dictionary<IScalarValue, IValue>` (or `Dictionary<Tuple, IValue>`) of CustomFunction1/2.
#[derive(Default)]
pub struct ResultCache {
    buckets: HashMap<u64, Vec<(Vec<Value>, CachedResult)>, BuildHasherDefault<MixHasher>>,
    count: usize,
}

/// The hash part of a scalar key: `double.GetHashCode` treats 0 and -0, and all NaNs, alike.
fn key_bits(v: &Value) -> u64 {
    let norm = |d: f64| -> u64 {
        if d.is_nan() {
            f64::NAN.to_bits()
        } else if d == 0.0 {
            0
        } else {
            d.to_bits()
        }
    };
    match v {
        Value::Real(r) => norm(r.d),
        Value::Complex(c) => norm(c.a).wrapping_mul(31).wrapping_add(norm(c.b)),
        _ => 0,
    }
}

/// `double.Equals`: NaN equals NaN, 0 equals -0.
fn dbl_eq(a: f64, b: f64) -> bool {
    a == b || a.is_nan() && b.is_nan()
}

/// `Unit.Equals` together with the part of `Unit.GetHashCode` that it does not cover
/// (temperature differences hash differently).
fn units_eq(a: &OptUnit, b: &OptUnit) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => {
            let delta = |u: &crate::base_types::unit::Unit| u.is_temp() && u.text().starts_with('\u{0394}');
            Unit::equals(x, y) && delta(x) == delta(y)
        }
        _ => false,
    }
}

/// `RealValue.Equals(object)` / `ComplexValue.Equals(object)` as used by the dictionary.
fn key_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Real(x), Value::Real(y)) => dbl_eq(x.d, y.d) && units_eq(&x.units, &y.units),
        (Value::Complex(x), Value::Complex(y)) => dbl_eq(x.a, y.a) && dbl_eq(x.b, y.b) && units_eq(&x.units, &y.units),
        _ => false,
    }
}

impl ResultCache {
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn clear(&mut self) {
        self.buckets.clear();
        self.count = 0;
    }

    fn hash(args: &[Value]) -> u64 {
        args.iter()
            .fold(17u64, |h, v| h.wrapping_mul(1_000_003).wrapping_add(key_bits(v)))
    }

    pub fn get(&self, args: &[Value]) -> Option<CachedResult> {
        let bucket = self.buckets.get(&ResultCache::hash(args))?;
        bucket
            .iter()
            .find(|(k, _)| k.len() == args.len() && k.iter().zip(args).all(|(a, b)| key_eq(a, b)))
            .map(|(_, v)| v.clone())
    }

    pub fn insert(&mut self, args: Vec<Value>, value: CachedResult) {
        let h = ResultCache::hash(&args);
        let bucket = self.buckets.entry(h).or_default();
        if let Some(e) = bucket
            .iter_mut()
            .find(|(k, _)| k.len() == args.len() && k.iter().zip(&args).all(|(a, b)| key_eq(a, b)))
        {
            e.1 = value;
            return;
        }
        bucket.push((args, value));
        self.count += 1;
    }
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
            cache: RefCell::new(ResultCache::default()),
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

    /// Sets the parameter values without cloning the parameter list.
    pub fn set_parameter_values(&self, args: &[Value]) {
        for (p, a) in self.parameters.borrow().iter().zip(args) {
            p.set_value(a.clone());
        }
    }

    /// True for the functions that keep a result cache (CustomFunction1/2).
    pub fn has_cache(&self) -> bool {
        matches!(self.parameter_count.get(), 1 | 2)
    }

    pub fn cache(&self) -> &RefCell<ResultCache> {
        &self.cache
    }

    /// `PurgeCache()`
    pub fn purge_cache(&self) {
        if self.has_cache() && self.cache.borrow().len() >= MAX_CACHE_SIZE {
            self.cache.borrow_mut().clear();
        }
    }

    pub fn parameter_name(&self, index: usize) -> String {
        self.parameters.borrow()[index].name.clone()
    }

    pub fn index_of_parameter(&self, name: &str) -> Option<usize> {
        self.parameters.borrow().iter().position(|p| p.name == name)
    }

    /// `ClearCache()`
    pub fn clear_cache(&self) {
        *self.function.borrow_mut() = None;
        self.cache.borrow_mut().clear();
    }

    pub fn clear(&self) {
        self.clear_cache();
        self.on_change.invoke();
    }

    /// `CheckRecursion(f, functions)`. Errors are reported as `Err(())` (C# catches and maps).
    pub fn check_recursion(self: &Rc<Self>, f: Option<&Rc<CustomFunction>>, functions: &Container) -> Result<bool, ()> {
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

fn check_recursion_rpn(rpn: &Rpn, f: &Rc<CustomFunction>, functions: &Container) -> Result<bool, ()> {
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
