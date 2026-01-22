//! Article execution engine
//!
//! Core engine for evaluating article-level machine_readable.execution sections.
//!
//! # Example
//!
//! ```ignore
//! use regelrecht_engine::{ArticleEngine, ArticleBasedLaw, Value};
//! use std::collections::HashMap;
//!
//! let law = ArticleBasedLaw::from_yaml_file("path/to/law.yaml")?;
//! let article = law.find_article_by_output("some_output").unwrap();
//!
//! let engine = ArticleEngine::new(article, &law);
//! let mut params = HashMap::new();
//! params.insert("BSN".to_string(), Value::String("123456789".to_string()));
//!
//! let result = engine.evaluate(params, "2025-01-01")?;
//! println!("Output: {:?}", result.outputs);
//! ```

use crate::article::{Action, ActionOperation, Article, ArticleBasedLaw};
use crate::context::RuleContext;
use crate::error::{EngineError, Result};
use crate::operations::{evaluate_value, execute_operation};
use crate::types::Value;
use std::collections::HashMap;

/// Result of article execution
#[derive(Debug, Clone)]
pub struct ArticleResult {
    /// Calculated output values
    pub outputs: HashMap<String, Value>,
    /// Resolved input values (from cross-law references)
    pub resolved_inputs: HashMap<String, Value>,
    /// Article number that was executed
    pub article_number: String,
    /// Law ID containing the article
    pub law_id: String,
    /// Law UUID if available
    pub law_uuid: Option<String>,
}

/// Executes a single article's machine_readable.execution section.
///
/// The engine orchestrates the execution of an article's actions,
/// resolving variables and evaluating operations to produce outputs.
pub struct ArticleEngine<'a> {
    /// Article to execute
    article: &'a Article,
    /// Law containing the article
    law: &'a ArticleBasedLaw,
}

impl<'a> ArticleEngine<'a> {
    /// Create a new article engine.
    ///
    /// # Arguments
    /// * `article` - Article to execute
    /// * `law` - Law containing the article
    pub fn new(article: &'a Article, law: &'a ArticleBasedLaw) -> Self {
        Self { article, law }
    }

    /// Execute this article's logic.
    ///
    /// # Arguments
    /// * `parameters` - Input parameters (e.g., {"BSN": "123456789"})
    /// * `calculation_date` - Date for which calculations are performed (YYYY-MM-DD)
    ///
    /// # Returns
    /// * `Ok(ArticleResult)` - Execution result with outputs and metadata
    /// * `Err(EngineError)` - If execution fails
    pub fn evaluate(
        &self,
        parameters: HashMap<String, Value>,
        calculation_date: &str,
    ) -> Result<ArticleResult> {
        self.evaluate_with_output(parameters, calculation_date, None)
    }

    /// Execute this article's logic, optionally calculating only a specific output.
    ///
    /// # Arguments
    /// * `parameters` - Input parameters (e.g., {"BSN": "123456789"})
    /// * `calculation_date` - Date for which calculations are performed (YYYY-MM-DD)
    /// * `requested_output` - Specific output to calculate (optional, calculates all if None)
    ///
    /// # Returns
    /// * `Ok(ArticleResult)` - Execution result with outputs and metadata
    /// * `Err(EngineError)` - If execution fails
    pub fn evaluate_with_output(
        &self,
        parameters: HashMap<String, Value>,
        calculation_date: &str,
        requested_output: Option<&str>,
    ) -> Result<ArticleResult> {
        // Create execution context
        let mut context = RuleContext::new(parameters, calculation_date)?;

        // Set definitions from article
        if let Some(definitions) = self.article.get_definitions() {
            context.set_definitions(definitions);
        }

        // Execute actions
        self.execute_actions(&mut context, requested_output)?;

        // Build result
        // Clone outputs and resolved_inputs since ArticleResult must own the data
        // (it may outlive the context)
        Ok(ArticleResult {
            outputs: context.outputs().clone(),
            resolved_inputs: context.resolved_inputs().clone(),
            article_number: self.article.number.clone(),
            law_id: self.law.id.clone(),
            law_uuid: self.law.uuid.clone(),
        })
    }

