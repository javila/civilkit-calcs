//! Port of Parsers/MathParser/MathParser.Evaluator.cs (RPN interpreter).
use std::rc::Rc;

use super::custom_function::CustomFunctionRef;
use super::input::out_of_scope;
use super::is_assignment;
use super::tokens::{Rpn, TokenKind, TokenRef, TokenTypes};
use super::MathParser;
use crate::base_types::complex_value::ComplexValue;
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{Field, OptUnit, Unit, UnitRef};
use crate::base_types::value::Value;
use crate::calculator::{index_out_of_range, NEG_STR};
use crate::exceptions;
use crate::math_parser_exception::CResult;

struct Stack {
    values: Vec<Value>,
    base: isize,
}

impl Stack {
    fn len(&self) -> isize {
        self.values.len() as isize
    }
    fn push(&mut self, v: Value) {
        self.values.push(v);
    }
    fn pop(&mut self) -> CResult<Value> {
        if self.len() <= self.base {
            return Err(exceptions::MissingOperand());
        }
        Ok(self.values.pop().unwrap())
    }
    fn at_base(&self) -> bool {
        self.len() == self.base
    }
    fn pop_values(&mut self, count: usize) -> CResult<Vec<Value>> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(self.values.pop().ok_or_else(exceptions::StackEmpty)?);
        }
        v.reverse();
        Ok(v)
    }
}

