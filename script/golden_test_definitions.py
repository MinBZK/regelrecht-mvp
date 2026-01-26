"""
Golden Test Case Definitions

Test cases extracted from tests/test_engine.py and tests/test_integration.py
for cross-engine verification between Python and Rust implementations.
"""

# Base law YAML template
# NOTE: Uses example.com schema URL intentionally for testing.
# This avoids network dependencies and decouples tests from production schema changes.
# The engine doesn't validate the schema URL at runtime.
BASE_LAW_TEMPLATE = """
$schema: https://example.com/schema/v0.1.0/schema.json
$id: {law_id}
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Test article
    machine_readable:
{machine_readable}
"""


def indent_yaml(yaml_content: str, spaces: int) -> str:
    """Indent each line of YAML content by specified spaces."""
    lines = yaml_content.split("\n")
    indented = [" " * spaces + line if line.strip() else line for line in lines]
    return "\n".join(indented)


def make_law_yaml(law_id: str, machine_readable: str, article_number: str = "1") -> str:
    """Create a complete law YAML from machine_readable section."""
    return BASE_LAW_TEMPLATE.format(
        law_id=law_id, machine_readable=indent_yaml(machine_readable, 6)
    )


# =============================================================================
# Test Definitions by Category
# =============================================================================

# Category: basic_operations
BASIC_OPERATIONS_TESTS = [
    {
        "id": "basic_001_literal_number",
        "description": "Direct assignment of literal number",
        "law_yaml": make_law_yaml(
            "test_basic",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      value: 42
""",
        ),
        "law_id": "test_basic",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "basic_002_literal_string",
        "description": "Direct assignment of literal string",
        "law_yaml": make_law_yaml(
            "test_basic",
            """
execution:
  output:
    - name: result
      type: string
  actions:
    - output: result
      value: "test_value"
""",
        ),
        "law_id": "test_basic",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "basic_003_literal_boolean_true",
        "description": "Direct assignment of literal boolean true",
        "law_yaml": make_law_yaml(
            "test_basic",
            """
execution:
  output:
    - name: result
      type: boolean
  actions:
    - output: result
      value: true
""",
        ),
        "law_id": "test_basic",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "basic_004_literal_boolean_false",
        "description": "Direct assignment of literal boolean false",
        "law_yaml": make_law_yaml(
            "test_basic",
            """
execution:
  output:
    - name: result
      type: boolean
  actions:
    - output: result
      value: false
""",
        ),
        "law_id": "test_basic",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "basic_005_variable_reference",
        "description": "Variable reference from definitions",
        "law_yaml": make_law_yaml(
            "test_basic",
            """
definitions:
  TEST_VALUE: 100
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      value: $TEST_VALUE
""",
        ),
        "law_id": "test_basic",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "basic_006_parameter_reference",
        "description": "Parameter value reference",
        "law_yaml": make_law_yaml(
            "test_basic",
            """
execution:
  parameters:
    - name: input_val
      type: number
      required: true
  output:
    - name: result
      type: number
  actions:
    - output: result
      value: $input_val
""",
        ),
        "law_id": "test_basic",
        "output_name": "result",
        "parameters": {"input_val": 123},
        "calculation_date": "2025-01-01",
    },
]

# Category: comparison_operations
COMPARISON_OPERATIONS_TESTS = [
    {
        "id": "comparison_001_equals_true",
        "description": "EQUALS comparison returns true when equal",
        "law_yaml": make_law_yaml(
            "test_comparison",
            """
execution:
  output:
    - name: result
      type: string
  actions:
    - output: result
      operation: IF
      when:
        operation: EQUALS
        subject: 10
        value: 10
      then: "equal"
      else: "not_equal"
""",
        ),
        "law_id": "test_comparison",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "comparison_002_equals_false",
        "description": "EQUALS comparison returns false when not equal",
        "law_yaml": make_law_yaml(
            "test_comparison",
            """
execution:
  output:
    - name: result
      type: string
  actions:
    - output: result
      operation: IF
      when:
        operation: EQUALS
        subject: 10
        value: 20
      then: "equal"
      else: "not_equal"
""",
        ),
        "law_id": "test_comparison",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "comparison_003_not_equals",
        "description": "NOT_EQUALS comparison",
        "law_yaml": make_law_yaml(
            "test_comparison",
            """
execution:
  output:
    - name: result
      type: string
  actions:
    - output: result
      operation: IF
      when:
        operation: NOT_EQUALS
        subject: 10
        value: 20
      then: "not_equal"
      else: "equal"
""",
        ),
        "law_id": "test_comparison",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "comparison_004_greater_than_true",
        "description": "GREATER_THAN comparison true",
        "law_yaml": make_law_yaml(
            "test_comparison",
            """
execution:
  output:
    - name: result
      type: string
  actions:
    - output: result
      operation: IF
      when:
        operation: GREATER_THAN
        subject: 20
        value: 10
      then: "greater"
      else: "not_greater"
""",
        ),
        "law_id": "test_comparison",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "comparison_005_less_than_true",
        "description": "LESS_THAN comparison true",
        "law_yaml": make_law_yaml(
            "test_comparison",
            """
execution:
  output:
    - name: result
      type: string
  actions:
    - output: result
      operation: IF
      when:
        operation: LESS_THAN
        subject: 10
        value: 20
      then: "less"
      else: "not_less"
""",
        ),
        "law_id": "test_comparison",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "comparison_006_gte_equal",
        "description": "GREATER_THAN_OR_EQUAL with equal values",
        "law_yaml": make_law_yaml(
            "test_comparison",
            """
execution:
  output:
    - name: result
      type: string
  actions:
    - output: result
      operation: IF
      when:
        operation: GREATER_THAN_OR_EQUAL
        subject: 10
        value: 10
      then: "gte"
      else: "not_gte"
""",
        ),
        "law_id": "test_comparison",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "comparison_007_lte_equal",
        "description": "LESS_THAN_OR_EQUAL with equal values",
        "law_yaml": make_law_yaml(
            "test_comparison",
            """
execution:
  output:
    - name: result
      type: string
  actions:
    - output: result
      operation: IF
      when:
        operation: LESS_THAN_OR_EQUAL
        subject: 10
        value: 10
      then: "lte"
      else: "not_lte"
""",
        ),
        "law_id": "test_comparison",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "comparison_008_with_variables",
        "description": "Comparison with variable references",
        "law_yaml": make_law_yaml(
            "test_comparison",
            """
definitions:
  THRESHOLD: 18
execution:
  output:
    - name: result
      type: string
  actions:
    - output: result
      operation: IF
      when:
        operation: GREATER_THAN
        subject: $THRESHOLD
        value: 15
      then: "above"
      else: "below"
""",
        ),
        "law_id": "test_comparison",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
]

# Category: arithmetic_operations
ARITHMETIC_OPERATIONS_TESTS = [
    {
        "id": "arithmetic_001_add_two",
        "description": "ADD two values",
        "law_yaml": make_law_yaml(
            "test_arithmetic",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: ADD
      values:
        - 10
        - 20
""",
        ),
        "law_id": "test_arithmetic",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "arithmetic_002_add_multiple",
        "description": "ADD multiple values",
        "law_yaml": make_law_yaml(
            "test_arithmetic",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: ADD
      values:
        - 10
        - 20
        - 30
        - 40
""",
        ),
        "law_id": "test_arithmetic",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "arithmetic_003_subtract_two",
        "description": "SUBTRACT two values",
        "law_yaml": make_law_yaml(
            "test_arithmetic",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: SUBTRACT
      values:
        - 50
        - 20
""",
        ),
        "law_id": "test_arithmetic",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "arithmetic_004_subtract_chain",
        "description": "SUBTRACT chain (a - b - c)",
        "law_yaml": make_law_yaml(
            "test_arithmetic",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: SUBTRACT
      values:
        - 100
        - 20
        - 10
""",
        ),
        "law_id": "test_arithmetic",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "arithmetic_005_multiply_two",
        "description": "MULTIPLY two values",
        "law_yaml": make_law_yaml(
            "test_arithmetic",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: MULTIPLY
      values:
        - 5
        - 4
""",
        ),
        "law_id": "test_arithmetic",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "arithmetic_006_multiply_multiple",
        "description": "MULTIPLY multiple values",
        "law_yaml": make_law_yaml(
            "test_arithmetic",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: MULTIPLY
      values:
        - 2
        - 3
        - 4
""",
        ),
        "law_id": "test_arithmetic",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "arithmetic_007_divide_two",
        "description": "DIVIDE two values",
        "law_yaml": make_law_yaml(
            "test_arithmetic",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: DIVIDE
      values:
        - 100
        - 4
""",
        ),
        "law_id": "test_arithmetic",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "arithmetic_008_with_variables",
        "description": "Arithmetic with variable references",
        "law_yaml": make_law_yaml(
            "test_arithmetic",
            """
definitions:
  BASE: 50
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: ADD
      values:
        - $BASE
        - 25
""",
        ),
        "law_id": "test_arithmetic",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "arithmetic_009_with_parameters",
        "description": "Arithmetic with parameter values",
        "law_yaml": make_law_yaml(
            "test_arithmetic",
            """
execution:
  parameters:
    - name: a
      type: number
    - name: b
      type: number
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: ADD
      values:
        - $a
        - $b
""",
        ),
        "law_id": "test_arithmetic",
        "output_name": "result",
        "parameters": {"a": 15, "b": 27},
        "calculation_date": "2025-01-01",
    },
]