    /// Execute all actions in order.
    ///
    /// # Arguments
    /// * `context` - Execution context
    /// * `requested_output` - Specific output to calculate (optional)
    fn execute_actions(
        &self,
        context: &mut RuleContext,
        requested_output: Option<&str>,
    ) -> Result<()> {
        let actions = self.get_actions();

        for action in actions {
            // Skip actions without output
            let output_name = match &action.output {
                Some(name) => name,
                None => continue,
            };

            // If requested_output specified, only execute matching action
            if let Some(requested) = requested_output {
                if output_name != requested {
                    continue;
                }
            }

            // Evaluate the action and store output
            let value = self.evaluate_action(action, context)?;
            context.set_output(output_name, value);
        }

        Ok(())
    }

    /// Evaluate a single action.
    ///
    /// # Arguments
    /// * `action` - Action specification
    /// * `context` - Execution context
    ///
    /// # Returns
    /// Calculated value
    fn evaluate_action(&self, action: &Action, context: &RuleContext) -> Result<Value> {
        // Check for operation at action level FIRST
        // When an action has an operation, the value/subject fields are operands, not direct results
        if let Some(operation) = &action.operation {
            let action_op = self.action_to_operation(action, operation)?;
            return execute_operation(&action_op, context);
        }

        // Check for direct value (only when no operation is specified)
        if let Some(value) = &action.value {
            return evaluate_value(value, context);
        }

        // Check for resolve (cross-law reference) - TODO: Phase 5
        if action.resolve.is_some() {
            return Err(EngineError::InvalidOperation(
                "Cross-law references (resolve) not yet implemented".to_string(),
            ));
        }

        // No value or operation specified
        Ok(Value::Null)
    }

    /// Convert an Action to an ActionOperation for execution.
    ///
    /// This is needed because actions can have operations specified inline
    /// rather than as nested ActionValue::Operation.
    ///
    /// # Limitations
    ///
    /// SWITCH operations at the action level are NOT supported because the
    /// `Action` struct doesn't have `cases` and `default` fields. SWITCH must
    /// be nested inside `value` as an `ActionValue::Operation`. Example:
    ///
    /// ```yaml
    /// # INCORRECT - won't work:
    /// - output: result
    ///   operation: SWITCH
    ///   cases: [...]  # Action doesn't have this field
    ///
    /// # CORRECT - use value wrapper:
    /// - output: result
    ///   value:
    ///     operation: SWITCH
    ///     cases: [...]
    /// ```
    fn action_to_operation(
        &self,
        action: &Action,
        operation: &crate::types::Operation,
    ) -> Result<ActionOperation> {
        Ok(ActionOperation {
            operation: *operation, // Operation is Copy
            subject: action.subject.clone(),
            value: action.value.clone(),
            values: action.values.clone(),
            when: action.when.clone(),
            then: action.then.clone(),
            else_branch: action.else_branch.clone(),
            conditions: action.conditions.clone(),
            // SWITCH-specific fields: Action struct doesn't have these,
            // so SWITCH must be nested inside `value` to work correctly
            cases: None,
            default: None,
        })
    }

