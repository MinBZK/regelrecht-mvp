//! Operation execution for the RegelRecht engine
//!
//! This module implements the execution logic for all 16 operation types:
//! - **Comparison (6):** EQUALS, NOT_EQUALS, GREATER_THAN, LESS_THAN, GREATER_THAN_OR_EQUAL, LESS_THAN_OR_EQUAL
//! - **Arithmetic (4):** ADD, SUBTRACT, MULTIPLY, DIVIDE
//! - **Aggregate (2):** MAX, MIN
//! - **Logical (2):** AND, OR
//! - **Conditional (2):** IF, SWITCH

use crate::article::{ActionOperation, ActionValue};
use crate::error::{EngineError, Result};
use crate::types::{Operation, Value};

/// Trait for resolving variable references ($var) during operation execution.
///
/// Implementations should provide variable resolution from context (parameters,
/// inputs, outputs, definitions, etc.)
///
/// # Error Handling
///
/// - Return `Err(EngineError::VariableNotFound)` when a variable doesn't exist
/// - The error will propagate up through the operation execution chain
/// - Callers should handle missing variables appropriately (e.g., default values)
///
/// # Example Implementation
///
/// ```ignore
/// impl ValueResolver for MyContext {
///     fn resolve(&self, name: &str) -> Result<Value> {
///         self.variables.get(name)
///             .cloned()
///             .ok_or_else(|| EngineError::VariableNotFound(name.to_string()))
///     }
/// }
/// ```
pub trait ValueResolver {
    /// Resolve a variable name to its value.
    ///
    /// # Arguments
    /// * `name` - Variable name without the `$` prefix (e.g., "age" not "$age")
    ///
    /// # Returns
    /// * `Ok(Value)` - The resolved value
    /// * `Err(EngineError::VariableNotFound)` - Variable doesn't exist in context
    fn resolve(&self, name: &str) -> Result<Value>;
}

/// Evaluate an ActionValue to a concrete Value.
///
/// Handles:
/// - Literal values (returned directly)
/// - Variable references ($name) - resolved via the resolver
/// - Nested operations - executed recursively
pub fn evaluate_value<R: ValueResolver>(value: &ActionValue, resolver: &R) -> Result<Value> {
    match value {
        ActionValue::Literal(v) => {
            // Check if it's a variable reference (starts with $)
            if let Value::String(s) = v {
                if let Some(var_name) = s.strip_prefix('$') {
                    return resolver.resolve(var_name);
                }
            }
            Ok(v.clone())
        }
        ActionValue::Operation(op) => execute_operation(op, resolver),
    }
}

/// Execute an operation and return the result.
///
/// Dispatches to the appropriate operation handler based on the operation type.
pub fn execute_operation<R: ValueResolver>(op: &ActionOperation, resolver: &R) -> Result<Value> {
    match op.operation {
        // Comparison operations
        Operation::Equals => execute_equals(op, resolver),
        Operation::NotEquals => execute_not_equals(op, resolver),
        Operation::GreaterThan => execute_numeric_comparison(op, resolver, |a, b| a > b),
        Operation::LessThan => execute_numeric_comparison(op, resolver, |a, b| a < b),
        Operation::GreaterThanOrEqual => execute_numeric_comparison(op, resolver, |a, b| a >= b),
        Operation::LessThanOrEqual => execute_numeric_comparison(op, resolver, |a, b| a <= b),

        // Arithmetic operations
        Operation::Add => execute_add(op, resolver),
        Operation::Subtract => execute_subtract(op, resolver),
        Operation::Multiply => execute_multiply(op, resolver),
        Operation::Divide => execute_divide(op, resolver),

        // Aggregate operations
        Operation::Max => execute_aggregate(op, resolver, f64::max),
        Operation::Min => execute_aggregate(op, resolver, f64::min),

        // Logical operations
        Operation::And => execute_and(op, resolver),
        Operation::Or => execute_or(op, resolver),

        // Conditional operations
        Operation::If => execute_if(op, resolver),
        Operation::Switch => execute_switch(op, resolver),
    }
}

