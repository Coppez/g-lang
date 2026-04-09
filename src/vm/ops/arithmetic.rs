//! Arithmetic and comparison operations using macros.

use crate::errors::RuntimeError;
use crate::runtime::helpers::type_converters::{normalize_int, obj_to_float, to_bigint};
use crate::runtime::obj::Object;
use num_traits::Zero;

macro_rules! gen_numeric_op {
    ($name:ident, $operator:tt, $is_div:expr) => {
        pub fn $name(obj1: Object, obj2: Object) -> Object {
            if let Object::Error(_) = obj1 { return obj1; }
            if let Object::Error(_) = obj2 { return obj2; }

            if matches!(obj1, Object::Float(_)) || matches!(obj2, Object::Float(_)) {
                let f1 = match obj_to_float(obj1) { Ok(f) => f, Err(e) => return e };
                let f2 = match obj_to_float(obj2) { Ok(f) => f, Err(e) => return e };

                if $is_div && f2 == 0.0 {
                    return Object::Error(RuntimeError::DivisionByZero);
                }
                return Object::Float(f1 $operator f2);
            }

            if let (Some(b1), Some(b2)) = (to_bigint(&obj1), to_bigint(&obj2)) {
                if $is_div && b2.is_zero() {
                    return Object::Error(RuntimeError::DivisionByZero);
                }
                return normalize_int(b1 $operator b2);
            }

            type_mismatch_error("number", obj1, obj2)
        }
    };
}

macro_rules! gen_compare_op {
    ($name:ident, $operator:tt) => {
        pub fn $name(obj1: Object, obj2: Object) -> Object {
            if let Object::Error(_) = obj1 { return obj1; }
            if let Object::Error(_) = obj2 { return obj2; }

            if matches!(obj1, Object::Float(_)) || matches!(obj2, Object::Float(_)) {
                let f1 = match obj_to_float(obj1) { Ok(f) => f, Err(e) => return e };
                let f2 = match obj_to_float(obj2) { Ok(f) => f, Err(e) => return e };
                return Object::Boolean(f1 $operator f2);
            }

            if let (Some(b_int1), Some(b_int2)) = (to_bigint(&obj1), to_bigint(&obj2)) {
                return Object::Boolean(b_int1 $operator b_int2);
            }

            type_mismatch_error("number", obj1, obj2)
        }
    };
}

fn type_mismatch_error(expected: &str, obj1: Object, obj2: Object) -> Object {
    Object::Error(RuntimeError::TypeMismatch {
        expected: expected.to_string(),
        got: format!("{} and {}", obj1.type_name(), obj2.type_name()),
    })
}

gen_numeric_op!(add, +, false);
gen_numeric_op!(subtract, -, false);
gen_numeric_op!(multiply, *, false);
gen_numeric_op!(divide, /, true);
gen_numeric_op!(modulo, %, true);

gen_compare_op!(less_than, <);
gen_compare_op!(greater_than, >);
gen_compare_op!(less_equal, <=);
gen_compare_op!(greater_equal, >=);

pub fn execute_equal(obj1: Object, obj2: Object) -> Object {
    Object::Boolean(obj1 == obj2)
}

pub fn execute_not_equal(obj1: Object, obj2: Object) -> Object {
    Object::Boolean(obj1 != obj2)
}

pub fn execute_not(obj: Object) -> Object {
    Object::Boolean(!is_truthy(&obj))
}

pub fn execute_negate(obj: Object) -> Object {
    match obj {
        Object::Integer(i) => Object::Integer(i.wrapping_neg()),
        Object::Float(f) => Object::Float(-f),
        Object::BigInteger(b) => Object::BigInteger(-b),
        other => Object::Error(RuntimeError::InvalidOperation(format!(
            "Negate not supported for {}",
            other.type_name()
        ))),
    }
}

pub fn is_truthy(obj: &Object) -> bool {
    match obj {
        Object::Boolean(b) => *b,
        Object::Null => false,
        Object::Integer(i) => *i != 0,
        Object::Float(f) => *f != 0.0,
        Object::String(s) => !s.is_empty(),
        Object::Array(a) => !a.is_empty(),
        Object::Hash(h) => !h.is_empty(),
        _ => true,
    }
}
