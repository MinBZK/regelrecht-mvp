//! Execution context for article evaluation
//!
//! Manages state and value resolution during article execution.
//!
//! # Resolution Priority
//!
//! Variables are resolved in the following order (first match wins):
//! 1. **Context variables** - Built-in variables like `referencedate`
//! 2. **Local scope** - Loop variables from FOREACH operations
//! 3. **Outputs** - Previously calculated output values
//! 4. **Resolved inputs** - Cached results from cross-law references
//! 5. **Definitions** - Article-level constants
//! 6. **Parameters** - Direct input parameters (e.g., BSN)
//!
//! # Dot Notation
//!
//! Supports nested property access using dot notation:
//! - `referencedate.year` - Get year from reference date
//! - `person.address.city` - Navigate nested objects

use crate::article::Definition;
use crate::error::{EngineError, Result};
use crate::operations::ValueResolver;
use crate::types::Value;
use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

/// Maximum recursion depth for dot notation property access.
/// Prevents stack overflow on malicious or malformed input like "a.a.a.a.a...".
const MAX_PROPERTY_DEPTH: usize = 32;

/// Execution context for article evaluation.
///
/// Holds all state needed during article execution including parameters,
/// definitions, outputs, and cached values.
///
/// # Shadowing Warning
///
/// Variables in higher-priority scopes shadow those in lower scopes.
/// For example, a local variable named "x" will shadow a parameter "x".
/// The priority order is: local > outputs > resolved_inputs > definitions > parameters.
#[derive(Debug, Clone)]
pub struct RuleContext {
    /// Article-level definitions (constants)
    definitions: HashMap<String, Value>,

    /// Input parameters (e.g., BSN, income)
    parameters: HashMap<String, Value>,

    /// Calculated output values
    outputs: HashMap<String, Value>,

    /// Local scope variables (for FOREACH loops)
    local: HashMap<String, Value>,

    /// Cached resolved inputs from cross-law references
    resolved_inputs: HashMap<String, Value>,

    /// Reference date for calculations
    reference_date: NaiveDate,

    /// Cached Value representation of reference_date (avoids repeated allocation)
    reference_date_value: Value,
}

impl RuleContext {
    /// Create a new execution context.
    ///
    /// # Arguments
    /// * `parameters` - Input parameters for the execution
    /// * `calculation_date` - Reference date for calculations (YYYY-MM-DD format)
    pub fn new(parameters: HashMap<String, Value>, calculation_date: &str) -> Result<Self> {
        let reference_date = NaiveDate::parse_from_str(calculation_date, "%Y-%m-%d")
            .map_err(|e| EngineError::InvalidDate(format!("{}: {}", calculation_date, e)))?;

        let reference_date_value = date_to_value(reference_date);

        Ok(Self {
            definitions: HashMap::new(),
            parameters,
            outputs: HashMap::new(),
            local: HashMap::new(),
            resolved_inputs: HashMap::new(),
            reference_date,
            reference_date_value,
        })
    }

    /// Create a context with a default date (today).
    ///
    /// Useful for testing.
    pub fn with_defaults(parameters: HashMap<String, Value>) -> Self {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        Self::new(parameters, &today).expect("today's date should always be valid")
    }

    /// Set definitions from an article's definitions section.
    ///
    /// Processes the Definition enum to extract actual values.
    pub fn set_definitions(&mut self, definitions: &HashMap<String, Definition>) {
        self.definitions = definitions
            .iter()
            .map(|(k, v)| (k.clone(), v.value().clone()))
            .collect();
    }

    /// Set definitions directly from a Value HashMap.
    pub fn set_definitions_raw(&mut self, definitions: HashMap<String, Value>) {
        self.definitions = definitions;
    }

    /// Set an output value.
    pub fn set_output(&mut self, name: impl Into<String>, value: Value) {
        self.outputs.insert(name.into(), value);
    }

    /// Get an output value.
    pub fn get_output(&self, name: &str) -> Option<&Value> {
        self.outputs.get(name)
    }

    /// Get all outputs.
    pub fn outputs(&self) -> &HashMap<String, Value> {
        &self.outputs
    }

    /// Set a local variable (for FOREACH loops).
    pub fn set_local(&mut self, name: impl Into<String>, value: Value) {
        self.local.insert(name.into(), value);
    }