// =============================================================================
// Comparison Operations
// =============================================================================

/// Check if two Values are equal, with Python-style numeric coercion.
///
/// This matches Python's behavior where `42 == 42.0` is `True`.
/// For non-numeric types, uses standard equality.
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        // Numeric comparison: Int and Float are compared as f64
        (Value::Int(i), Value::Float(f)) => (*i as f64) == *f,
        (Value::Float(f), Value::Int(i)) => *f == (*i as f64),
        // Default: use structural equality
        _ => a == b,
    }
}

/// Execute EQUALS operation with Python-style numeric coercion.
///
/// - `Int(42) == Float(42.0)` returns `true` (like Python)
/// - Non-numeric types use structural equality
fn execute_equals<R: ValueResolver>(op: &ActionOperation, resolver: &R) -> Result<Value> {
    let subject = op.subject.as_ref().ok_or_else(|| {
        EngineError::InvalidOperation("Comparison requires 'subject'".to_string())
    })?;
    let value = op
        .value
        .as_ref()
        .ok_or_else(|| EngineError::InvalidOperation("Comparison requires 'value'".to_string()))?;

    let subject_val = evaluate_value(subject, resolver)?;
    let value_val = evaluate_value(value, resolver)?;

    Ok(Value::Bool(values_equal(&subject_val, &value_val)))
}

/// Execute NOT_EQUALS operation with Python-style numeric coercion.
fn execute_not_equals<R: ValueResolver>(op: &ActionOperation, resolver: &R) -> Result<Value> {
    let subject = op.subject.as_ref().ok_or_else(|| {
        EngineError::InvalidOperation("Comparison requires 'subject'".to_string())
    })?;
    let value = op
        .value
        .as_ref()
        .ok_or_else(|| EngineError::InvalidOperation("Comparison requires 'value'".to_string()))?;

    let subject_val = evaluate_value(subject, resolver)?;
    let value_val = evaluate_value(value, resolver)?;

    Ok(Value::Bool(!values_equal(&subject_val, &value_val)))
}

/// Execute a numeric comparison (>, <, >=, <=).
///
/// Converts values to f64 for comparison to handle both Int and Float types.
fn execute_numeric_comparison<R: ValueResolver, F>(
    op: &ActionOperation,
    resolver: &R,
    compare: F,
) -> Result<Value>
where
    F: Fn(f64, f64) -> bool,
{
    let subject = op.subject.as_ref().ok_or_else(|| {
        EngineError::InvalidOperation("Comparison requires 'subject'".to_string())
    })?;
    let value = op
        .value
        .as_ref()
        .ok_or_else(|| EngineError::InvalidOperation("Comparison requires 'value'".to_string()))?;

    let subject_val = evaluate_value(subject, resolver)?;
    let value_val = evaluate_value(value, resolver)?;

    let subject_num = to_number(&subject_val)?;
    let value_num = to_number(&value_val)?;

    Ok(Value::Bool(compare(subject_num, value_num)))
}

// =============================================================================
// Arithmetic Operations
// =============================================================================

/// Execute ADD operation: sum of all values.
fn execute_add<R: ValueResolver>(op: &ActionOperation, resolver: &R) -> Result<Value> {
    let values = get_values(op)?;
    let evaluated = evaluate_values(values, resolver)?;

    let mut sum = 0.0;
    let mut has_float = false;

    for val in &evaluated {
        match val {
            Value::Int(i) => sum += *i as f64,
            Value::Float(f) => {
                sum += f;
                has_float = true;
            }
            _ => return Err(type_error("number", val)),
        }
    }

    Ok(if has_float {
        Value::Float(sum)
    } else {
        Value::Int(sum as i64)
    })
}

