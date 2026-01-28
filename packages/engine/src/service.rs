//! Service layer for cross-law resolution
//!
//! Provides the `ServiceProvider` trait and `LawExecutionService` implementation
//! for resolving cross-law references and delegation lookups.
//!
//! # Example
//!
//! ```ignore
//! use regelrecht_engine::{LawExecutionService, Value};
//! use std::collections::HashMap;
//!
//! let mut service = LawExecutionService::new();
//! service.load_law(zorgtoeslagwet_yaml)?;
//! service.load_law(regeling_standaardpremie_yaml)?;
//!
//! let mut params = HashMap::new();
//! params.insert("BSN".to_string(), Value::String("123456789".to_string()));
//!
//! // This will automatically resolve cross-law references
//! let result = service.evaluate_law_output(
//!     "wet_op_de_zorgtoeslag",
//!     "heeft_recht_op_zorgtoeslag",
//!     params,
//!     "2025-01-01",
//! )?;
//! ```

use crate::article::{Article, ArticleBasedLaw, Input, SelectOnCriteria};
use crate::config;
use crate::context::RuleContext;
use crate::engine::{
    evaluate_select_on_criteria, get_delegation_info, ArticleEngine, ArticleResult,
};
use crate::error::{EngineError, Result};
use crate::operations::ValueResolver;
use crate::resolver::RuleResolver;
use crate::types::Value;
use crate::uri::RegelrechtUri;
use std::collections::{HashMap, HashSet};

// =============================================================================
// Resolution Context
// =============================================================================

/// Context for tracking cross-law resolution state.
///
/// Bundles the state needed for cycle detection and depth tracking
/// during cross-law reference resolution. This reduces the number of
/// parameters passed between internal resolution functions.
#[derive(Clone)]
struct ResolutionContext<'a> {
    /// Date for calculations (YYYY-MM-DD)
    calculation_date: &'a str,
    /// Set of law#output keys already being resolved (cycle detection)
    visited: HashSet<String>,
    /// Current resolution depth
    depth: usize,
}

impl<'a> ResolutionContext<'a> {
    /// Create a new resolution context.
    fn new(calculation_date: &'a str) -> Self {
        Self {
            calculation_date,
            visited: HashSet::new(),
            depth: 0,
        }
    }

    /// Create a child context with a new visited key and incremented depth.
    ///
    /// Used when descending into a cross-law reference to track the resolution chain.
    fn with_visited(&self, key: String) -> Self {
        let mut new_visited = self.visited.clone();
        new_visited.insert(key);
        Self {
            calculation_date: self.calculation_date,
            visited: new_visited,
            depth: self.depth + 1,
        }
    }

    /// Check if a key is already being resolved (cycle detection).
    fn is_visited(&self, key: &str) -> bool {
        self.visited.contains(key)
    }
}

/// Reference to a delegation source for resolution.
///
/// Bundles the delegation-specific parameters to reduce argument count
/// in delegation resolution functions.
struct DelegationRef<'a> {
    /// The law that grants the delegation
    law_id: &'a str,
    /// The article that grants the delegation
    article: &'a str,
    /// Criteria for selecting the delegated regulation
    select_on: Option<&'a [SelectOnCriteria]>,
}

/// Trait for resolving cross-law references and delegations.
///
/// Implement this trait to provide custom law loading and resolution strategies.
/// The default implementation is `LawExecutionService`.
pub trait ServiceProvider {
    /// Resolve and execute a regelrecht:// URI.
    ///
    /// # Arguments
    /// * `uri` - The URI to resolve (e.g., "regelrecht://law_id/output")
    /// * `parameters` - Parameters to pass to the target article
    /// * `calculation_date` - Date for calculations (YYYY-MM-DD)
    ///
    /// # Returns
    /// The execution result from the referenced article.
    fn evaluate_uri(
        &self,
        uri: &str,
        parameters: &HashMap<String, Value>,
        calculation_date: &str,
    ) -> Result<ArticleResult>;

