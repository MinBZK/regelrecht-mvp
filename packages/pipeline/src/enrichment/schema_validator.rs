use jsonschema::Validator;
use serde_json::Value;

use crate::error::{PipelineError, Result};

const SCHEMA_JSON: &str = include_str!("../../../../schema/v0.3.0/schema.json");

/// Validates machine_readable sections against the regelrecht JSON schema v0.3.0.
pub struct SchemaValidator {
    validator: Validator,
}

impl SchemaValidator {
    /// Create a new schema validator.
    ///
    /// Extracts the `machineReadableSection` definition from the full schema
    /// and uses it as the validation root.
    pub fn new() -> Result<Self> {
        let full_schema: Value =
            serde_json::from_str(SCHEMA_JSON).map_err(|e| PipelineError::SchemaLoad(e.to_string()))?;

        // Build a sub-schema that references machineReadableSection as the root.
        // We inline the definitions from the parent schema so $ref resolution works.
        let sub_schema = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "$ref": "#/definitions/machineReadableSection",
            "definitions": full_schema["definitions"]
        });

        let validator = Validator::new(&sub_schema)
            .map_err(|e| PipelineError::SchemaLoad(format!("failed to compile schema: {e}")))?;

        Ok(Self { validator })
    }

    /// Validate a machine_readable JSON value against the schema.
    ///
    /// Returns `Ok(())` if valid, or `Err(SchemaValidation)` with a list of errors.
    pub fn validate(&self, value: &Value) -> Result<()> {
        let errors: Vec<String> = self
            .validator
            .iter_errors(value)
            .map(|e| {
                let path = e.instance_path().to_string();
                if path.is_empty() {
                    e.to_string()
                } else {
                    format!("{path}: {e}")
                }
            })
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(PipelineError::SchemaValidation { errors })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = SchemaValidator::new();
        assert!(validator.is_ok(), "Schema validator should be created successfully");
    }

    #[test]
    fn test_valid_machine_readable() {
        let validator = SchemaValidator::new().expect("validator");
        let valid = serde_json::json!({
            "competent_authority": {
                "name": "Belastingdienst/Toeslagen"
            },
            "execution": {
                "parameters": [
                    {
                        "name": "bsn",
                        "type": "string",
                        "required": true
                    }
                ],
                "output": [
                    {
                        "name": "heeft_recht",
                        "type": "boolean"
                    }
                ],
                "actions": [
                    {
                        "output": "heeft_recht",
                        "value": true
                    }
                ]
            }
        });

        assert!(validator.validate(&valid).is_ok());
    }

    #[test]
    fn test_invalid_additional_properties() {
        let validator = SchemaValidator::new().expect("validator");
        let invalid = serde_json::json!({
            "unknown_field": "should not be here",
            "execution": {
                "output": [
                    {
                        "name": "test",
                        "type": "boolean"
                    }
                ]
            }
        });

        let result = validator.validate(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_with_definitions_and_actions() {
        let validator = SchemaValidator::new().expect("validator");
        // Note: actions use the `action` schema which only allows:
        // output, value, operation, values, resolve, legal_basis.
        // For comparison operations, use `value` (not `subject`).
        let valid = serde_json::json!({
            "definitions": {
                "STANDAARDPREMIE": {
                    "value": 211200
                }
            },
            "execution": {
                "parameters": [
                    {
                        "name": "bsn",
                        "type": "string",
                        "required": true
                    }
                ],
                "input": [
                    {
                        "name": "toetsingsinkomen",
                        "type": "amount",
                        "source": {
                            "regulation": "awir",
                            "output": "toetsingsinkomen",
                            "parameters": {
                                "bsn": "$bsn"
                            }
                        }
                    }
                ],
                "output": [
                    {
                        "name": "onder_grens",
                        "type": "boolean"
                    }
                ],
                "actions": [
                    {
                        "output": "onder_grens",
                        "operation": "LESS_THAN_OR_EQUAL",
                        "values": [
                            "$toetsingsinkomen",
                            "$STANDAARDPREMIE"
                        ]
                    }
                ]
            }
        });

        assert!(validator.validate(&valid).is_ok());
    }

    #[test]
    fn test_invalid_operation_type() {
        let validator = SchemaValidator::new().expect("validator");
        let invalid = serde_json::json!({
            "execution": {
                "output": [
                    {
                        "name": "test",
                        "type": "boolean"
                    }
                ],
                "actions": [
                    {
                        "output": "test",
                        "operation": "INVALID_OPERATION",
                        "value": true
                    }
                ]
            }
        });

        let result = validator.validate(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_object_is_valid() {
        // An empty machine_readable section should be valid (all fields optional)
        let validator = SchemaValidator::new().expect("validator");
        let empty = serde_json::json!({});
        assert!(validator.validate(&empty).is_ok());
    }
}