/// Execute SUBTRACT operation: first value minus all subsequent values.
fn execute_subtract<R: ValueResolver>(op: &ActionOperation, resolver: &R) -> Result<Value> {
    let values = get_values(op)?;
    if values.is_empty() {
        return Err(EngineError::InvalidOperation(
            "SUBTRACT requires at least one value".to_string(),
        ));
    }

    let evaluated = evaluate_values(values, resolver)?;

    let (first, rest) = evaluated.split_first().unwrap();
    let mut result = to_number(first)?;
    let mut has_float = matches!(first, Value::Float(_));

    for val in rest {
        result -= to_number(val)?;
        if matches!(val, Value::Float(_)) {
            has_float = true;
        }
    }

    Ok(if has_float {
        Value::Float(result)
    } else {
        Value::Int(result as i64)
    })
}

/// Execute MULTIPLY operation: product of all values.
fn execute_multiply<R: ValueResolver>(op: &ActionOperation, resolver: &R) -> Result<Value> {
    let values = get_values(op)?;
    if values.is_empty() {
        return Err(EngineError::InvalidOperation(
            "MULTIPLY requires at least one value".to_string(),
        ));
    }

    let evaluated = evaluate_values(values, resolver)?;

    let mut result = 1.0;
    let mut has_float = false;

    for val in &evaluated {
        match val {
            Value::Int(i) => result *= *i as f64,
            Value::Float(f) => {
                result *= f;
                has_float = true;
            }
            _ => return Err(type_error("number", val)),
        }
    }

    Ok(if has_float {
        Value::Float(result)
    } else {
        Value::Int(result as i64)
    })
}

/// Execute DIVIDE operation: first value divided by all subsequent values.
///
/// Returns `Err(DivisionByZero)` for division by zero.
/// Returns `Err(InvalidOperation)` for NaN or Infinity results.
fn execute_divide<R: ValueResolver>(op: &ActionOperation, resolver: &R) -> Result<Value> {
    let values = get_values(op)?;
    if values.is_empty() {
        return Err(EngineError::InvalidOperation(
            "DIVIDE requires at least one value".to_string(),
        ));
    }

    let evaluated = evaluate_values(values, resolver)?;

    let (first, rest) = evaluated.split_first().unwrap();
    let mut result = to_number(first)?;

    for val in rest {
        let divisor = to_number(val)?;
        if divisor == 0.0 {
            return Err(EngineError::DivisionByZero);
        }
        result /= divisor;
    }

    // Check for invalid results (NaN or Infinity)
    if result.is_nan() {
        return Err(EngineError::InvalidOperation(
            "Division resulted in NaN".to_string(),
        ));
    }
    if result.is_infinite() {
        return Err(EngineError::InvalidOperation(
            "Division resulted in infinity".to_string(),
        ));
    }

    // Division always returns a float (like Python)
    Ok(Value::Float(result))
}

// =============================================================================
// Aggregate Operations
// =============================================================================

/// Execute aggregate operation (MAX, MIN).
fn execute_aggregate<R: ValueResolver, F>(
    op: &ActionOperation,
    resolver: &R,
    combine: F,
) -> Result<Value>
where
    F: Fn(f64, f64) -> f64,
{
    let values = get_values(op)?;
    if values.is_empty() {
        return Err(EngineError::InvalidOperation(
            "Aggregate operation requires at least one value".to_string(),
        ));
    }

    let evaluated = evaluate_values(values, resolver)?;

    let mut has_float = false;
    let nums: Vec<f64> = evaluated
        .iter()
        .map(|v| {
            if matches!(v, Value::Float(_)) {
                has_float = true;
            }
            to_number(v)
        })
        .collect::<Result<Vec<_>>>()?;

    let result = nums.into_iter().reduce(combine).unwrap();

    Ok(if has_float {
        Value::Float(result)
    } else {
        Value::Int(result as i64)
    })
}

// =============================================================================
// Logical Operations
// =============================================================================