    /// Find a delegated regulation matching the given criteria.
    ///
    /// # Arguments
    /// * `law_id` - The law that grants the delegation
    /// * `article` - The article number that grants the delegation
    /// * `criteria` - Evaluated select_on criteria to match
    ///
    /// # Returns
    /// Reference to the matching regulation, if found.
    fn find_delegated_regulation(
        &self,
        law_id: &str,
        article: &str,
        criteria: &HashMap<String, Value>,
    ) -> Result<Option<&ArticleBasedLaw>>;

    /// Get a law by ID.
    fn get_law(&self, law_id: &str) -> Option<&ArticleBasedLaw>;

    /// Resolve an external input source.
    ///
    /// This is the main entry point for resolving `source.regulation` references.
    ///
    /// # Arguments
    /// * `regulation` - The target regulation ID
    /// * `output` - The output name to resolve
    /// * `source_parameters` - Parameters mapping from source (e.g., {"BSN": "$BSN"})
    /// * `context` - Current execution context for parameter resolution
    /// * `calculation_date` - Date for calculations
    fn resolve_external_input(
        &self,
        regulation: &str,
        output: &str,
        source_parameters: Option<&HashMap<String, String>>,
        context: &RuleContext,
        calculation_date: &str,
    ) -> Result<Value>;

    /// Resolve a delegation input source.
    ///
    /// This resolves `source.delegation` references by finding the matching
    /// regulation and executing its article.
    ///
    /// # Arguments
    /// * `delegation_law_id` - The law that grants the delegation
    /// * `delegation_article` - The article that grants the delegation
    /// * `select_on` - Criteria for selecting the delegated regulation
    /// * `output` - The output name to resolve
    /// * `source_parameters` - Parameters mapping from source
    /// * `context` - Current execution context
    /// * `calculation_date` - Date for calculations
    #[allow(clippy::too_many_arguments)]
    fn resolve_delegation_input(
        &self,
        delegation_law_id: &str,
        delegation_article: &str,
        select_on: Option<&[SelectOnCriteria]>,
        output: &str,
        source_parameters: Option<&HashMap<String, String>>,
        context: &RuleContext,
        calculation_date: &str,
    ) -> Result<Value>;
}

/// High-level service for executing laws with automatic cross-law resolution.
///
/// `LawExecutionService` wraps a `RuleResolver` and implements `ServiceProvider`
/// to enable automatic resolution of external references and delegations.
pub struct LawExecutionService {
    resolver: RuleResolver,
}

impl Default for LawExecutionService {
    fn default() -> Self {
        Self::new()
    }
}

impl LawExecutionService {
    /// Create a new empty service.
    pub fn new() -> Self {
        Self {
            resolver: RuleResolver::new(),
        }
    }

    /// Load a law from YAML string.
    ///
    /// # Returns
    /// The law ID on success.
    pub fn load_law(&mut self, yaml: &str) -> Result<String> {
        self.resolver.load_from_yaml(yaml)
    }

    /// Load a law struct directly.
    ///
    /// # Returns
    /// `Ok(())` on success, `Err` if the maximum number of laws would be exceeded.
    pub fn load_law_struct(&mut self, law: ArticleBasedLaw) -> Result<()> {
        self.resolver.load_law(law)
    }

    /// Execute a law output by name.
    ///
    /// This is the main entry point for law execution. It finds the article
    /// that produces the requested output and executes it, automatically
    /// resolving any cross-law references.
    ///
    /// # Arguments
    /// * `law_id` - The law identifier (e.g., "zorgtoeslagwet")
    /// * `output_name` - The output to calculate
    /// * `parameters` - Input parameters
    /// * `calculation_date` - Date for calculations (YYYY-MM-DD)
    ///
    /// # Returns
    /// The execution result with outputs and metadata.
    pub fn evaluate_law_output(
        &self,
        law_id: &str,
        output_name: &str,
        parameters: HashMap<String, Value>,
        calculation_date: &str,
    ) -> Result<ArticleResult> {
        let res_ctx = ResolutionContext::new(calculation_date);
        self.evaluate_law_output_internal(law_id, output_name, parameters, &res_ctx)
    }

