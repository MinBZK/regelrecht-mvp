//! Core types for the RegelRecht engine

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents any value in the engine (similar to Python's Any)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// Null/None value
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Floating point value
    Float(f64),
    /// String value
    String(String),
    /// Array of values
    Array(Vec<Value>),
    /// Object/Map of values
    Object(HashMap<String, Value>),
}

impl Value {
    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Try to get value as boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get value as i64
    ///
    /// For floats, truncates toward zero (like Python's `int()`).
    /// For example: `1.9` becomes `1`, `-1.9` becomes `-1`.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    /// Try to get value as f64
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Try to get value as string reference
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get value as array reference
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Try to get value as object reference
    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Convert value to boolean (Python-style truthiness)
    ///
    /// Note: NaN is treated as falsy (unlike Python where `bool(float('nan'))` is True).
    /// This is intentional for law execution where NaN represents invalid/missing data.
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0 && !f.is_nan(),
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Int(i)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Int(i as i64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        Value::Array(v.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => Value::Null,
        }
    }
}

/// Operation types supported by the engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Operation {
    // Comparison operations (6)
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,

    // Arithmetic operations (4)
    Add,
    Subtract,
    Multiply,
    Divide,

    // Aggregate operations (2)
    Max,
    Min,

    // Logical operations (2)
    And,
    Or,

    // Conditional operations (2)
    If,
    Switch,
}

impl Operation {
    /// Check if this is a comparison operation
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            Operation::Equals
                | Operation::NotEquals
                | Operation::GreaterThan
                | Operation::LessThan
                | Operation::GreaterThanOrEqual
                | Operation::LessThanOrEqual
        )
    }

    /// Check if this is an arithmetic operation
    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            Operation::Add | Operation::Subtract | Operation::Multiply | Operation::Divide
        )
    }

    /// Check if this is an aggregate operation
    pub fn is_aggregate(&self) -> bool {
        matches!(self, Operation::Max | Operation::Min)
    }

    /// Check if this is a logical operation
    pub fn is_logical(&self) -> bool {
        matches!(self, Operation::And | Operation::Or)
    }

    /// Check if this is a conditional operation
    pub fn is_conditional(&self) -> bool {
        matches!(self, Operation::If | Operation::Switch)
    }
}

/// Regulatory layer types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RegulatoryLayer {
    /// Formal law (wet)
    Wet,
    /// Ministerial regulation (ministeriële regeling)
    MinisterieleRegeling,
    /// General administrative order (AMvB)
    Amvb,
    /// Municipal ordinance (gemeentelijke verordening)
    GemeentelijkeVerordening,
    /// Policy rule (beleidsregel)
    Beleidsregel,
}

impl Default for RegulatoryLayer {
    fn default() -> Self {
        RegulatoryLayer::Wet
    }
}

/// Parameter type specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Date,
    Array,
    Object,
}

impl Default for ParameterType {
    fn default() -> Self {
        ParameterType::String
    }
}

/// Node type in execution trace
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathNodeType {
    Resolve,
    Operation,
    Action,
    Requirement,
    UriCall,
}

/// Resolve type for variable resolution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResolveType {
    Uri,
    Parameter,
    Definition,
    Output,
    Input,
    Local,
    Context,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_bool_conversion() {
        assert!(Value::Bool(true).to_bool());
        assert!(!Value::Bool(false).to_bool());
        assert!(!Value::Null.to_bool());
        assert!(Value::Int(1).to_bool());
        assert!(!Value::Int(0).to_bool());
        assert!(Value::String("hello".to_string()).to_bool());
        assert!(!Value::String("".to_string()).to_bool());
        // NaN is falsy (intentional deviation from Python)
        assert!(!Value::Float(f64::NAN).to_bool());
        assert!(Value::Float(1.0).to_bool());
        assert!(!Value::Float(0.0).to_bool());
    }

    #[test]
    fn test_value_from_primitives() {
        assert_eq!(Value::from(true), Value::Bool(true));
        assert_eq!(Value::from(42i64), Value::Int(42));
        assert_eq!(Value::from(3.14f64), Value::Float(3.14));
        assert_eq!(Value::from("test"), Value::String("test".to_string()));
    }

    #[test]
    fn test_value_as_methods() {
        let bool_val = Value::Bool(true);
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(bool_val.as_int(), None);

        let int_val = Value::Int(42);
        assert_eq!(int_val.as_int(), Some(42));
        assert_eq!(int_val.as_float(), Some(42.0));

        let str_val = Value::String("hello".to_string());
        assert_eq!(str_val.as_str(), Some("hello"));
    }

    #[test]
    fn test_operation_categories() {
        assert!(Operation::Equals.is_comparison());
        assert!(Operation::Add.is_arithmetic());
        assert!(Operation::Max.is_aggregate());
        assert!(Operation::And.is_logical());
        assert!(Operation::If.is_conditional());
    }

    #[test]
    fn test_value_serde_roundtrip() {
        let values = vec![
            Value::Null,
            Value::Bool(true),
            Value::Int(42),
            Value::Float(3.14),
            Value::String("test".to_string()),
            Value::Array(vec![Value::Int(1), Value::Int(2)]),
        ];

        for value in values {
            let json = serde_json::to_string(&value).unwrap();
            let parsed: Value = serde_json::from_str(&json).unwrap();
            assert_eq!(value, parsed);
        }
    }
}