    /// Clear all local variables.
    pub fn clear_local(&mut self) {
        self.local.clear();
    }

    /// Set a resolved input value (cached cross-law result).
    pub fn set_resolved_input(&mut self, name: impl Into<String>, value: Value) {
        self.resolved_inputs.insert(name.into(), value);
    }

    /// Get all resolved inputs (cached cross-law results).
    pub fn resolved_inputs(&self) -> &HashMap<String, Value> {
        &self.resolved_inputs
    }

    /// Get the reference date.
    pub fn reference_date(&self) -> NaiveDate {
        self.reference_date
    }

    /// Create a child context for nested evaluation (e.g., FOREACH).
    ///
    /// The child inherits definitions, parameters, resolved_inputs, and outputs,
    /// but starts with an **empty local scope**. This ensures that FOREACH loop
    /// variables from a parent context don't leak into child iterations.
    pub fn create_child(&self) -> Self {
        Self {
            definitions: self.definitions.clone(),
            parameters: self.parameters.clone(),
            outputs: self.outputs.clone(),
            local: HashMap::new(), // Child starts with empty local scope
            resolved_inputs: self.resolved_inputs.clone(),
            reference_date: self.reference_date,
            reference_date_value: self.reference_date_value.clone(),
        }
    }

    /// Resolve a variable name using the priority chain.
    ///
    /// # Resolution Priority
    /// 1. Context variables (referencedate)
    /// 2. Local scope (loop variables)
    /// 3. Outputs (calculated values)
    /// 4. Resolved inputs (cached cross-law results)
    /// 5. Definitions (constants)
    /// 6. Parameters (direct inputs)
    ///
    /// # Dot Notation
    /// Supports nested property access: `referencedate.year`, `person.name`
    fn resolve_variable(&self, path: &str) -> Result<Value> {
        // Handle dot notation for property access
        if let Some((base, property)) = path.split_once('.') {
            let base_value = self.resolve_variable(base)?;
            return get_property(&base_value, property, 0);
        }

        // 1. Context variables (cached)
        if path == "referencedate" {
            return Ok(self.reference_date_value.clone());
        }

        // 2. Local scope (FOREACH loop variables)
        if let Some(value) = self.local.get(path) {
            return Ok(value.clone());
        }

        // 3. Outputs (calculated values)
        if let Some(value) = self.outputs.get(path) {
            return Ok(value.clone());
        }

        // 4. Resolved inputs (cached cross-law results)
        if let Some(value) = self.resolved_inputs.get(path) {
            return Ok(value.clone());
        }

        // 5. Definitions (constants)
        if let Some(value) = self.definitions.get(path) {
            return Ok(value.clone());
        }

        // 6. Parameters (direct inputs)
        if let Some(value) = self.parameters.get(path) {
            return Ok(value.clone());
        }

        // Not found
        Err(EngineError::VariableNotFound(path.to_string()))
    }
}

impl ValueResolver for RuleContext {
    fn resolve(&self, name: &str) -> Result<Value> {
        self.resolve_variable(name)
    }
}

/// Convert a NaiveDate to a Value object with year, month, day properties.
fn date_to_value(date: NaiveDate) -> Value {
    let mut obj = HashMap::new();
    obj.insert("year".to_string(), Value::Int(date.year() as i64));
    obj.insert("month".to_string(), Value::Int(date.month() as i64));
    obj.insert("day".to_string(), Value::Int(date.day() as i64));
    // Also include ISO format string for direct use
    obj.insert(
        "iso".to_string(),
        Value::String(date.format("%Y-%m-%d").to_string()),
    );
    Value::Object(obj)
}