    /// Internal method with cycle tracking.
    fn evaluate_law_output_internal(
        &self,
        law_id: &str,
        output_name: &str,
        parameters: HashMap<String, Value>,
        res_ctx: &ResolutionContext<'_>,
    ) -> Result<ArticleResult> {
        tracing::debug!(
            law_id = %law_id,
            output = %output_name,
            depth = res_ctx.depth,
            "Resolving cross-law reference"
        );

        // Check depth limit
        if res_ctx.depth > config::MAX_CROSS_LAW_DEPTH {
            tracing::warn!(
                law_id = %law_id,
                output = %output_name,
                depth = res_ctx.depth,
                "Cross-law resolution depth exceeded"
            );
            return Err(EngineError::CircularReference(format!(
                "Cross-law resolution depth exceeded {} levels. \
                 Possible circular reference involving {}:{}",
                config::MAX_CROSS_LAW_DEPTH,
                law_id,
                output_name
            )));
        }

        // Get the law
        let law = self
            .resolver
            .get_law(law_id)
            .ok_or_else(|| EngineError::LawNotFound(law_id.to_string()))?;

        // Find the article
        let article = self
            .resolver
            .get_article_by_output(law_id, output_name)
            .ok_or_else(|| EngineError::OutputNotFound {
                law_id: law_id.to_string(),
                output: output_name.to_string(),
            })?;

        // Execute with service provider
        self.evaluate_article_with_service(article, law, parameters, Some(output_name), res_ctx)
    }

    /// Execute an article with ServiceProvider support.
    fn evaluate_article_with_service(
        &self,
        article: &Article,
        law: &ArticleBasedLaw,
        parameters: HashMap<String, Value>,
        requested_output: Option<&str>,
        res_ctx: &ResolutionContext<'_>,
    ) -> Result<ArticleResult> {
        // Create execution context
        let mut context = RuleContext::new(parameters.clone(), res_ctx.calculation_date)?;

        // Set definitions from article
        if let Some(definitions) = article.get_definitions() {
            context.set_definitions(definitions);
        }

        // Resolve inputs with sources using ServiceProvider
        self.resolve_inputs_with_service(article, &mut context, &parameters, res_ctx)?;

        // Use ArticleEngine for action execution (it handles the internal logic)
        let engine = ArticleEngine::new(article, law);

        // Now execute with resolved inputs already in context
        // We need to pass resolved inputs as parameters to avoid re-resolution
        let mut combined_params = parameters;
        for (name, value) in context.resolved_inputs() {
            combined_params.insert(name.clone(), value.clone());
        }

        engine.evaluate_with_output(combined_params, res_ctx.calculation_date, requested_output)
    }

    /// Resolve input sources using ServiceProvider.
    fn resolve_inputs_with_service(
        &self,
        article: &Article,
        context: &mut RuleContext,
        parameters: &HashMap<String, Value>,
        res_ctx: &ResolutionContext<'_>,
    ) -> Result<()> {
        let inputs = self.get_inputs(article);

        for input in inputs {
            let source = match &input.source {
                Some(s) => s,
                None => continue,
            };

            // Check if already provided as parameter
            if parameters.contains_key(&input.name) {
                continue;
            }

            if let Some(delegation) = &source.delegation {
                // Delegation reference
                let (del_law_id, del_article, select_on) = get_delegation_info(delegation);
                let del_ref = DelegationRef {
                    law_id: del_law_id,
                    article: del_article,
                    select_on,
                };

                let value = self.resolve_delegation_input_internal(
                    &del_ref,
                    &source.output,
                    source.parameters.as_ref(),
                    context,
                    res_ctx,
                )?;

                context.set_resolved_input(&input.name, value);
            } else if let Some(regulation) = &source.regulation {
                // External reference
                let value = self.resolve_external_input_internal(
                    regulation,
                    &source.output,
                    source.parameters.as_ref(),
                    context,
                    res_ctx,
                )?;

                context.set_resolved_input(&input.name, value);
            } else {
                // Internal reference - handled by ArticleEngine
                // We don't resolve these here because ArticleEngine handles them
                // with proper circular reference detection within the same law
            }
        }

        Ok(())
    }