/// Execute AND operation: short-circuit evaluation, returns false if any condition is false.
fn execute_and<R: ValueResolver>(op: &ActionOperation, resolver: &R) -> Result<Value> {
    let conditions = op
        .conditions
        .as_ref()
        .ok_or_else(|| EngineError::InvalidOperation("AND requires 'conditions'".to_string()))?;

    for condition in conditions {
        let val = evaluate_value(condition, resolver)?;
        if !val.to_bool() {
            return Ok(Value::Bool(false));
        }
    }

    Ok(Value::Bool(true))
}

/// Execute OR operation: short-circuit evaluation, returns true if any condition is true.
fn execute_or<R: ValueResolver>(op: &ActionOperation, resolver: &R) -> Result<Value> {
    let conditions = op
        .conditions
        .as_ref()
        .ok_or_else(|| EngineError::InvalidOperation("OR requires 'conditions'".to_string()))?;

    for condition in conditions {
        let val = evaluate_value(condition, resolver)?;
        if val.to_bool() {
            return Ok(Value::Bool(true));
        }
    }

    Ok(Value::Bool(false))
}

// =============================================================================
// Conditional Operations
// =============================================================================

/// Execute IF operation: evaluates condition, returns then or else branch.
fn execute_if<R: ValueResolver>(op: &ActionOperation, resolver: &R) -> Result<Value> {
    let when = op
        .when
        .as_ref()
        .ok_or_else(|| EngineError::InvalidOperation("IF requires 'when'".to_string()))?;
    let then = op
        .then
        .as_ref()
        .ok_or_else(|| EngineError::InvalidOperation("IF requires 'then'".to_string()))?;

    let condition_result = evaluate_value(when, resolver)?;

    if condition_result.to_bool() {
        evaluate_value(then, resolver)
    } else if let Some(else_branch) = &op.else_branch {
        evaluate_value(else_branch, resolver)
    } else {
        Ok(Value::Null)
    }
}