impl MathParser {
    /// `Evaluator.Evaluate(rpn, isVisible)`
    pub(crate) fn evaluate(&self, rpn: &Rpn, is_visible: bool) -> CResult<Value> {
        let rpn_length = rpn.len();
        if rpn_length < 1 {
            return Err(exceptions::ExpressionEmpty());
        }
        let mut stack = Stack {
            values: Vec::with_capacity(16),
            base: 0,
        };
        let mut i0 = 0;
        if rpn[0].borrow().ttype == TokenTypes::Variable
            && is_assignment(&rpn[rpn_length - 1].borrow().content)
        {
            i0 = 1;
        }
        *self.backup_variable.borrow_mut() = (None, Some(Value::zero()));
        for t in rpn.iter().skip(i0) {
            if stack.len() < stack.base {
                return Err(exceptions::StackEmpty());
            }
            let (tt, content) = {
                let tb = t.borrow();
                (tb.ttype, tb.content.clone())
            };
            match tt {
                TokenTypes::Constant => {
                    let v = t
                        .borrow()
                        .token_value()
                        .cloned()
                        .unwrap_or_else(Value::zero);
                    stack.push(v);
                }
                TokenTypes::Unit | TokenTypes::Variable | TokenTypes::Input => {
                    let v = self.evaluate_token0(t)?;
                    stack.push(v);
                }
                TokenTypes::Operator if content == NEG_STR => {
                    if stack.at_base() {
                        return Err(exceptions::MissingOperand());
                    }
                    let a = stack.pop()?;
                    stack.push(a.neg());
                }
                TokenTypes::Function => {
                    if stack.at_base() {
                        return Err(exceptions::MissingOperand());
                    }
                    let a = stack.pop()?;
                    let index = t.borrow().index;
                    stack.push(Value::evaluate_function(self.calc(), index, &a)?);
                }
                TokenTypes::VectorFunction
                | TokenTypes::MatrixFunction
                | TokenTypes::MatrixOptionalFunction => {
                    self.note_out_of_scope(&format!("vector/matrix function {}", content));
                    return Err(out_of_scope("Vector and matrix functions"));
                }
                TokenTypes::Operator if is_assignment(&content) => {
                    if stack.at_base() {
                        return Err(exceptions::MissingOperand());
                    }
                    let b = stack.pop()?;
                    return self.evaluate_assignment(b, rpn, is_visible);
                }
                TokenTypes::Operator | TokenTypes::Function2 => {
                    if stack.at_base() {
                        return Err(exceptions::MissingOperand());
                    }
                    let b = stack.pop()?;
                    if stack.at_base() {
                        return Err(exceptions::MissingOperand());
                    }
                    let c = stack.pop()?;
                    let index = t.borrow().index;
                    let r = if tt == TokenTypes::Operator {
                        Value::evaluate_operator(self.calc(), index, &c, &b)?
                    } else {
                        Value::evaluate_function2(self.calc(), index, &c, &b)?
                    };
                    stack.push(r);
                }
                TokenTypes::VectorFunction2
                | TokenTypes::MatrixFunction2
                | TokenTypes::MatrixIterativeFunction => {
                    self.note_out_of_scope(&format!("vector/matrix function {}", content));
                    return Err(out_of_scope("Vector and matrix functions"));
                }
                TokenTypes::Function3 => {
                    let c = stack.pop()?;
                    let b = stack.pop()?;
                    let a = stack.pop()?;
                    let index = t.borrow().index;
                    stack.push(
                        self.evaluator_calc()
                            .evaluate_function3(index, &a, &b, &c)?,
                    );
                }
                TokenTypes::VectorFunction3
                | TokenTypes::MatrixFunction3
                | TokenTypes::MatrixFunction4
                | TokenTypes::MatrixFunction5 => {
                    self.note_out_of_scope(&format!("vector/matrix function {}", content));
                    return Err(out_of_scope("Vector and matrix functions"));
                }
                TokenTypes::Interpolation => {
                    let (index, pc) = {
                        let tb = t.borrow();
                        (tb.index, tb.parameter_count())
                    };
                    let params = stack.pop_values(pc.max(0) as usize)?;
                    stack.push(
                        self.evaluator_calc()
                            .evaluate_interpolation(index, &params)?,
                    );
                }
                TokenTypes::MultiFunction => {
                    let (index, pc) = {
                        let tb = t.borrow();
                        (tb.index, tb.parameter_count())
                    };
                    let params = stack.pop_values(pc.max(0) as usize)?;
                    if index == crate::calculator::SWITCH_INDEX {
                        stack.push(Value::evaluate_switch(&params));
                    } else {
                        stack.push(
                            self.evaluator_calc()
                                .evaluate_multi_function(index, &params)?,
                        );
                    }
                }
                TokenTypes::VectorMultiFunction | TokenTypes::MatrixMultiFunction => {
                    self.note_out_of_scope(&format!("vector/matrix function {}", content));
                    return Err(out_of_scope("Vector and matrix functions"));
                }
                TokenTypes::CustomFunction => {
                    let v = self.evaluate_function_token(t, &mut stack)?;
                    stack.push(v);
                }
                TokenTypes::Vector
                | TokenTypes::RowDivisor
                | TokenTypes::Matrix
                | TokenTypes::VectorIndex
                | TokenTypes::MatrixIndex => {
                    self.note_out_of_scope("vectors and matrices");
                    return Err(out_of_scope("Vectors and matrices"));
                }
                TokenTypes::Solver => {
                    let index = t.borrow().index;
                    let sb = self.solve_blocks.borrow()[index as usize].clone();
                    sb.calculate(self)?;
                    stack.push(sb.result());
                }
                _ => return Err(exceptions::CannotEvaluateAsType(&content, &tt.name())),
            }
        }
        if stack.len() > stack.base {
            let mut v = stack.values.pop().unwrap();
            let tu = self.target_units.borrow().clone();
            let u = apply_units(&mut v, &tu)?;
            self.set_result_units(u);
            return Ok(v);
        }
        self.set_result_units(None);
        Ok(Value::zero())
    }

    /// `EvaluateToken(Token t)` for Unit, Variable and Input tokens.
    pub(crate) fn evaluate_token0(&self, t: &TokenRef) -> CResult<Value> {
        let tt = t.borrow().ttype;
        match tt {
            TokenTypes::Unit => {
                let tb = t.borrow();
                match &tb.kind {
                    TokenKind::Value(v) => Ok(v.clone()),
                    TokenKind::Variable(Some(var)) => {
                        var.value().ok_or_else(|| exceptions::InvalidOperand(""))
                    }
                    _ => Err(exceptions::InvalidOperand("")),
                }
            }
            TokenTypes::Variable => self.evaluate_variable_token(t),
            TokenTypes::Input if t.borrow().content == "?" => {
                Err(exceptions::UndefinedInputField())
            }
            _ => Ok(t
                .borrow()
                .token_value()
                .cloned()
                .unwrap_or_else(Value::zero)),
        }
    }