# Category: aggregate_operations
AGGREGATE_OPERATIONS_TESTS = [
    {
        "id": "aggregate_001_max_multiple",
        "description": "MAX with multiple values",
        "law_yaml": make_law_yaml(
            "test_aggregate",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: MAX
      values:
        - 10
        - 50
        - 30
        - 20
""",
        ),
        "law_id": "test_aggregate",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "aggregate_002_max_with_variables",
        "description": "MAX with variable references",
        "law_yaml": make_law_yaml(
            "test_aggregate",
            """
definitions:
  VALUE_A: 100
  VALUE_B: 150
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: MAX
      values:
        - $VALUE_A
        - $VALUE_B
        - 120
""",
        ),
        "law_id": "test_aggregate",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "aggregate_003_min_multiple",
        "description": "MIN with multiple values",
        "law_yaml": make_law_yaml(
            "test_aggregate",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: MIN
      values:
        - 50
        - 10
        - 30
        - 20
""",
        ),
        "law_id": "test_aggregate",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "aggregate_004_min_with_variables",
        "description": "MIN with variable references",
        "law_yaml": make_law_yaml(
            "test_aggregate",
            """
definitions:
  VALUE_A: 100
  VALUE_B: 150
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: MIN
      values:
        - $VALUE_A
        - $VALUE_B
        - 120
""",
        ),
        "law_id": "test_aggregate",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "aggregate_005_single_value",
        "description": "MAX/MIN with single value returns that value",
        "law_yaml": make_law_yaml(
            "test_aggregate",
            """
execution:
  output:
    - name: max_result
      type: number
    - name: min_result
      type: number
  actions:
    - output: max_result
      operation: MAX
      values:
        - 42
    - output: min_result
      operation: MIN
      values:
        - 42
""",
        ),
        "law_id": "test_aggregate",
        "output_name": "max_result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
]

# Category: logical_operations
LOGICAL_OPERATIONS_TESTS = [
    {
        "id": "logical_001_and_all_true",
        "description": "AND with all true conditions",
        "law_yaml": make_law_yaml(
            "test_logical",
            """
execution:
  output:
    - name: result
      type: boolean
  actions:
    - output: result
      operation: AND
      conditions:
        - operation: EQUALS
          subject: 10
          value: 10
        - operation: GREATER_THAN
          subject: 20
          value: 15
""",
        ),
        "law_id": "test_logical",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "logical_002_and_one_false",
        "description": "AND with one false condition",
        "law_yaml": make_law_yaml(
            "test_logical",
            """
execution:
  output:
    - name: result
      type: boolean
  actions:
    - output: result
      operation: AND
      conditions:
        - operation: EQUALS
          subject: 10
          value: 10
        - operation: GREATER_THAN
          subject: 10
          value: 15
""",
        ),
        "law_id": "test_logical",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "logical_003_or_one_true",
        "description": "OR with one true condition",
        "law_yaml": make_law_yaml(
            "test_logical",
            """
execution:
  output:
    - name: result
      type: boolean
  actions:
    - output: result
      operation: OR
      conditions:
        - operation: EQUALS
          subject: 10
          value: 20
        - operation: GREATER_THAN
          subject: 20
          value: 15
""",
        ),
        "law_id": "test_logical",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "logical_004_or_all_false",
        "description": "OR with all false conditions",
        "law_yaml": make_law_yaml(
            "test_logical",
            """
execution:
  output:
    - name: result
      type: boolean
  actions:
    - output: result
      operation: OR
      conditions:
        - operation: EQUALS
          subject: 10
          value: 20
        - operation: GREATER_THAN
          subject: 10
          value: 15
""",
        ),
        "law_id": "test_logical",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "logical_005_and_all_false",
        "description": "AND with all false conditions",
        "law_yaml": make_law_yaml(
            "test_logical",
            """
execution:
  output:
    - name: result
      type: boolean
  actions:
    - output: result
      operation: AND
      conditions:
        - operation: EQUALS
          subject: 10
          value: 20
        - operation: GREATER_THAN
          subject: 10
          value: 15
""",
        ),
        "law_id": "test_logical",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
]

# Category: conditional_operations
CONDITIONAL_OPERATIONS_TESTS = [
    {
        "id": "conditional_001_if_true",
        "description": "IF operation with true test returns then value",
        "law_yaml": make_law_yaml(
            "test_conditional",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: IF
      when:
        operation: EQUALS
        subject: 10
        value: 10
      then: 100
      else: 200
""",
        ),
        "law_id": "test_conditional",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "conditional_002_if_false",
        "description": "IF operation with false test returns else value",
        "law_yaml": make_law_yaml(
            "test_conditional",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: IF
      when:
        operation: EQUALS
        subject: 10
        value: 20
      then: 100
      else: 200
""",
        ),
        "law_id": "test_conditional",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "conditional_003_if_nested_operation_in_test",
        "description": "IF with nested operation in test",
        "law_yaml": make_law_yaml(
            "test_conditional",
            """
execution:
  output:
    - name: result
      type: string
  actions:
    - output: result
      operation: IF
      when:
        operation: GREATER_THAN
        subject:
          operation: ADD
          values:
            - 10
            - 20
        value: 25
      then: "greater"
      else: "not_greater"
""",
        ),
        "law_id": "test_conditional",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "conditional_004_if_nested_operation_in_then",
        "description": "IF with nested operation in then",
        "law_yaml": make_law_yaml(
            "test_conditional",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: IF
      when:
        operation: EQUALS
        subject: 10
        value: 10
      then:
        operation: ADD
        values:
          - 50
          - 25
      else: 0
""",
        ),
        "law_id": "test_conditional",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "conditional_005_if_nested_operation_in_else",
        "description": "IF with nested operation in else",
        "law_yaml": make_law_yaml(
            "test_conditional",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: IF
      when:
        operation: EQUALS
        subject: 10
        value: 20
      then: 0
      else:
        operation: MULTIPLY
        values:
          - 5
          - 10
""",
        ),
        "law_id": "test_conditional",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "conditional_006_switch_first_case",
        "description": "SWITCH operation - first case matches",
        "law_yaml": make_law_yaml(
            "test_conditional",
            """
execution:
  parameters:
    - name: x
      type: number
  output:
    - name: result
      type: string
  actions:
    - output: result
      value:
        operation: SWITCH
        cases:
          - when:
              operation: EQUALS
              subject: $x
              value: 10
            then: "first"
          - when:
              operation: EQUALS
              subject: $x
              value: 20
            then: "second"
        default: "none"
""",
        ),
        "law_id": "test_conditional",
        "output_name": "result",
        "parameters": {"x": 10},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "conditional_007_switch_second_case",
        "description": "SWITCH operation - second case matches",
        "law_yaml": make_law_yaml(
            "test_conditional",
            """
execution:
  parameters:
    - name: x
      type: number
  output:
    - name: result
      type: string
  actions:
    - output: result
      value:
        operation: SWITCH
        cases:
          - when:
              operation: EQUALS
              subject: $x
              value: 10
            then: "first"
          - when:
              operation: EQUALS
              subject: $x
              value: 20
            then: "second"
        default: "none"
""",
        ),
        "law_id": "test_conditional",
        "output_name": "result",
        "parameters": {"x": 20},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "conditional_008_switch_default",
        "description": "SWITCH operation - no case matches, returns default",
        "law_yaml": make_law_yaml(
            "test_conditional",
            """
execution:
  parameters:
    - name: x
      type: number
  output:
    - name: result
      type: string
  actions:
    - output: result
      value:
        operation: SWITCH
        cases:
          - when:
              operation: EQUALS
              subject: $x
              value: 10
            then: "first"
          - when:
              operation: EQUALS
              subject: $x
              value: 20
            then: "second"
        default: "none"
""",
        ),
        "law_id": "test_conditional",
        "output_name": "result",
        "parameters": {"x": 99},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "conditional_009_switch_with_nested_operation",
        "description": "SWITCH with nested operation in then value",
        "law_yaml": make_law_yaml(
            "test_conditional",
            """
definitions:
  BASE: 100
execution:
  parameters:
    - name: x
      type: number
  output:
    - name: result
      type: number
  actions:
    - output: result
      value:
        operation: SWITCH
        cases:
          - when:
              operation: EQUALS
              subject: $x
              value: 1
            then:
              operation: MULTIPLY
              values:
                - $BASE
                - 2
        default: $BASE
""",
        ),
        "law_id": "test_conditional",
        "output_name": "result",
        "parameters": {"x": 1},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "conditional_010_switch_boolean_variable",
        "description": "SWITCH with boolean variable in when clause",
        "law_yaml": make_law_yaml(
            "test_conditional",
            """
execution:
  parameters:
    - name: is_eligible
      type: boolean
    - name: is_fallback
      type: boolean
  output:
    - name: result
      type: string
  actions:
    - output: result
      value:
        operation: SWITCH
        cases:
          - when: $is_eligible
            then: "eligible"
          - when: $is_fallback
            then: "fallback"
        default: "none"
""",
        ),
        "law_id": "test_conditional",
        "output_name": "result",
        "parameters": {"is_eligible": True, "is_fallback": False},
        "calculation_date": "2025-01-01",
    },
]

# Category: nested_operations
NESTED_OPERATIONS_TESTS = [
    {
        "id": "nested_001_two_levels",
        "description": "Nested operations 2 levels deep",
        "law_yaml": make_law_yaml(
            "test_nested",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: ADD
      values:
        - operation: MULTIPLY
          values:
            - 5
            - 3
        - 10
""",
        ),
        "law_id": "test_nested",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "nested_002_three_levels",
        "description": "Nested operations 3 levels deep",
        "law_yaml": make_law_yaml(
            "test_nested",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: MULTIPLY
      values:
        - operation: ADD
          values:
            - operation: SUBTRACT
              values:
                - 20
                - 5
            - 5
        - 2
""",
        ),
        "law_id": "test_nested",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "nested_003_four_plus_levels",
        "description": "Nested operations 4+ levels deep",
        "law_yaml": make_law_yaml(
            "test_nested",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: ADD
      values:
        - operation: MULTIPLY
          values:
            - operation: SUBTRACT
              values:
                - operation: ADD
                  values:
                    - 10
                    - 5
                - 5
            - 3
        - 20
""",
        ),
        "law_id": "test_nested",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "nested_004_mixed_operation_types",
        "description": "Mixed operation types in nesting",
        "law_yaml": make_law_yaml(
            "test_nested",
            """
execution:
  output:
    - name: result
      type: string
  actions:
    - output: result
      operation: IF
      when:
        operation: GREATER_THAN
        subject:
          operation: ADD
          values:
            - 10
            - 20
        value:
          operation: MULTIPLY
          values:
            - 5
            - 5
      then: "greater"
      else: "not_greater"
""",
        ),
        "law_id": "test_nested",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "nested_005_variables_at_any_level",
        "description": "Variables at any nesting level",
        "law_yaml": make_law_yaml(
            "test_nested",
            """
definitions:
  BASE: 10
  MULTIPLIER: 3
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: ADD
      values:
        - operation: MULTIPLY
          values:
            - $BASE
            - $MULTIPLIER
        - 20
""",
        ),
        "law_id": "test_nested",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
]

# Category: action_execution
ACTION_EXECUTION_TESTS = [
    {
        "id": "action_001_multiple_actions",
        "description": "Execute multiple actions in sequence",
        "law_yaml": make_law_yaml(
            "test_action",
            """
execution:
  output:
    - name: first
      type: number
    - name: second
      type: number
  actions:
    - output: first
      value: 100
    - output: second
      value: 200
""",
        ),
        "law_id": "test_action",
        "output_name": "first",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "action_002_using_previous_output",
        # NOTE: This test currently fails during fixture generation because the Python
        # engine doesn't expose outputs from earlier actions to later actions within
        # the same article. This is a known engine limitation (see generator_error in
        # the generated fixture). The test should pass once the engine is fixed.
        "description": "Action using previous action's output",
        "law_yaml": make_law_yaml(
            "test_action",
            """
execution:
  output:
    - name: first
      type: number
    - name: second
      type: number
  actions:
    - output: first
      value: 50
    - output: second
      operation: ADD
      values:
        - $first
        - 25
""",
        ),
        "law_id": "test_action",
        "output_name": "second",
        "parameters": {},
        "calculation_date": "2025-01-01",
    },
]

# Category: cross_law_references
# These tests require multiple laws to be loaded
CROSS_LAW_TESTS = [
    {
        "id": "cross_law_001_simple_reference",
        "description": "Article in law_b calls law_a via URI",
        "multi_law": True,
        "laws": [
            {
                "yaml": """
$schema: https://example.com/schema/v0.1.0/schema.json
$id: test_law_a
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Simple arithmetic test article
    machine_readable:
      definitions:
        BASE_VALUE: 100
      execution:
        parameters:
          - name: input_value
            type: number
            required: true
        output:
          - name: add_numbers
            type: number
        actions:
          - output: add_numbers
            operation: ADD
            values:
              - $BASE_VALUE
              - $input_value
""",
                "law_id": "test_law_a",
            },
            {
                "yaml": """
$schema: https://example.com/schema/v0.1.0/schema.json
$id: test_law_b
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Cross-law test article
    machine_readable:
      execution:
        parameters:
          - name: my_value
            type: number
            required: true
        input:
          - name: result_from_a
            type: number
            source:
              regulation: test_law_a
              output: add_numbers
              parameters:
                input_value: $my_value
        output:
          - name: call_other_law
            type: number
        actions:
          - output: call_other_law
            operation: MULTIPLY
            values:
              - $result_from_a
              - 2
""",
                "law_id": "test_law_b",
            },
        ],
        "law_id": "test_law_b",
        "output_name": "call_other_law",
        "parameters": {"my_value": 25},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "cross_law_002_parameters_flow_through",
        "description": "Parameters flow through URI calls",
        "multi_law": True,
        "laws": [
            {
                "yaml": """
$schema: https://example.com/schema/v0.1.0/schema.json
$id: test_law_a
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Simple arithmetic test article
    machine_readable:
      definitions:
        BASE_VALUE: 100
      execution:
        parameters:
          - name: input_value
            type: number
            required: true
        output:
          - name: add_numbers
            type: number
        actions:
          - output: add_numbers
            operation: ADD
            values:
              - $BASE_VALUE
              - $input_value
""",
                "law_id": "test_law_a",
            },
            {
                "yaml": """
$schema: https://example.com/schema/v0.1.0/schema.json
$id: test_law_b
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Cross-law test article
    machine_readable:
      execution:
        parameters:
          - name: my_value
            type: number
            required: true
        input:
          - name: result_from_a
            type: number
            source:
              regulation: test_law_a
              output: add_numbers
              parameters:
                input_value: $my_value
        output:
          - name: call_other_law
            type: number
        actions:
          - output: call_other_law
            operation: MULTIPLY
            values:
              - $result_from_a
              - 2
""",
                "law_id": "test_law_b",
            },
        ],
        "law_id": "test_law_b",
        "output_name": "call_other_law",
        "parameters": {"my_value": 50},
        "calculation_date": "2025-01-01",
    },
]

# Category: error_cases
ERROR_CASES_TESTS = [
    {
        "id": "error_001_division_by_zero",
        "description": "DIVIDE by zero raises error",
        "law_yaml": make_law_yaml(
            "test_error",
            """
execution:
  output:
    - name: result
      type: number
  actions:
    - output: result
      operation: DIVIDE
      values:
        - 100
        - 0
""",
        ),
        "law_id": "test_error",
        "output_name": "result",
        "parameters": {},
        "calculation_date": "2025-01-01",
        "expect_error": True,
        "error_type": "DivisionByZero",
    },
    {
        "id": "error_002_missing_law_reference",
        "description": "URI referencing non-existent law raises error",
        "multi_law": True,
        "laws": [
            {
                "yaml": """
$schema: https://example.com/schema/v0.1.0/schema.json
$id: test_error
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: References missing law
    machine_readable:
      execution:
        input:
          - name: missing_value
            type: number
            source:
              regulation: nonexistent_law
              output: some_output
        output:
          - name: call_missing_law
            type: number
        actions:
          - output: call_missing_law
            value: $missing_value
""",
                "law_id": "test_error",
            }
        ],
        "law_id": "test_error",
        "output_name": "call_missing_law",
        "parameters": {},
        "calculation_date": "2025-01-01",
        "expect_error": True,
        "error_type": "LawNotFound",
    },
]

# Category: real_regulations
# These tests use real zorgtoeslagwet-style laws
REAL_REGULATIONS_TESTS = [
    {
        "id": "real_001_simple_calculation",
        "description": "Simple real-style calculation",
        "law_yaml": """
$schema: https://example.com/schema/v0.1.0/schema.json
$id: test_wet
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Bepaalt of iemand recht heeft op toeslag.
    machine_readable:
      definitions:
        MAX_INKOMEN: 50000
      execution:
        parameters:
          - name: jaarinkomen
            type: number
            required: true
        output:
          - name: heeft_recht
            type: boolean
        actions:
          - output: heeft_recht
            operation: IF
            when:
              operation: LESS_THAN_OR_EQUAL
              subject: $jaarinkomen
              value: $MAX_INKOMEN
            then: true
            else: false
""",
        "law_id": "test_wet",
        "output_name": "heeft_recht",
        "parameters": {"jaarinkomen": 45000},
        "calculation_date": "2025-01-01",
    },
    {
        "id": "real_002_toeslag_calculation",
        "description": "Calculate toeslag amount based on income",
        "law_yaml": """
$schema: https://example.com/schema/v0.1.0/schema.json
$id: test_toeslag
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Berekent de hoogte van de toeslag.
    machine_readable:
      definitions:
        DREMPEL_INKOMEN: 25000
        PERCENTAGE_ONDER_DREMPEL: 0.05
        PERCENTAGE_BOVEN_DREMPEL: 0.02
      execution:
        parameters:
          - name: inkomen
            type: number
            required: true
        output:
          - name: toeslag_bedrag
            type: number
        actions:
          - output: toeslag_bedrag
            operation: IF
            when:
              operation: LESS_THAN_OR_EQUAL
              subject: $inkomen
              value: $DREMPEL_INKOMEN
            then:
              operation: MULTIPLY
              values:
                - $inkomen
                - $PERCENTAGE_ONDER_DREMPEL
            else:
              operation: ADD
              values:
                - operation: MULTIPLY
                  values:
                    - $DREMPEL_INKOMEN
                    - $PERCENTAGE_ONDER_DREMPEL
                - operation: MULTIPLY
                  values:
                    - operation: SUBTRACT
                      values:
                        - $inkomen
                        - $DREMPEL_INKOMEN
                    - $PERCENTAGE_BOVEN_DREMPEL
""",
        "law_id": "test_toeslag",
        "output_name": "toeslag_bedrag",
        "parameters": {"inkomen": 30000},
        "calculation_date": "2025-01-01",
    },
]

# =============================================================================
# All test categories combined
# =============================================================================

ALL_TEST_CATEGORIES = {
    "basic_operations": BASIC_OPERATIONS_TESTS,
    "comparison_operations": COMPARISON_OPERATIONS_TESTS,
    "arithmetic_operations": ARITHMETIC_OPERATIONS_TESTS,
    "aggregate_operations": AGGREGATE_OPERATIONS_TESTS,
    "logical_operations": LOGICAL_OPERATIONS_TESTS,
    "conditional_operations": CONDITIONAL_OPERATIONS_TESTS,
    "nested_operations": NESTED_OPERATIONS_TESTS,
    "action_execution": ACTION_EXECUTION_TESTS,
    "cross_law_references": CROSS_LAW_TESTS,
    "error_cases": ERROR_CASES_TESTS,
    "real_regulations": REAL_REGULATIONS_TESTS,
}


def get_all_tests() -> list[dict]:
    """Get all test definitions as a flat list.

    Returns copies of test dicts to avoid mutating the originals.
    """
    all_tests = []
    for category, tests in ALL_TEST_CATEGORIES.items():
        for test in tests:
            # Create a copy to avoid mutating the original dict
            all_tests.append({**test, "category": category})
    return all_tests


def get_tests_by_category(category: str) -> list[dict]:
    """Get test definitions for a specific category.

    Returns copies of test dicts to avoid mutating the originals.
    """
    tests = ALL_TEST_CATEGORIES.get(category, [])
    # Create copies to avoid mutating the originals
    return [{**test, "category": category} for test in tests]


if __name__ == "__main__":
    # Print summary of all tests
    print("Golden Test Definitions Summary")
    print("=" * 50)
    total = 0
    for category, tests in ALL_TEST_CATEGORIES.items():
        print(f"  {category}: {len(tests)} tests")
        total += len(tests)
    print("=" * 50)
    print(f"  Total: {total} tests")