/// Get a property from a Value, supporting nested access.
///
/// Handles:
/// - Object property access: `obj.property`
/// - Nested paths: `obj.nested.property`
/// - Array indexing: `arr.0`, `arr.1`
///
/// # Arguments
/// * `value` - The value to access property from
/// * `property_path` - Property path (may contain dots for nesting)
/// * `depth` - Current recursion depth (for stack overflow protection)
fn get_property(value: &Value, property_path: &str, depth: usize) -> Result<Value> {
    // Prevent stack overflow on deeply nested or malicious input
    if depth >= MAX_PROPERTY_DEPTH {
        return Err(EngineError::InvalidOperation(format!(
            "Property access depth exceeds maximum of {}",
            MAX_PROPERTY_DEPTH
        )));
    }

    // Handle nested paths recursively
    if let Some((first, rest)) = property_path.split_once('.') {
        let intermediate = get_property(value, first, depth + 1)?;
        return get_property(&intermediate, rest, depth + 1);
    }

    match value {
        Value::Object(obj) => obj
            .get(property_path)
            .cloned()
            .ok_or_else(|| EngineError::VariableNotFound(format!(".{}", property_path))),
        Value::Array(arr) => {
            // Support numeric indexing for arrays
            if let Ok(index) = property_path.parse::<usize>() {
                arr.get(index)
                    .cloned()
                    .ok_or_else(|| EngineError::VariableNotFound(format!("[{}]", index)))
            } else {
                Err(EngineError::TypeMismatch {
                    expected: "object".to_string(),
                    actual: "array".to_string(),
                })
            }
        }
        _ => Err(EngineError::TypeMismatch {
            expected: "object".to_string(),
            actual: value_type_name(value).to_string(),
        }),
    }
}