    fn evaluate_variable_token(&self, t: &TokenRef) -> CResult<Value> {
        let (v, content, tt) = {
            let tb = t.borrow();
            (tb.variable_ref(), tb.content.clone(), tb.ttype)
        };
        let v = match v {
            Some(v) => v,
            None => return Err(exceptions::UndefinedVariableOrUnits(&content)),
        };
        if v.is_initialized() || tt == TokenTypes::Vector || tt == TokenTypes::Matrix {
            return v
                .value()
                .ok_or_else(|| exceptions::UndefinedVariableOrUnits(&content));
        }
        let custom = self.units.borrow().get(&content).cloned();
        let u: OptUnit = match custom {
            Some(u) => u,
            None => match Unit::try_get(&content, self.is_us.get()) {
                Some(u) => Some(u),
                None => return Err(exceptions::UndefinedVariableOrUnits(&content)),
            },
        };
        t.borrow_mut().ttype = TokenTypes::Unit;
        v.set_unit_value(u);
        v.value()
            .ok_or_else(|| exceptions::UndefinedVariableOrUnits(&content))
    }

    fn evaluate_assignment(&self, b: Value, rpn: &Rpn, is_visible: bool) -> CResult<Value> {
        let mut b = b;
        let tu = self.target_units.borrow().clone();
        let u = apply_units(&mut b, &tu)?;
        self.set_result_units(u);
        if is_visible {
            self.normalize_units(&mut b);
        }
        let t0 = rpn[0].clone();
        let (t0_type, is_var, content) = {
            let t = t0.borrow();
            (t.ttype, t.is_variable_token(), t.content.clone())
        };
        if is_var {
            if t0_type == TokenTypes::Vector || t0_type == TokenTypes::Matrix {
                self.note_out_of_scope("vector/matrix element assignment");
                return Err(out_of_scope("Vector and matrix element assignment"));
            } else if t0_type == TokenTypes::Variable {
                let var = t0.borrow().variable_ref().unwrap();
                *self.backup_variable.borrow_mut() = (Some(content), var.value());
                var.assign(b.clone());
            }
        } else if t0_type == TokenTypes::Unit && t0.borrow().is_value_token() {
            let existing_units = t0.borrow().token_value().and_then(|v| v.units().clone());
            if let Some(eu) = existing_units {
                return Err(exceptions::CannotRewriteUnits(&eu.text()));
            }
            self.set_unit(&content, &b.as_complex())?;
            let u = self.get_unit(&content)?;
            t0.borrow_mut().set_value(Value::Real(RealValue::unit(u)));
        }
        Ok(b)
    }

    fn evaluate_function_token(&self, t: &TokenRef, stack: &mut Stack) -> CResult<Value> {
        let (index, map) = {
            let tb = t.borrow();
            (tb.index, tb.argument_map())
        };
        let cf = match self.functions.borrow().get(index) {
            Some(cf) => cf,
            None => return Err(index_out_of_range()),
        };
        let cf_param_count = map
            .as_ref()
            .map(|m| m.len())
            .unwrap_or(cf.parameter_count());
        if cf.is_recursion.get() {
            stack.base -= cf_param_count as isize;
            return Ok(Value::nan());
        }
        let args = stack.pop_values(cf_param_count)?;
        match map {
            Some(m) => self.evaluate_custom_function_map(&cf, &m, args),
            None => self.evaluate_custom_function(&cf, args),
        }
    }

    pub(crate) fn ensure_compiled(
        &self,
        cf: &CustomFunctionRef,
    ) -> CResult<Rc<super::compiler::Expr>> {
        if let Some(f) = cf.function.borrow().clone() {
            return Ok(f);
        }
        let rpn = cf.rpn.borrow().clone();
        let f = Rc::new(self.compile_rpn(&rpn, false)?);
        *cf.function.borrow_mut() = Some(f.clone());
        Ok(f)
    }