/// Execute SWITCH operation: evaluates cases in order, returns first matching case.
fn execute_switch<R: ValueResolver>(op: &ActionOperation, resolver: &R) -> Result<Value> {
    let cases = op
        .cases
        .as_ref()
        .ok_or_else(|| EngineError::InvalidOperation("SWITCH requires 'cases'".to_string()))?;

    for case in cases {
        let condition_result = evaluate_value(&case.when, resolver)?;
        if condition_result.to_bool() {
            return evaluate_value(&case.then, resolver);
        }
    }

    // Return default if no case matched
    if let Some(default) = &op.default {
        evaluate_value(default, resolver)
    } else {
        Ok(Value::Null)
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Get the 'values' array from an operation, or return an error.
fn get_values(op: &ActionOperation) -> Result<&Vec<ActionValue>> {
    op.values.as_ref().ok_or_else(|| {
        EngineError::InvalidOperation(format!("{:?} requires 'values'", op.operation))
    })
}

/// Evaluate a slice of ActionValues to concrete Values.
fn evaluate_values<R: ValueResolver>(values: &[ActionValue], resolver: &R) -> Result<Vec<Value>> {
    values.iter().map(|v| evaluate_value(v, resolver)).collect()
}

/// Convert a Value to a number (f64).
fn to_number(val: &Value) -> Result<f64> {
    match val {
        Value::Int(i) => Ok(*i as f64),
        Value::Float(f) => Ok(*f),
        _ => Err(type_error("number", val)),
    }
}

/// Create a TypeMismatch error.
fn type_error(expected: &str, actual: &Value) -> EngineError {
    let actual_type = match actual {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Int(_) => "integer",
        Value::Float(_) => "float",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    };
    EngineError::TypeMismatch {
        expected: expected.to_string(),
        actual: actual_type.to_string(),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Simple resolver for testing that uses a HashMap
    struct TestResolver {
        vars: HashMap<String, Value>,
    }

    impl TestResolver {
        fn new() -> Self {
            Self {
                vars: HashMap::new(),
            }
        }

        fn with_var(mut self, name: &str, value: impl Into<Value>) -> Self {
            self.vars.insert(name.to_string(), value.into());
            self
        }
    }

    impl ValueResolver for TestResolver {
        fn resolve(&self, name: &str) -> Result<Value> {
            self.vars
                .get(name)
                .cloned()
                .ok_or_else(|| EngineError::VariableNotFound(name.to_string()))
        }
    }

    /// Helper to create a literal ActionValue
    fn lit(v: impl Into<Value>) -> ActionValue {
        ActionValue::Literal(v.into())
    }

    /// Helper to create a variable reference
    fn var(name: &str) -> ActionValue {
        ActionValue::Literal(Value::String(format!("${}", name)))
    }

    // -------------------------------------------------------------------------
    // Comparison Operations Tests
    // -------------------------------------------------------------------------

    mod comparison {
        use super::*;

        #[test]
        fn test_equals_integers() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Equals,
                subject: Some(lit(42i64)),
                value: Some(lit(42i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Bool(true));
        }

        #[test]
        fn test_equals_different_values() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Equals,
                subject: Some(lit(42i64)),
                value: Some(lit(43i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Bool(false));
        }

        #[test]
        fn test_not_equals() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::NotEquals,
                subject: Some(lit(42i64)),
                value: Some(lit(43i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Bool(true));
        }

        #[test]
        fn test_greater_than() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::GreaterThan,
                subject: Some(lit(50i64)),
                value: Some(lit(42i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Bool(true));
        }

        #[test]
        fn test_less_than() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::LessThan,
                subject: Some(lit(30i64)),
                value: Some(lit(42i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Bool(true));
        }

        #[test]
        fn test_greater_than_or_equal() {
            let resolver = TestResolver::new();

            // Test equal case
            let op = ActionOperation {
                operation: Operation::GreaterThanOrEqual,
                subject: Some(lit(42i64)),
                value: Some(lit(42i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };
            assert_eq!(
                execute_operation(&op, &resolver).unwrap(),
                Value::Bool(true)
            );

            // Test greater case
            let op2 = ActionOperation {
                operation: Operation::GreaterThanOrEqual,
                subject: Some(lit(50i64)),
                value: Some(lit(42i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };
            assert_eq!(
                execute_operation(&op2, &resolver).unwrap(),
                Value::Bool(true)
            );
        }

        #[test]
        fn test_less_than_or_equal() {
            let resolver = TestResolver::new();

            // Test equal case
            let op = ActionOperation {
                operation: Operation::LessThanOrEqual,
                subject: Some(lit(42i64)),
                value: Some(lit(42i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };
            assert_eq!(
                execute_operation(&op, &resolver).unwrap(),
                Value::Bool(true)
            );

            // Test less case
            let op2 = ActionOperation {
                operation: Operation::LessThanOrEqual,
                subject: Some(lit(30i64)),
                value: Some(lit(42i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };
            assert_eq!(
                execute_operation(&op2, &resolver).unwrap(),
                Value::Bool(true)
            );
        }

        #[test]
        fn test_comparison_with_variables() {
            let resolver = TestResolver::new()
                .with_var("age", 25i64)
                .with_var("min_age", 18i64);

            let op = ActionOperation {
                operation: Operation::GreaterThanOrEqual,
                subject: Some(var("age")),
                value: Some(var("min_age")),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Bool(true));
        }

        #[test]
        fn test_comparison_mixed_int_float() {
            let resolver = TestResolver::new();

            // Int(42) == Float(42.0) should be true (Python-style numeric coercion)
            let op = ActionOperation {
                operation: Operation::Equals,
                subject: Some(lit(42i64)),
                value: Some(lit(42.0f64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };
            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Bool(true));

            // Float(42.0) == Int(42) should also be true (symmetric)
            let op2 = ActionOperation {
                operation: Operation::Equals,
                subject: Some(lit(42.0f64)),
                value: Some(lit(42i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };
            let result2 = execute_operation(&op2, &resolver).unwrap();
            assert_eq!(result2, Value::Bool(true));

            // Int(42) != Float(42.5) should be true
            let op3 = ActionOperation {
                operation: Operation::NotEquals,
                subject: Some(lit(42i64)),
                value: Some(lit(42.5f64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };
            let result3 = execute_operation(&op3, &resolver).unwrap();
            assert_eq!(result3, Value::Bool(true));
        }
    }

    // -------------------------------------------------------------------------
    // Arithmetic Operations Tests
    // -------------------------------------------------------------------------

    mod arithmetic {
        use super::*;

        #[test]
        fn test_add_integers() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Add,
                subject: None,
                value: None,
                values: Some(vec![lit(10i64), lit(20i64), lit(30i64)]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(60));
        }

        #[test]
        fn test_add_with_floats() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Add,
                subject: None,
                value: None,
                values: Some(vec![lit(10i64), lit(20.5f64), lit(30i64)]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Float(60.5));
        }

        #[test]
        fn test_subtract() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Subtract,
                subject: None,
                value: None,
                values: Some(vec![lit(100i64), lit(30i64), lit(20i64)]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(50));
        }

        #[test]
        fn test_multiply() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Multiply,
                subject: None,
                value: None,
                values: Some(vec![lit(2i64), lit(3i64), lit(4i64)]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(24));
        }

        #[test]
        fn test_divide() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Divide,
                subject: None,
                value: None,
                values: Some(vec![lit(100i64), lit(2i64)]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Float(50.0));
        }

        #[test]
        fn test_divide_by_zero() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Divide,
                subject: None,
                value: None,
                values: Some(vec![lit(100i64), lit(0i64)]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver);
            assert!(matches!(result, Err(EngineError::DivisionByZero)));
        }

        #[test]
        fn test_arithmetic_with_variables() {
            let resolver = TestResolver::new()
                .with_var("base", 1000i64)
                .with_var("rate", 0.05f64);

            let op = ActionOperation {
                operation: Operation::Multiply,
                subject: None,
                value: None,
                values: Some(vec![var("base"), var("rate")]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Float(50.0));
        }
    }

    // -------------------------------------------------------------------------
    // Aggregate Operations Tests
    // -------------------------------------------------------------------------

    mod aggregate {
        use super::*;

        #[test]
        fn test_max() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Max,
                subject: None,
                value: None,
                values: Some(vec![lit(10i64), lit(50i64), lit(30i64)]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(50));
        }

        #[test]
        fn test_min() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Min,
                subject: None,
                value: None,
                values: Some(vec![lit(10i64), lit(50i64), lit(30i64)]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(10));
        }

        #[test]
        fn test_max_with_floats() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Max,
                subject: None,
                value: None,
                values: Some(vec![lit(10.5f64), lit(50.3f64), lit(30.7f64)]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Float(50.3));
        }

        #[test]
        fn test_max_with_zero() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Max,
                subject: None,
                value: None,
                values: Some(vec![lit(0i64), lit(-10i64)]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(0));
        }
    }

    // -------------------------------------------------------------------------
    // Logical Operations Tests
    // -------------------------------------------------------------------------

    mod logical {
        use super::*;

        #[test]
        fn test_and_all_true() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::And,
                subject: None,
                value: None,
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: Some(vec![lit(true), lit(true), lit(true)]),
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Bool(true));
        }

        #[test]
        fn test_and_one_false() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::And,
                subject: None,
                value: None,
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: Some(vec![lit(true), lit(false), lit(true)]),
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Bool(false));
        }

        #[test]
        fn test_or_one_true() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Or,
                subject: None,
                value: None,
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: Some(vec![lit(false), lit(true), lit(false)]),
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Bool(true));
        }

        #[test]
        fn test_or_all_false() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Or,
                subject: None,
                value: None,
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: Some(vec![lit(false), lit(false), lit(false)]),
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Bool(false));
        }

        #[test]
        fn test_and_with_nested_comparison() {
            let resolver = TestResolver::new()
                .with_var("age", 25i64)
                .with_var("has_insurance", true);

            // AND with nested operations: age >= 18 AND has_insurance
            let age_check = ActionValue::Operation(Box::new(ActionOperation {
                operation: Operation::GreaterThanOrEqual,
                subject: Some(var("age")),
                value: Some(lit(18i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            }));

            let op = ActionOperation {
                operation: Operation::And,
                subject: None,
                value: None,
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: Some(vec![age_check, var("has_insurance")]),
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Bool(true));
        }
    }

    // -------------------------------------------------------------------------
    // Conditional Operations Tests
    // -------------------------------------------------------------------------

    mod conditional {
        use super::*;
        use crate::article::SwitchCase;

        #[test]
        fn test_if_true_branch() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::If,
                subject: None,
                value: None,
                values: None,
                when: Some(lit(true)),
                then: Some(lit(100i64)),
                else_branch: Some(lit(50i64)),
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(100));
        }

        #[test]
        fn test_if_false_branch() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::If,
                subject: None,
                value: None,
                values: None,
                when: Some(lit(false)),
                then: Some(lit(100i64)),
                else_branch: Some(lit(50i64)),
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(50));
        }

        #[test]
        fn test_if_no_else_returns_null() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::If,
                subject: None,
                value: None,
                values: None,
                when: Some(lit(false)),
                then: Some(lit(100i64)),
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Null);
        }

        #[test]
        fn test_if_with_nested_condition() {
            let resolver = TestResolver::new().with_var("has_partner", true);

            let condition = ActionValue::Operation(Box::new(ActionOperation {
                operation: Operation::Equals,
                subject: Some(var("has_partner")),
                value: Some(lit(true)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            }));

            let op = ActionOperation {
                operation: Operation::If,
                subject: None,
                value: None,
                values: None,
                when: Some(condition),
                then: Some(lit(200i64)),
                else_branch: Some(lit(100i64)),
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(200));
        }

        #[test]
        fn test_switch_first_match() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Switch,
                subject: None,
                value: None,
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: Some(vec![
                    SwitchCase {
                        when: lit(true),
                        then: lit(100i64),
                    },
                    SwitchCase {
                        when: lit(true),
                        then: lit(200i64),
                    },
                ]),
                default: Some(lit(0i64)),
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(100));
        }

        #[test]
        fn test_switch_second_match() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Switch,
                subject: None,
                value: None,
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: Some(vec![
                    SwitchCase {
                        when: lit(false),
                        then: lit(100i64),
                    },
                    SwitchCase {
                        when: lit(true),
                        then: lit(200i64),
                    },
                ]),
                default: Some(lit(0i64)),
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(200));
        }

        #[test]
        fn test_switch_default() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Switch,
                subject: None,
                value: None,
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: Some(vec![
                    SwitchCase {
                        when: lit(false),
                        then: lit(100i64),
                    },
                    SwitchCase {
                        when: lit(false),
                        then: lit(200i64),
                    },
                ]),
                default: Some(lit(0i64)),
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(0));
        }

        #[test]
        fn test_switch_no_default_returns_null() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Switch,
                subject: None,
                value: None,
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: Some(vec![SwitchCase {
                    when: lit(false),
                    then: lit(100i64),
                }]),
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Null);
        }

        #[test]
        fn test_switch_with_nested_conditions() {
            let resolver = TestResolver::new().with_var("status", "active");

            let op = ActionOperation {
                operation: Operation::Switch,
                subject: None,
                value: None,
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: Some(vec![
                    SwitchCase {
                        when: ActionValue::Operation(Box::new(ActionOperation {
                            operation: Operation::Equals,
                            subject: Some(var("status")),
                            value: Some(lit("pending")),
                            values: None,
                            when: None,
                            then: None,
                            else_branch: None,
                            conditions: None,
                            cases: None,
                            default: None,
                        })),
                        then: lit(10i64),
                    },
                    SwitchCase {
                        when: ActionValue::Operation(Box::new(ActionOperation {
                            operation: Operation::Equals,
                            subject: Some(var("status")),
                            value: Some(lit("active")),
                            values: None,
                            when: None,
                            then: None,
                            else_branch: None,
                            conditions: None,
                            cases: None,
                            default: None,
                        })),
                        then: lit(20i64),
                    },
                ]),
                default: Some(lit(0i64)),
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(20));
        }
    }

    // -------------------------------------------------------------------------
    // Nested Operations Tests
    // -------------------------------------------------------------------------

    mod nested {
        use super::*;

        #[test]
        fn test_nested_arithmetic_in_max() {
            // MAX(0, 100 - 50) = MAX(0, 50) = 50
            let resolver = TestResolver::new();

            let subtract_op = ActionValue::Operation(Box::new(ActionOperation {
                operation: Operation::Subtract,
                subject: None,
                value: None,
                values: Some(vec![lit(100i64), lit(50i64)]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            }));

            let op = ActionOperation {
                operation: Operation::Max,
                subject: None,
                value: None,
                values: Some(vec![lit(0i64), subtract_op]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(50));
        }

        #[test]
        fn test_deeply_nested_operations() {
            // IF (10 > 5) THEN (2 * 3) ELSE (1 + 1)
            let resolver = TestResolver::new();

            let condition = ActionValue::Operation(Box::new(ActionOperation {
                operation: Operation::GreaterThan,
                subject: Some(lit(10i64)),
                value: Some(lit(5i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            }));

            let then_branch = ActionValue::Operation(Box::new(ActionOperation {
                operation: Operation::Multiply,
                subject: None,
                value: None,
                values: Some(vec![lit(2i64), lit(3i64)]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            }));

            let else_branch = ActionValue::Operation(Box::new(ActionOperation {
                operation: Operation::Add,
                subject: None,
                value: None,
                values: Some(vec![lit(1i64), lit(1i64)]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            }));

            let op = ActionOperation {
                operation: Operation::If,
                subject: None,
                value: None,
                values: None,
                when: Some(condition),
                then: Some(then_branch),
                else_branch: Some(else_branch),
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver).unwrap();
            assert_eq!(result, Value::Int(6));
        }
    }

    // -------------------------------------------------------------------------
    // Error Handling Tests
    // -------------------------------------------------------------------------

    mod errors {
        use super::*;

        #[test]
        fn test_missing_subject_in_comparison() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Equals,
                subject: None,
                value: Some(lit(42i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver);
            assert!(matches!(result, Err(EngineError::InvalidOperation(_))));
        }

        #[test]
        fn test_missing_values_in_add() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Add,
                subject: None,
                value: None,
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver);
            assert!(matches!(result, Err(EngineError::InvalidOperation(_))));
        }

        #[test]
        fn test_type_mismatch_in_arithmetic() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Add,
                subject: None,
                value: None,
                values: Some(vec![lit(10i64), lit("not a number")]),
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver);
            assert!(matches!(result, Err(EngineError::TypeMismatch { .. })));
        }

        #[test]
        fn test_variable_not_found() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Equals,
                subject: Some(var("nonexistent")),
                value: Some(lit(42i64)),
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver);
            assert!(matches!(result, Err(EngineError::VariableNotFound(_))));
        }

        #[test]
        fn test_missing_when_in_if() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::If,
                subject: None,
                value: None,
                values: None,
                when: None,
                then: Some(lit(100i64)),
                else_branch: None,
                conditions: None,
                cases: None,
                default: None,
            };

            let result = execute_operation(&op, &resolver);
            assert!(matches!(result, Err(EngineError::InvalidOperation(_))));
        }

        #[test]
        fn test_missing_cases_in_switch() {
            let resolver = TestResolver::new();
            let op = ActionOperation {
                operation: Operation::Switch,
                subject: None,
                value: None,
                values: None,
                when: None,
                then: None,
                else_branch: None,
                conditions: None,
                cases: None,
                default: Some(lit(0i64)),
            };

            let result = execute_operation(&op, &resolver);
            assert!(matches!(result, Err(EngineError::InvalidOperation(_))));
        }
    }
}