/// Get the type name of a Value for error messages.
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Int(_) => "integer",
        Value::Float(_) => "float",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context() -> RuleContext {
        let mut params = HashMap::new();
        params.insert("BSN".to_string(), Value::String("123456789".to_string()));
        params.insert("income".to_string(), Value::Int(30000));

        RuleContext::new(params, "2025-06-15").unwrap()
    }

    // -------------------------------------------------------------------------
    // Basic Resolution Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_resolve_parameter() {
        let ctx = make_context();

        let bsn = ctx.resolve("BSN").unwrap();
        assert_eq!(bsn, Value::String("123456789".to_string()));

        let income = ctx.resolve("income").unwrap();
        assert_eq!(income, Value::Int(30000));
    }

    #[test]
    fn test_resolve_definition() {
        let mut ctx = make_context();
        let mut defs = HashMap::new();
        defs.insert("MAX_INCOME".to_string(), Value::Int(50000));
        defs.insert("TAX_RATE".to_string(), Value::Float(0.21));
        ctx.set_definitions_raw(defs);

        let max = ctx.resolve("MAX_INCOME").unwrap();
        assert_eq!(max, Value::Int(50000));

        let rate = ctx.resolve("TAX_RATE").unwrap();
        assert_eq!(rate, Value::Float(0.21));
    }

    #[test]
    fn test_resolve_output() {
        let mut ctx = make_context();
        ctx.set_output("result", Value::Bool(true));
        ctx.set_output("amount", Value::Int(1000));

        let result = ctx.resolve("result").unwrap();
        assert_eq!(result, Value::Bool(true));

        let amount = ctx.resolve("amount").unwrap();
        assert_eq!(amount, Value::Int(1000));
    }

    #[test]
    fn test_resolve_local() {
        let mut ctx = make_context();
        ctx.set_local("item", Value::String("test".to_string()));
        ctx.set_local("index", Value::Int(0));

        let item = ctx.resolve("item").unwrap();
        assert_eq!(item, Value::String("test".to_string()));

        let index = ctx.resolve("index").unwrap();
        assert_eq!(index, Value::Int(0));
    }

    #[test]
    fn test_resolve_not_found() {
        let ctx = make_context();
        let result = ctx.resolve("nonexistent");
        assert!(matches!(result, Err(EngineError::VariableNotFound(_))));
    }

    // -------------------------------------------------------------------------
    // Priority Chain Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_priority_local_over_output() {
        let mut ctx = make_context();
        ctx.set_output("x", Value::Int(100));
        ctx.set_local("x", Value::Int(200));

        // Local should win
        let x = ctx.resolve("x").unwrap();
        assert_eq!(x, Value::Int(200));

        // After clearing local, output should be visible
        ctx.clear_local();
        let x = ctx.resolve("x").unwrap();
        assert_eq!(x, Value::Int(100));
    }

    #[test]
    fn test_priority_output_over_definition() {
        let mut ctx = make_context();
        let mut defs = HashMap::new();
        defs.insert("x".to_string(), Value::Int(100));
        ctx.set_definitions_raw(defs);
        ctx.set_output("x", Value::Int(200));

        // Output should win over definition
        let x = ctx.resolve("x").unwrap();
        assert_eq!(x, Value::Int(200));
    }

    #[test]
    fn test_priority_definition_over_parameter() {
        let mut ctx = make_context();
        // "income" exists as parameter (30000)
        let mut defs = HashMap::new();
        defs.insert("income".to_string(), Value::Int(50000));
        ctx.set_definitions_raw(defs);

        // Definition should win over parameter
        let income = ctx.resolve("income").unwrap();
        assert_eq!(income, Value::Int(50000));
    }

    // -------------------------------------------------------------------------
    // Reference Date Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_resolve_referencedate() {
        let ctx = make_context();
        let refdate = ctx.resolve("referencedate").unwrap();

        // Should be an object with year, month, day
        let obj = refdate.as_object().unwrap();
        assert_eq!(obj.get("year"), Some(&Value::Int(2025)));
        assert_eq!(obj.get("month"), Some(&Value::Int(6)));
        assert_eq!(obj.get("day"), Some(&Value::Int(15)));
        assert_eq!(
            obj.get("iso"),
            Some(&Value::String("2025-06-15".to_string()))
        );
    }

    #[test]
    fn test_resolve_referencedate_year() {
        let ctx = make_context();
        let year = ctx.resolve("referencedate.year").unwrap();
        assert_eq!(year, Value::Int(2025));
    }

    #[test]
    fn test_resolve_referencedate_month() {
        let ctx = make_context();
        let month = ctx.resolve("referencedate.month").unwrap();
        assert_eq!(month, Value::Int(6));
    }

    // -------------------------------------------------------------------------
    // Dot Notation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_dot_notation_object() {
        let mut ctx = make_context();

        let mut person = HashMap::new();
        person.insert("name".to_string(), Value::String("Jan".to_string()));
        person.insert("age".to_string(), Value::Int(35));
        ctx.set_output("person", Value::Object(person));

        let name = ctx.resolve("person.name").unwrap();
        assert_eq!(name, Value::String("Jan".to_string()));

        let age = ctx.resolve("person.age").unwrap();
        assert_eq!(age, Value::Int(35));
    }

    #[test]
    fn test_dot_notation_nested() {
        let mut ctx = make_context();

        let mut address = HashMap::new();
        address.insert("city".to_string(), Value::String("Amsterdam".to_string()));
        address.insert("zip".to_string(), Value::String("1012AB".to_string()));

        let mut person = HashMap::new();
        person.insert("name".to_string(), Value::String("Jan".to_string()));
        person.insert("address".to_string(), Value::Object(address));
        ctx.set_output("person", Value::Object(person));

        let city = ctx.resolve("person.address.city").unwrap();
        assert_eq!(city, Value::String("Amsterdam".to_string()));
    }

    #[test]
    fn test_dot_notation_array_index() {
        let mut ctx = make_context();

        let items = vec![
            Value::String("first".to_string()),
            Value::String("second".to_string()),
            Value::String("third".to_string()),
        ];
        ctx.set_output("items", Value::Array(items));

        let first = ctx.resolve("items.0").unwrap();
        assert_eq!(first, Value::String("first".to_string()));

        let second = ctx.resolve("items.1").unwrap();
        assert_eq!(second, Value::String("second".to_string()));
    }

    #[test]
    fn test_dot_notation_not_found() {
        let mut ctx = make_context();

        let mut person = HashMap::new();
        person.insert("name".to_string(), Value::String("Jan".to_string()));
        ctx.set_output("person", Value::Object(person));

        let result = ctx.resolve("person.nonexistent");
        assert!(matches!(result, Err(EngineError::VariableNotFound(_))));
    }

    #[test]
    fn test_dot_notation_type_error() {
        let mut ctx = make_context();
        ctx.set_output("value", Value::Int(42));

        // Can't access property on integer
        let result = ctx.resolve("value.something");
        assert!(matches!(result, Err(EngineError::TypeMismatch { .. })));
    }

    // -------------------------------------------------------------------------
    // Child Context Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_child_context_inherits() {
        let mut ctx = make_context();
        ctx.set_output("parent_output", Value::Int(100));
        let mut defs = HashMap::new();
        defs.insert("CONSTANT".to_string(), Value::Int(42));
        ctx.set_definitions_raw(defs);

        let child = ctx.create_child();

        // Child should see parent's output and definition
        assert_eq!(child.resolve("parent_output").unwrap(), Value::Int(100));
        assert_eq!(child.resolve("CONSTANT").unwrap(), Value::Int(42));
        // And parameters
        assert_eq!(
            child.resolve("BSN").unwrap(),
            Value::String("123456789".to_string())
        );
    }

    #[test]
    fn test_child_context_isolation() {
        let mut ctx = make_context();
        ctx.set_output("shared", Value::Int(100));

        let mut child = ctx.create_child();
        child.set_output("shared", Value::Int(200));
        child.set_local("child_only", Value::Int(999));

        // Child's changes shouldn't affect parent
        assert_eq!(ctx.resolve("shared").unwrap(), Value::Int(100));
        assert!(ctx.resolve("child_only").is_err());

        // Child should see its own value
        assert_eq!(child.resolve("shared").unwrap(), Value::Int(200));
    }

    #[test]
    fn test_child_context_empty_local_scope() {
        let mut ctx = make_context();
        ctx.set_local("parent_loop_var", Value::Int(999));
        ctx.set_local("index", Value::Int(5));

        // Create child - should NOT inherit parent's local variables
        let child = ctx.create_child();

        // Child should NOT see parent's loop variables
        assert!(child.resolve("parent_loop_var").is_err());
        assert!(child.resolve("index").is_err());

        // But parent should still have them
        assert_eq!(ctx.resolve("parent_loop_var").unwrap(), Value::Int(999));
        assert_eq!(ctx.resolve("index").unwrap(), Value::Int(5));
    }

    // -------------------------------------------------------------------------
    // Edge Cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_invalid_date() {
        let params = HashMap::new();
        let result = RuleContext::new(params, "not-a-date");
        assert!(matches!(result, Err(EngineError::InvalidDate(_))));
    }

    #[test]
    fn test_empty_context() {
        let ctx = RuleContext::with_defaults(HashMap::new());
        let result = ctx.resolve("anything");
        assert!(matches!(result, Err(EngineError::VariableNotFound(_))));
    }

    #[test]
    fn test_resolved_inputs() {
        let mut ctx = make_context();
        ctx.set_resolved_input("external_value", Value::Int(12345));

        let value = ctx.resolve("external_value").unwrap();
        assert_eq!(value, Value::Int(12345));
    }

    #[test]
    fn test_priority_resolved_input_over_definition() {
        let mut ctx = make_context();
        let mut defs = HashMap::new();
        defs.insert("x".to_string(), Value::Int(100));
        ctx.set_definitions_raw(defs);
        ctx.set_resolved_input("x", Value::Int(200));

        // Resolved input should win over definition
        let x = ctx.resolve("x").unwrap();
        assert_eq!(x, Value::Int(200));
    }

    #[test]
    fn test_recursion_depth_limit() {
        let mut ctx = make_context();

        // Create a deeply nested object (but not exceeding limit)
        // 5 levels of nesting: {n: {n: {n: {n: {n: {value: 42}}}}}}
        let mut deep = HashMap::new();
        deep.insert("value".to_string(), Value::Int(42));

        for _ in 0..5 {
            let mut wrapper = HashMap::new();
            wrapper.insert("n".to_string(), Value::Object(deep));
            deep = wrapper;
        }
        ctx.set_output("deep", Value::Object(deep));

        // This should work: deep.n.n.n.n.n.value (6 property accesses, well within limit of 32)
        let path = "deep.n.n.n.n.n.value";
        let result = ctx.resolve(path);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        assert_eq!(result.unwrap(), Value::Int(42));

        // Create a structure that's deeper than MAX_PROPERTY_DEPTH (32)
        // We need 35 levels to trigger the depth limit
        let mut very_deep = HashMap::new();
        very_deep.insert("end".to_string(), Value::Int(999));

        for _ in 0..35 {
            let mut wrapper = HashMap::new();
            wrapper.insert("n".to_string(), Value::Object(very_deep));
            very_deep = wrapper;
        }
        ctx.set_output("very_deep", Value::Object(very_deep));

        // Build a path with 35 "n"s + "end" = 36 property accesses, exceeding limit of 32
        let excessive_path = format!("very_deep.{}.end", (0..35).map(|_| "n").collect::<Vec<_>>().join("."));

        // This should fail due to depth limit
        let result = ctx.resolve(&excessive_path);
        assert!(
            matches!(result, Err(EngineError::InvalidOperation(ref msg)) if msg.contains("depth exceeds")),
            "Expected InvalidOperation error for excessive depth, got: {:?}",
            result
        );
    }
}