    /// `Evaluator.EvaluateFunction(cf, map, supplied)`
    pub(crate) fn evaluate_custom_function_map(
        &self,
        cf: &CustomFunctionRef,
        map: &[usize],
        supplied: Vec<Value>,
    ) -> CResult<Value> {
        if cf.function.borrow().is_none() {
            let rpn = cf.rpn.borrow().clone();
            let f = Rc::new(self.compile_rpn(&rpn, false)?);
            *cf.function.borrow_mut() = Some(f);
            if let Some(df) = cf.default_functions.borrow_mut().as_mut() {
                for x in df.iter_mut() {
                    *x = None;
                }
            }
        }
        let n = cf.parameter_count();
        let mut arguments: Vec<Option<Value>> = vec![None; n];
        for (k, v) in supplied.into_iter().enumerate() {
            arguments[map[k]] = Some(v);
        }
        let parameters = cf.parameters();
        let mut args: Vec<Value> = Vec::with_capacity(n);
        for i in 0..n {
            let a = match arguments[i].take() {
                Some(a) => a,
                None => {
                    let rpn = cf
                        .defaults
                        .borrow()
                        .as_ref()
                        .and_then(|d| d.get(i).cloned().flatten());
                    let rpn = match rpn {
                        Some(r) => r,
                        None => return Err(exceptions::InvalidNumberOfArguments()),
                    };
                    let existing = cf
                        .default_functions
                        .borrow()
                        .as_ref()
                        .and_then(|d| d.get(i).cloned().flatten());
                    let f = match existing {
                        Some(f) => f,
                        None => {
                            let f = Rc::new(self.compile_rpn(&rpn, false)?);
                            if let Some(df) = cf.default_functions.borrow_mut().as_mut() {
                                df[i] = Some(f.clone());
                            }
                            f
                        }
                    };
                    self.eval_expr(&f)?
                }
            };
            parameters[i].set_value(a.clone());
            args.push(a);
        }
        self.evaluate_custom_function(cf, args)
    }

    /// `Evaluator.EvaluateFunction(cf, x...)`
    pub(crate) fn evaluate_custom_function(
        &self,
        cf: &CustomFunctionRef,
        args: Vec<Value>,
    ) -> CResult<Value> {
        let f = self.ensure_compiled(cf)?;
        let parameters = cf.parameters();
        for (p, a) in parameters.iter().zip(args) {
            p.set_value(a);
        }
        let mut result = self.eval_expr(&f)?;
        let cu = cf.units.borrow().clone();
        let u = apply_units(&mut result, &cu)?;
        self.set_result_units(u);
        Ok(result)
    }

    /// `Evaluator.NormalizeUnits(ref IValue)`
    pub(crate) fn normalize_units(&self, ival: &mut Value) {
        match ival {
            Value::Real(real) => {
                if let Some(u) = &real.units {
                    let d = u.normalize();
                    if d != 1.0 {
                        *real = RealValue::new(real.d * d, real.units.clone());
                    }
                }
            }
            Value::Complex(c) => {
                if let Some(u) = &c.units {
                    let d = u.normalize();
                    if d != 1.0 {
                        *c = ComplexValue::from_c(c.complex() * d, c.units.clone());
                    }
                }
            }
        }
    }
}

/// `MathParser.GetFieldUnit`
pub fn get_field_unit(vu: &UnitRef) -> UnitRef {
    match vu.get_field() {
        Field::Mechanical => Unit::get_force_unit(vu),
        Field::Electrical => Unit::get_electrical_unit(vu),
        Field::Other => vu.clone(),
    }
}

/// `Evaluator.ApplyUnits(ref IValue, Unit)`
pub fn apply_units(ival: &mut Value, u: &OptUnit) -> CResult<OptUnit> {
    match ival {
        Value::Real(real) => apply_units_real(real, u),
        Value::Complex(c) => apply_units_complex(c, u),
    }
}