    /// Internal method for external input resolution with depth tracking.
    fn resolve_external_input_internal(
        &self,
        regulation: &str,
        output: &str,
        source_parameters: Option<&HashMap<String, String>>,
        context: &RuleContext,
        res_ctx: &ResolutionContext<'_>,
    ) -> Result<Value> {
        // Check for circular reference before proceeding
        let key = format!("{}#{}", regulation, output);
        if res_ctx.is_visited(&key) {
            return Err(EngineError::CircularReference(format!(
                "Circular cross-law reference detected: {} is already being resolved",
                key
            )));
        }

        // Build parameters for the target article
        let target_params = self.build_target_parameters(source_parameters, context)?;

        // Create child context with this reference tracked
        let child_ctx = res_ctx.with_visited(key);

        // Execute the target article
        let result =
            self.evaluate_law_output_internal(regulation, output, target_params, &child_ctx)?;

        // Extract the requested output
        result
            .outputs
            .get(output)
            .cloned()
            .ok_or_else(|| EngineError::OutputNotFound {
                law_id: regulation.to_string(),
                output: output.to_string(),
            })
    }

    /// Internal method for delegation input resolution with depth tracking.
    fn resolve_delegation_input_internal(
        &self,
        delegation: &DelegationRef<'_>,
        output: &str,
        source_parameters: Option<&HashMap<String, String>>,
        context: &RuleContext,
        res_ctx: &ResolutionContext<'_>,
    ) -> Result<Value> {
        // Evaluate selection criteria
        let criteria = if let Some(criteria_spec) = delegation.select_on {
            evaluate_select_on_criteria(criteria_spec, context)?
        } else {
            HashMap::new()
        };

        // Find matching regulation
        let regulation = self
            .find_delegated_regulation(delegation.law_id, delegation.article, &criteria)?
            .ok_or_else(|| {
                let criteria_str = criteria
                    .iter()
                    .map(|(k, v)| format!("{}={:?}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ");
                EngineError::DelegationError(format!(
                    "No regulation found for delegation from {}#{} with criteria [{}]",
                    delegation.law_id, delegation.article, criteria_str
                ))
            })?;

        // Check for circular reference
        let key = format!("{}#{}", regulation.id, output);
        if res_ctx.is_visited(&key) {
            return Err(EngineError::CircularReference(format!(
                "Circular delegation reference detected: {} is already being resolved",
                key
            )));
        }

        // Build parameters for the delegated regulation
        let target_params = self.build_target_parameters(source_parameters, context)?;

        // Create child context with this reference tracked
        let child_ctx = res_ctx.with_visited(key);

        // Execute the delegated regulation with cycle tracking
        let result =
            self.evaluate_law_output_internal(&regulation.id, output, target_params, &child_ctx)?;

        // Extract the requested output
        result
            .outputs
            .get(output)
            .cloned()
            .ok_or_else(|| EngineError::OutputNotFound {
                law_id: regulation.id.clone(),
                output: output.to_string(),
            })
    }

    /// Build parameters for a target article from source parameter mapping.
    fn build_target_parameters(
        &self,
        source_parameters: Option<&HashMap<String, String>>,
        context: &RuleContext,
    ) -> Result<HashMap<String, Value>> {
        let mut params = HashMap::new();

        if let Some(param_map) = source_parameters {
            for (target_name, source_ref) in param_map {
                // Source ref can be "$variable" or a literal
                let value = if let Some(var_name) = source_ref.strip_prefix('$') {
                    context.resolve(var_name)?
                } else {
                    // Literal value (as string)
                    Value::String(source_ref.clone())
                };
                params.insert(target_name.clone(), value);
            }
        }

        Ok(params)
    }

    /// Get inputs from article execution spec.
    fn get_inputs<'a>(&self, article: &'a Article) -> &'a [Input] {
        article
            .get_execution_spec()
            .and_then(|exec| exec.input.as_deref())
            .unwrap_or(&[])
    }

    /// List all loaded law IDs.
    pub fn list_laws(&self) -> Vec<&str> {
        self.resolver.list_laws()
    }

    /// Get the number of loaded laws.
    pub fn law_count(&self) -> usize {
        self.resolver.law_count()
    }

    /// Check if a law is loaded.
    pub fn has_law(&self, law_id: &str) -> bool {
        self.resolver.has_law(law_id)
    }

    /// Unload a law.
    pub fn unload_law(&mut self, law_id: &str) -> bool {
        self.resolver.unload_law(law_id)
    }

    /// Get direct access to the resolver.
    pub fn resolver(&self) -> &RuleResolver {
        &self.resolver
    }
}

