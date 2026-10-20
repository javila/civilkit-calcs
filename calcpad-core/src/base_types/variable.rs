//! Port of BaseTypes/Variable.cs and BaseTypes/Parameter.cs.
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use crate::base_types::complex::Complex;
use crate::base_types::complex_value::ComplexValue;
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::OptUnit;
use crate::base_types::value::Value;

/// Receiver of `OnChange` events (custom functions and solver blocks).
pub trait ChangeListener {
    fn on_change(&self);
}

/// The C# `event Action OnChange`.
#[derive(Default)]
pub struct ChangeEvent {
    listeners: RefCell<Vec<Weak<dyn ChangeListener>>>,
}

impl ChangeEvent {
    pub fn subscribe(&self, l: Weak<dyn ChangeListener>) {
        self.listeners.borrow_mut().push(l);
    }

    pub fn invoke(&self) {
        let list: Vec<Weak<dyn ChangeListener>> = self.listeners.borrow().clone();
        for l in list {
            if let Some(l) = l.upgrade() {
                l.on_change();
            }
        }
    }
}

#[derive(Default)]
pub struct Variable {
    value: RefCell<Option<Value>>,
    initialized: Cell<bool>,
    pub is_read_only: Cell<bool>,
    pub on_change: ChangeEvent,
}

pub type VarRef = Rc<Variable>;

impl std::fmt::Debug for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Variable({:?})", self.value.borrow())
    }
}

impl Variable {
    pub fn new() -> VarRef {
        Rc::new(Variable::default())
    }

    pub fn with_value(value: Value) -> VarRef {
        let v = Variable::default();
        *v.value.borrow_mut() = Some(value);
        v.initialized.set(true);
        Rc::new(v)
    }

    pub fn with_complex(c: Complex) -> VarRef {
        Variable::with_value(Value::Complex(ComplexValue::c(c)))
    }

    pub fn value(&self) -> Option<Value> {
        self.value.borrow().clone()
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized.get()
    }

    pub fn reset_initialized(&self) {
        self.initialized.set(false);
    }

    pub fn change(&self) {
        self.on_change.invoke();
    }

    pub fn set_number_c(&self, number: Complex) {
        let units: OptUnit = match &*self.value.borrow() {
            Some(Value::Real(r)) => r.units.clone(),
            Some(Value::Complex(c)) => c.units.clone(),
            _ => None,
        };
        *self.value.borrow_mut() = Some(Value::Complex(ComplexValue::from_c(number, units)));
    }

    pub fn set_number(&self, number: f64) {
        let cur = self.value.borrow().clone();
        let nv = match cur {
            Some(Value::Real(r)) => Value::Real(RealValue::new(number, r.units.clone())),
            Some(Value::Complex(c)) => Value::Real(RealValue::new(number, c.units.clone())),
            _ => Value::Complex(ComplexValue::num(number)),
        };
        *self.value.borrow_mut() = Some(nv);
    }

    pub fn set_units(&self, units: OptUnit) {
        let cur = self.value.borrow().clone();
        let nv = match cur {
            Some(Value::Real(r)) => Value::Real(RealValue::new(r.d, units)),
            Some(Value::Complex(c)) => Value::Complex(ComplexValue::new(c.a, c.b, units)),
            // ((Vector)Value).SetUnits(units)
            Some(Value::Vector(v)) => {
                let _ = v.set_units(&units);
                return;
            }
            _ => return,
        };
        *self.value.borrow_mut() = Some(nv);
    }

    /// `SetValue(Unit units)`: does not change the initialized flag.
    pub fn set_unit_value(&self, units: OptUnit) {
        *self.value.borrow_mut() = Some(Value::Real(RealValue::unit(units)));
    }

    pub fn set_value_units(&self, number: f64, units: OptUnit) {
        *self.value.borrow_mut() = Some(Value::Real(RealValue::new(number, units)));
        self.initialized.set(true);
    }

    pub fn set_value(&self, value: Value) {
        *self.value.borrow_mut() = Some(value);
        self.initialized.set(true);
    }

    pub fn assign(&self, value: Value) {
        *self.value.borrow_mut() = Some(value);
        self.initialized.set(true);
        self.on_change.invoke();
    }
}

/// Port of Parameter.cs: a named variable initialised to zero.
#[derive(Clone)]
pub struct Parameter {
    pub name: String,
    pub variable: VarRef,
}

impl Parameter {
    pub fn new(name: &str) -> Parameter {
        Parameter {
            name: name.to_string(),
            variable: Variable::with_value(Value::Real(RealValue::ZERO)),
        }
    }

    pub fn set_value(&self, value: Value) {
        self.variable.set_value(value);
    }
}