fn apply_units_complex(value: &mut ComplexValue, u: &OptUnit) -> CResult<OptUnit> {
    let vu = value.units.clone();
    let u = match u {
        None => {
            let vu = match vu {
                None => return Ok(None),
                Some(vu) => vu,
            };
            let u = get_field_unit(&vu);
            if Rc::ptr_eq(&u, &vu) {
                return Ok(Some(vu));
            }
            let c = vu.convert_to(&u);
            *value = ComplexValue::from_c(value.complex() * c, Some(u.clone()));
            return Ok(Some(u));
        }
        Some(u) => u.clone(),
    };
    if u.is_dimensionless() {
        match &vu {
            None => {
                *value = ComplexValue::from_c(
                    value.complex().div_f(u.get_dimensionless_factor()),
                    Some(u.clone()),
                );
                return Ok(Some(u));
            }
            Some(vuu) => {
                if let Some(format) = u.format_string_with_prefix() {
                    let fu = get_field_unit(vuu);
                    let c = if Rc::ptr_eq(&fu, vuu) {
                        1.0
                    } else {
                        vuu.convert_to(&fu)
                    };
                    let nvu = Rc::new(Unit::copy(&fu));
                    nvu.set_text(&format!("{}{}", fu.text(), format));
                    *value = ComplexValue::from_c(value.complex() * c, Some(nvu.clone()));
                    return Ok(Some(nvu));
                }
            }
        }
    }
    if !Unit::is_consistent_opt(&vu, &Some(u.clone())) {
        return Err(exceptions::InconsistentTargetUnits(
            &Unit::get_text_of(&vu),
            &u.text(),
        ));
    }
    let vu = vu.unwrap();
    let d = vu.convert_to(&u);
    if u.is_temp() {
        let c = value.complex() * d
            + crate::base_types::complex::Complex::real(Unit::get_temp_units_delta(
                &vu.text(),
                &u.text(),
            ));
        *value = ComplexValue::from_c(c, Some(u.clone()));
    } else {
        *value = ComplexValue::from_c(value.complex() * d, Some(u.clone()));
    }
    Ok(value.units.clone())
}

fn apply_units_real(value: &mut RealValue, u: &OptUnit) -> CResult<OptUnit> {
    let vu = value.units.clone();
    let u = match u {
        None => {
            let vu = match vu {
                None => return Ok(None),
                Some(vu) => vu,
            };
            let u = get_field_unit(&vu);
            if Rc::ptr_eq(&u, &vu) {
                return Ok(Some(vu));
            }
            let c = vu.convert_to(&u);
            *value = RealValue::new(value.d * c, Some(u.clone()));
            return Ok(Some(u));
        }
        Some(u) => u.clone(),
    };
    if u.is_dimensionless() {
        match &vu {
            None => {
                *value = RealValue::new(value.d / u.get_dimensionless_factor(), Some(u.clone()));
                return Ok(Some(u));
            }
            Some(vuu) => {
                if let Some(format) = u.format_string_with_prefix() {
                    let fu = get_field_unit(vuu);
                    let c = if Rc::ptr_eq(&fu, vuu) {
                        1.0
                    } else {
                        vuu.convert_to(&fu)
                    };
                    let nvu = Rc::new(Unit::copy(&fu));
                    nvu.set_text(&format!("{}{}", fu.text(), format));
                    *value = RealValue::new(value.d * c, Some(nvu.clone()));
                    return Ok(Some(nvu));
                }
            }
        }
    }
    if !Unit::is_consistent_opt(&vu, &Some(u.clone())) {
        return Err(exceptions::InconsistentTargetUnits(
            &Unit::get_text_of(&vu),
            &u.text(),
        ));
    }
    let vu = vu.unwrap();
    let d = vu.convert_to(&u);
    if u.is_temp() {
        let re = value.d * d + Unit::get_temp_units_delta(&vu.text(), &u.text());
        *value = RealValue::new(re, Some(u.clone()));
    } else {
        *value = RealValue::new(value.d * d, Some(u.clone()));
    }
    Ok(value.units.clone())
}