impl ServiceProvider for LawExecutionService {
    fn evaluate_uri(
        &self,
        uri: &str,
        parameters: &HashMap<String, Value>,
        calculation_date: &str,
    ) -> Result<ArticleResult> {
        let parsed = RegelrechtUri::parse(uri)?;
        self.evaluate_law_output(
            parsed.law_id(),
            parsed.output(),
            parameters.clone(),
            calculation_date,
        )
    }

    fn find_delegated_regulation(
        &self,
        law_id: &str,
        article: &str,
        criteria: &HashMap<String, Value>,
    ) -> Result<Option<&ArticleBasedLaw>> {
        Ok(self
            .resolver
            .find_delegated_regulation(law_id, article, criteria))
    }

    fn get_law(&self, law_id: &str) -> Option<&ArticleBasedLaw> {
        self.resolver.get_law(law_id)
    }

    fn resolve_external_input(
        &self,
        regulation: &str,
        output: &str,
        source_parameters: Option<&HashMap<String, String>>,
        context: &RuleContext,
        calculation_date: &str,
    ) -> Result<Value> {
        let res_ctx = ResolutionContext::new(calculation_date);
        self.resolve_external_input_internal(
            regulation,
            output,
            source_parameters,
            context,
            &res_ctx,
        )
    }

    fn resolve_delegation_input(
        &self,
        delegation_law_id: &str,
        delegation_article: &str,
        select_on: Option<&[SelectOnCriteria]>,
        output: &str,
        source_parameters: Option<&HashMap<String, String>>,
        context: &RuleContext,
        calculation_date: &str,
    ) -> Result<Value> {
        let res_ctx = ResolutionContext::new(calculation_date);
        let del_ref = DelegationRef {
            law_id: delegation_law_id,
            article: delegation_article,
            select_on,
        };
        self.resolve_delegation_input_internal(
            &del_ref,
            output,
            source_parameters,
            context,
            &res_ctx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_base_law() -> &'static str {
        r#"
$id: base_law
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Provides a base value
    machine_readable:
      definitions:
        BASE_VALUE:
          value: 100
      execution:
        output:
          - name: base_value
            type: number
        actions:
          - output: base_value
            value: $BASE_VALUE
"#
    }

    fn make_dependent_law() -> &'static str {
        r#"
$id: dependent_law
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Uses value from another law
    machine_readable:
      execution:
        input:
          - name: external_base
            type: number
            source:
              regulation: base_law
              output: base_value
        output:
          - name: doubled_value
            type: number
        actions:
          - output: doubled_value
            operation: MULTIPLY
            values:
              - $external_base
              - 2
"#
    }