    /// Get actions from the article's execution spec.
    fn get_actions(&self) -> &[Action] {
        self.article
            .get_execution_spec()
            .and_then(|exec| exec.actions.as_deref())
            .unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::article::ArticleBasedLaw;

    fn make_simple_law() -> ArticleBasedLaw {
        let yaml = r#"
$id: test_law
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Test article
    machine_readable:
      definitions:
        MAX_AGE:
          value: 67
        MIN_AGE:
          value: 18
      execution:
        parameters:
          - name: age
            type: number
            required: true
        output:
          - name: is_adult
            type: boolean
          - name: age_check_result
            type: string
        actions:
          - output: is_adult
            operation: GREATER_THAN_OR_EQUAL
            subject: $age
            value: $MIN_AGE
          - output: age_check_result
            value:
              operation: IF
              when:
                operation: GREATER_THAN_OR_EQUAL
                subject: $age
                value: $MIN_AGE
              then: "adult"
              else: "minor"
"#;
        ArticleBasedLaw::from_yaml_str(yaml).unwrap()
    }

    fn make_arithmetic_law() -> ArticleBasedLaw {
        let yaml = r#"
$id: calc_law
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Calculation article
    machine_readable:
      definitions:
        TAX_RATE:
          value: 0.21
        BASE_DEDUCTION:
          value: 1000
      execution:
        parameters:
          - name: income
            type: number
            required: true
        output:
          - name: taxable_income
            type: number
          - name: tax_amount
            type: number
        actions:
          - output: taxable_income
            operation: MAX
            values:
              - 0
              - operation: SUBTRACT
                values:
                  - $income
                  - $BASE_DEDUCTION
          - output: tax_amount
            operation: MULTIPLY
            values:
              - $taxable_income
              - $TAX_RATE
"#;
        ArticleBasedLaw::from_yaml_str(yaml).unwrap()
    }

    // -------------------------------------------------------------------------
    // Basic Execution Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_evaluate_simple_comparison() {
        let law = make_simple_law();
        let article = law.find_article_by_number("1").unwrap();
        let engine = ArticleEngine::new(article, &law);

        let mut params = HashMap::new();
        params.insert("age".to_string(), Value::Int(25));

        let result = engine.evaluate(params, "2025-01-01").unwrap();

        assert_eq!(result.article_number, "1");
        assert_eq!(result.law_id, "test_law");
        assert_eq!(result.outputs.get("is_adult"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_evaluate_with_definitions() {
        let law = make_simple_law();
        let article = law.find_article_by_number("1").unwrap();
        let engine = ArticleEngine::new(article, &law);

        // Age 15 is less than MIN_AGE (18)
        let mut params = HashMap::new();
        params.insert("age".to_string(), Value::Int(15));

        let result = engine.evaluate(params, "2025-01-01").unwrap();

        assert_eq!(result.outputs.get("is_adult"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_evaluate_nested_if() {
        let law = make_simple_law();
        let article = law.find_article_by_number("1").unwrap();
        let engine = ArticleEngine::new(article, &law);

        // Adult case
        let mut params = HashMap::new();
        params.insert("age".to_string(), Value::Int(25));
        let result = engine.evaluate(params, "2025-01-01").unwrap();
        assert_eq!(
            result.outputs.get("age_check_result"),
            Some(&Value::String("adult".to_string()))
        );

        // Minor case
        let mut params = HashMap::new();
        params.insert("age".to_string(), Value::Int(15));
        let result = engine.evaluate(params, "2025-01-01").unwrap();
        assert_eq!(
            result.outputs.get("age_check_result"),
            Some(&Value::String("minor".to_string()))
        );
    }

    // -------------------------------------------------------------------------
    // Arithmetic Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_evaluate_arithmetic_operations() {
        let law = make_arithmetic_law();
        let article = law.find_article_by_number("1").unwrap();
        let engine = ArticleEngine::new(article, &law);

        let mut params = HashMap::new();
        params.insert("income".to_string(), Value::Int(5000));

        let result = engine.evaluate(params, "2025-01-01").unwrap();

        // taxable_income = MAX(0, 5000 - 1000) = 4000
        assert_eq!(
            result.outputs.get("taxable_income"),
            Some(&Value::Int(4000))
        );

        // tax_amount = 4000 * 0.21 = 840.0
        assert_eq!(
            result.outputs.get("tax_amount"),
            Some(&Value::Float(840.0))
        );
    }

    #[test]
    fn test_evaluate_arithmetic_with_floor() {
        let law = make_arithmetic_law();
        let article = law.find_article_by_number("1").unwrap();
        let engine = ArticleEngine::new(article, &law);

        // Income below deduction
        let mut params = HashMap::new();
        params.insert("income".to_string(), Value::Int(500));

        let result = engine.evaluate(params, "2025-01-01").unwrap();

        // taxable_income = MAX(0, 500 - 1000) = MAX(0, -500) = 0
        assert_eq!(result.outputs.get("taxable_income"), Some(&Value::Int(0)));

        // tax_amount = 0 * 0.21 = 0.0
        assert_eq!(
            result.outputs.get("tax_amount"),
            Some(&Value::Float(0.0))
        );
    }

    // -------------------------------------------------------------------------
    // Selective Output Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_evaluate_specific_output() {
        let law = make_simple_law();
        let article = law.find_article_by_number("1").unwrap();
        let engine = ArticleEngine::new(article, &law);

        let mut params = HashMap::new();
        params.insert("age".to_string(), Value::Int(25));

        // Only request is_adult output
        let result = engine
            .evaluate_with_output(params, "2025-01-01", Some("is_adult"))
            .unwrap();

        // Should have is_adult
        assert!(result.outputs.contains_key("is_adult"));
        // Should NOT have age_check_result (wasn't requested)
        assert!(!result.outputs.contains_key("age_check_result"));
    }

    // -------------------------------------------------------------------------
    // Error Handling Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_missing_parameter() {
        let law = make_simple_law();
        let article = law.find_article_by_number("1").unwrap();
        let engine = ArticleEngine::new(article, &law);

        // No parameters - age is missing
        let params = HashMap::new();
        let result = engine.evaluate(params, "2025-01-01");

        assert!(matches!(result, Err(EngineError::VariableNotFound(_))));
    }

    #[test]
    fn test_invalid_date() {
        let law = make_simple_law();
        let article = law.find_article_by_number("1").unwrap();
        let engine = ArticleEngine::new(article, &law);

        let mut params = HashMap::new();
        params.insert("age".to_string(), Value::Int(25));

        let result = engine.evaluate(params, "not-a-date");
        assert!(matches!(result, Err(EngineError::InvalidDate(_))));
    }

    // -------------------------------------------------------------------------
    // Reference Date Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_reference_date_accessible() {
        let yaml = r#"
$id: date_law
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Date test
    machine_readable:
      execution:
        output:
          - name: current_year
            type: number
        actions:
          - output: current_year
            value: $referencedate.year
"#;
        let law = ArticleBasedLaw::from_yaml_str(yaml).unwrap();
        let article = law.find_article_by_number("1").unwrap();
        let engine = ArticleEngine::new(article, &law);

        let result = engine.evaluate(HashMap::new(), "2025-06-15").unwrap();

        assert_eq!(result.outputs.get("current_year"), Some(&Value::Int(2025)));
    }

    // -------------------------------------------------------------------------
    // Integration Tests with Real Regulation Files
    // -------------------------------------------------------------------------

    mod integration {
        use super::*;
        use std::path::PathBuf;

        fn get_regulation_path() -> PathBuf {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            PathBuf::from(manifest_dir)
                .join("..")
                .join("..")
                .join("regulation")
        }

        #[test]
        fn test_execute_zorgtoeslagwet_vermogen_check() {
            let path = get_regulation_path()
                .join("nl/wet/wet_op_de_zorgtoeslag/2025-01-01.yaml");
            let law = ArticleBasedLaw::from_yaml_file(&path).unwrap();

            // Find article that calculates vermogen_onder_grens
            let article = law.find_article_by_output("vermogen_onder_grens");
            assert!(article.is_some(), "Should find vermogen_onder_grens article");

            let article = article.unwrap();
            let engine = ArticleEngine::new(article, &law);

            // Test with vermogen under threshold for single person
            // The article requires: vermogen, heeft_toeslagpartner
            // Thresholds: €161.329 single, €203.643 with partner
            let mut params = HashMap::new();
            params.insert("vermogen".to_string(), Value::Int(100000)); // €1000 in cents, well under €161.329
            params.insert("heeft_toeslagpartner".to_string(), Value::Bool(false));

            let result = engine.evaluate(params, "2025-01-01").unwrap();

            // Should have vermogen_onder_grens output
            assert!(result.outputs.contains_key("vermogen_onder_grens"));
            assert_eq!(
                result.outputs.get("vermogen_onder_grens"),
                Some(&Value::Bool(true))
            );
        }

        #[test]
        fn test_execute_regeling_standaardpremie() {
            let path = get_regulation_path()
                .join("nl/ministeriele_regeling/regeling_standaardpremie/2025-01-01.yaml");
            let law = ArticleBasedLaw::from_yaml_file(&path).unwrap();

            // Find article with standaardpremie output
            let article = law.find_article_by_output("standaardpremie");
            assert!(article.is_some(), "Should find standaardpremie article");

            let article = article.unwrap();
            let engine = ArticleEngine::new(article, &law);

            // Execute with minimal params
            let result = engine.evaluate(HashMap::new(), "2025-01-01").unwrap();

            // Should have standaardpremie output (211200 eurocent = €2112 for 2025)
            assert_eq!(
                result.outputs.get("standaardpremie"),
                Some(&Value::Int(211200))
            );
        }
    }
}