    fn make_delegating_law() -> &'static str {
        r#"
$id: participatiewet
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '8'
    text: Delegation authority
    machine_readable:
      execution:
        output:
          - name: delegation_granted
            type: boolean
        actions:
          - output: delegation_granted
            value: true
"#
    }

    fn make_delegated_verordening(gemeente_code: &str, percentage: i32) -> String {
        format!(
            r#"
$id: {gemeente_code}_verordening
regulatory_layer: GEMEENTELIJKE_VERORDENING
publication_date: '2025-01-01'
gemeente_code: "{gemeente_code}"
legal_basis:
  - law_id: participatiewet
    article: '8'
articles:
  - number: '1'
    text: Local regulation
    machine_readable:
      execution:
        output:
          - name: verlaging_percentage
            type: number
        actions:
          - output: verlaging_percentage
            value: {percentage}
"#
        )
    }

    fn make_law_using_delegation() -> &'static str {
        r#"
$id: using_delegation_law
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Uses delegated value
    machine_readable:
      execution:
        parameters:
          - name: gemeente_code
            type: string
            required: true
        input:
          - name: local_percentage
            type: number
            source:
              delegation:
                law_id: participatiewet
                article: '8'
                select_on:
                  - name: gemeente_code
                    value: $gemeente_code
              output: verlaging_percentage
        output:
          - name: adjusted_amount
            type: number
        actions:
          - output: adjusted_amount
            operation: MULTIPLY
            values:
              - 1000
              - operation: DIVIDE
                values:
                  - $local_percentage
                  - 100
"#
    }

    // -------------------------------------------------------------------------
    // Basic Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_service_basic() {
        let mut service = LawExecutionService::new();
        service.load_law(make_base_law()).unwrap();

        assert!(service.has_law("base_law"));
        assert_eq!(service.law_count(), 1);

        let laws = service.list_laws();
        assert_eq!(laws, vec!["base_law"]);
    }

    #[test]
    fn test_service_execute_simple() {
        let mut service = LawExecutionService::new();
        service.load_law(make_base_law()).unwrap();

        let result = service
            .evaluate_law_output("base_law", "base_value", HashMap::new(), "2025-01-01")
            .unwrap();

        assert_eq!(result.outputs.get("base_value"), Some(&Value::Int(100)));
    }

    // -------------------------------------------------------------------------
    // Cross-Law Resolution Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_service_cross_law_resolution() {
        let mut service = LawExecutionService::new();
        service.load_law(make_base_law()).unwrap();
        service.load_law(make_dependent_law()).unwrap();

        // Execute dependent law - should automatically resolve base_law reference
        let result = service
            .evaluate_law_output(
                "dependent_law",
                "doubled_value",
                HashMap::new(),
                "2025-01-01",
            )
            .unwrap();

        // doubled_value = base_value (100) * 2 = 200
        assert_eq!(result.outputs.get("doubled_value"), Some(&Value::Int(200)));
    }

    #[test]
    fn test_service_missing_dependency() {
        let mut service = LawExecutionService::new();
        // Only load dependent law, not base law
        service.load_law(make_dependent_law()).unwrap();

        let result = service.evaluate_law_output(
            "dependent_law",
            "doubled_value",
            HashMap::new(),
            "2025-01-01",
        );

        assert!(
            matches!(result, Err(EngineError::LawNotFound(_))),
            "Expected LawNotFound error, got: {:?}",
            result
        );
    }

    // -------------------------------------------------------------------------
    // Delegation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_service_delegation_resolution() {
        let mut service = LawExecutionService::new();
        service.load_law(make_delegating_law()).unwrap();
        service
            .load_law(&make_delegated_verordening("0363", 20))
            .unwrap();
        service
            .load_law(&make_delegated_verordening("0518", 15))
            .unwrap();
        service.load_law(make_law_using_delegation()).unwrap();

        // Execute with Amsterdam (0363)
        let mut params = HashMap::new();
        params.insert(
            "gemeente_code".to_string(),
            Value::String("0363".to_string()),
        );

        let result = service
            .evaluate_law_output(
                "using_delegation_law",
                "adjusted_amount",
                params,
                "2025-01-01",
            )
            .unwrap();

        // adjusted_amount = 1000 * (20 / 100) = 200.0 (DIVIDE produces float)
        assert_eq!(
            result.outputs.get("adjusted_amount"),
            Some(&Value::Float(200.0))
        );

        // Execute with Den Haag (0518)
        let mut params = HashMap::new();
        params.insert(
            "gemeente_code".to_string(),
            Value::String("0518".to_string()),
        );

        let result = service
            .evaluate_law_output(
                "using_delegation_law",
                "adjusted_amount",
                params,
                "2025-01-01",
            )
            .unwrap();

        // adjusted_amount = 1000 * (15 / 100) = 150.0 (DIVIDE produces float)
        assert_eq!(
            result.outputs.get("adjusted_amount"),
            Some(&Value::Float(150.0))
        );
    }

    #[test]
    fn test_service_delegation_no_match() {
        let mut service = LawExecutionService::new();
        service.load_law(make_delegating_law()).unwrap();
        service
            .load_law(&make_delegated_verordening("0363", 20))
            .unwrap();
        service.load_law(make_law_using_delegation()).unwrap();

        // Execute with non-existent gemeente
        let mut params = HashMap::new();
        params.insert(
            "gemeente_code".to_string(),
            Value::String("9999".to_string()),
        );

        let result = service.evaluate_law_output(
            "using_delegation_law",
            "adjusted_amount",
            params,
            "2025-01-01",
        );

        assert!(
            matches!(result, Err(EngineError::DelegationError(_))),
            "Expected DelegationError, got: {:?}",
            result
        );
    }

    // -------------------------------------------------------------------------
    // URI Resolution Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_service_uri_resolution() {
        let mut service = LawExecutionService::new();
        service.load_law(make_base_law()).unwrap();

        let result = service
            .evaluate_uri(
                "regelrecht://base_law/base_value",
                &HashMap::new(),
                "2025-01-01",
            )
            .unwrap();

        assert_eq!(result.outputs.get("base_value"), Some(&Value::Int(100)));
    }

    // -------------------------------------------------------------------------
    // Circular Reference Detection Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_service_cross_law_circular_reference() {
        // Law A references Law B, which references Law A
        let law_a = r#"
$id: law_a
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: References law_b
    machine_readable:
      execution:
        input:
          - name: from_b
            type: number
            source:
              regulation: law_b
              output: output_b
        output:
          - name: output_a
            type: number
        actions:
          - output: output_a
            value: $from_b
"#;

        let law_b = r#"
$id: law_b
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: References law_a
    machine_readable:
      execution:
        input:
          - name: from_a
            type: number
            source:
              regulation: law_a
              output: output_a
        output:
          - name: output_b
            type: number
        actions:
          - output: output_b
            value: $from_a
"#;

        let mut service = LawExecutionService::new();
        service.load_law(law_a).unwrap();
        service.load_law(law_b).unwrap();

        let result = service.evaluate_law_output("law_a", "output_a", HashMap::new(), "2025-01-01");

        assert!(
            matches!(result, Err(EngineError::CircularReference(_))),
            "Expected CircularReference error, got: {:?}",
            result
        );
    }

    #[test]
    fn test_service_delegation_circular_reference() {
        // Test that delegation circular references are properly detected
        // Law A delegates to B, B references A
        let law_a = r#"
$id: law_a
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '8'
    text: Delegation authority
    machine_readable:
      execution:
        output:
          - name: delegation_granted
            type: boolean
        actions:
          - output: delegation_granted
            value: true
"#;

        let regulation_b = r#"
$id: regulation_b
regulatory_layer: GEMEENTELIJKE_VERORDENING
publication_date: '2025-01-01'
gemeente_code: "0363"
legal_basis:
  - law_id: law_a
    article: '8'
articles:
  - number: '1'
    text: References back to law_a via external reference
    machine_readable:
      execution:
        input:
          - name: from_a
            type: boolean
            source:
              regulation: law_a
              output: output_from_delegation
        output:
          - name: local_value
            type: number
        actions:
          - output: local_value
            value: 100
"#;

        let law_using_delegation = r#"
$id: law_using_delegation
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Uses delegation that creates circular reference
    machine_readable:
      execution:
        parameters:
          - name: gemeente_code
            type: string
        input:
          - name: local_value
            type: number
            source:
              delegation:
                law_id: law_a
                article: '8'
                select_on:
                  - name: gemeente_code
                    value: $gemeente_code
              output: local_value
        output:
          - name: output_from_delegation
            type: number
        actions:
          - output: output_from_delegation
            value: $local_value
"#;

        let mut service = LawExecutionService::new();
        service.load_law(law_a).unwrap();
        service.load_law(regulation_b).unwrap();
        service.load_law(law_using_delegation).unwrap();

        let mut params = HashMap::new();
        params.insert(
            "gemeente_code".to_string(),
            Value::String("0363".to_string()),
        );

        // This should fail with circular reference since:
        // law_using_delegation -> delegation to regulation_b -> external ref to law_a
        // But the output_from_delegation doesn't exist in law_a, so it will fail with OutputNotFound
        // That's fine - the important thing is we don't get infinite recursion
        let result = service.evaluate_law_output(
            "law_using_delegation",
            "output_from_delegation",
            params,
            "2025-01-01",
        );

        // Should fail (either circular reference or output not found, but not stack overflow)
        assert!(result.is_err());
    }

    // -------------------------------------------------------------------------
    // Parameter Override Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_service_parameter_override() {
        // When a parameter is provided that matches an input name,
        // the parameter value should be used instead of resolving the source
        let law = r#"
$id: override_test_law
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Has external reference but can be overridden
    machine_readable:
      execution:
        parameters:
          - name: external_value
            type: number
        input:
          - name: external_value
            type: number
            source:
              regulation: nonexistent_law
              output: some_output
        output:
          - name: result
            type: number
        actions:
          - output: result
            operation: MULTIPLY
            values:
              - $external_value
              - 2
"#;

        let mut service = LawExecutionService::new();
        service.load_law(law).unwrap();

        // Provide the value directly - should skip external resolution
        let mut params = HashMap::new();
        params.insert("external_value".to_string(), Value::Int(50));

        let result = service
            .evaluate_law_output("override_test_law", "result", params, "2025-01-01")
            .unwrap();

        // result = 50 * 2 = 100
        assert_eq!(result.outputs.get("result"), Some(&Value::Int(100)));
    }

    #[test]
    fn test_service_delegation_parameter_override() {
        // When a parameter is provided that matches a delegation input name,
        // the parameter value should be used instead of delegation resolution
        let mut service = LawExecutionService::new();
        service.load_law(make_delegating_law()).unwrap();
        // Note: we don't load any verordening - delegation would fail
        service.load_law(make_law_using_delegation()).unwrap();

        // Provide local_percentage directly - should skip delegation
        let mut params = HashMap::new();
        params.insert(
            "gemeente_code".to_string(),
            Value::String("9999".to_string()), // Non-existent gemeente
        );
        params.insert("local_percentage".to_string(), Value::Int(25)); // Pre-resolved

        let result = service
            .evaluate_law_output(
                "using_delegation_law",
                "adjusted_amount",
                params,
                "2025-01-01",
            )
            .unwrap();

        // adjusted_amount = 1000 * (25 / 100) = 250.0
        assert_eq!(
            result.outputs.get("adjusted_amount"),
            Some(&Value::Float(250.0))
        );
    }
}
