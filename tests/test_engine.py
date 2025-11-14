"""
Unit tests for engine.py - Article Execution Engine
"""
import pytest
from unittest.mock import Mock, MagicMock
from datetime import datetime

from engine.engine import ArticleEngine, ArticleResult
from engine.article_loader import Article, ArticleBasedLaw
from engine.context import RuleContext


def make_minimal_law(law_id="test_law", uuid="test-uuid"):
    """Helper to create minimal law for testing"""
    return ArticleBasedLaw({
        "$id": law_id,
        "uuid": uuid,
        "regulatory_layer": "WET",
        "publication_date": "2025-01-01",
        "articles": []
    })


class TestDirectValueActions:
    """Test direct value actions"""

    def test_action_with_literal_number(self):
        """Action with literal number value"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{"output": "result", "value": 42}]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 42

    def test_action_with_literal_string(self):
        """Action with literal string value"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{"output": "result", "value": "test_value"}]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "test_value"

    def test_action_with_literal_boolean(self):
        """Action with literal boolean value"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "boolean"}],
                    "actions": [{"output": "result", "value": True}]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] is True

    def test_action_with_variable_reference(self):
        """Action with variable reference ($VAR)"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "definitions": {"TEST_VALUE": 100},
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{"output": "result", "value": "$TEST_VALUE"}]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 100

    def test_action_with_undefined_variable(self):
        """Action with undefined variable reference returns None"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{"output": "result", "value": "$UNDEFINED"}]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] is None


class TestComparisonOperations:
    """Test comparison operations (used in IF/conditions)"""

    def test_equals_true(self):
        """EQUALS comparison returns true when equal"""
        # Comparisons are typically used in IF operations or conditions
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "EQUALS", "subject": 10, "value": 10},
                        "then": "equal",
                        "else": "not_equal"
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "equal"

    def test_equals_false(self):
        """EQUALS comparison returns false when not equal"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "EQUALS", "subject": 10, "value": 20},
                        "then": "equal",
                        "else": "not_equal"
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "not_equal"

    def test_not_equals(self):
        """NOT_EQUALS comparison"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "NOT_EQUALS", "subject": 10, "value": 20},
                        "then": "not_equal",
                        "else": "equal"
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "not_equal"

    def test_greater_than(self):
        """GREATER_THAN comparison"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "GREATER_THAN", "subject": 20, "value": 10},
                        "then": "greater",
                        "else": "not_greater"
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "greater"

    def test_less_than(self):
        """LESS_THAN comparison"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "LESS_THAN", "subject": 10, "value": 20},
                        "then": "less",
                        "else": "not_less"
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "less"

    def test_greater_than_or_equal(self):
        """GREATER_THAN_OR_EQUAL comparison"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "GREATER_THAN_OR_EQUAL", "subject": 10, "value": 10},
                        "then": "gte",
                        "else": "not_gte"
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "gte"

    def test_less_than_or_equal(self):
        """LESS_THAN_OR_EQUAL comparison"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "LESS_THAN_OR_EQUAL", "subject": 10, "value": 10},
                        "then": "lte",
                        "else": "not_lte"
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "lte"

    def test_comparison_with_variables(self):
        """Comparison with variable references"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "definitions": {"THRESHOLD": 18},
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "GREATER_THAN", "subject": "$THRESHOLD", "value": 15},
                        "then": "above",
                        "else": "below"
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "above"

    def test_comparison_with_none_values(self):
        """Comparison with None values"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "EQUALS", "subject": "$UNDEFINED", "value": None},
                        "then": "is_none",
                        "else": "not_none"
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "is_none"


class TestArithmeticOperations:
    """Test arithmetic operations"""

    def test_add_two_values(self):
        """ADD with two values"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "ADD",
                        "values": [10, 20]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 30

    def test_add_multiple_values(self):
        """ADD with multiple values"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "ADD",
                        "values": [10, 20, 30, 40]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 100

    def test_subtract_two_values(self):
        """SUBTRACT with two values"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "SUBTRACT",
                        "values": [50, 20]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 30

    def test_subtract_chain(self):
        """SUBTRACT chain (a - b - c)"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "SUBTRACT",
                        "values": [100, 20, 10]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 70

    def test_multiply_two_values(self):
        """MULTIPLY with two values"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "MULTIPLY",
                        "values": [5, 4]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 20

    def test_multiply_multiple_values(self):
        """MULTIPLY with multiple values"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "MULTIPLY",
                        "values": [2, 3, 4]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 24

    def test_divide_two_values(self):
        """DIVIDE with two values"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "DIVIDE",
                        "values": [100, 4]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 25

    def test_divide_by_zero(self):
        """DIVIDE by zero raises error"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "DIVIDE",
                        "values": [100, 0]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        with pytest.raises(ZeroDivisionError):
            engine.evaluate({}, Mock(), "2025-01-01")

    def test_arithmetic_with_variables(self):
        """Arithmetic with variable references"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "definitions": {"BASE": 50},
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "ADD",
                        "values": ["$BASE", 25]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 75


class TestAggregateOperations:
    """Test aggregate operations (MAX, MIN)"""

    def test_max_with_multiple_values(self):
        """MAX with multiple values"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "MAX",
                        "values": [10, 50, 30, 20]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 50

    def test_max_with_variables(self):
        """MAX with variable references"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "definitions": {"VALUE_A": 100, "VALUE_B": 150},
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "MAX",
                        "values": ["$VALUE_A", "$VALUE_B", 120]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 150

    def test_min_with_multiple_values(self):
        """MIN with multiple values"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "MIN",
                        "values": [50, 10, 30, 20]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 10

    def test_min_with_variables(self):
        """MIN with variable references"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "definitions": {"VALUE_A": 100, "VALUE_B": 150},
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "MIN",
                        "values": ["$VALUE_A", "$VALUE_B", 120]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 100

    def test_max_min_with_single_value(self):
        """MAX/MIN with single value returns that value"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "max_result", "type": "number"}, {"name": "min_result", "type": "number"}],
                    "actions": [
                        {"output": "max_result", "operation": "MAX", "values": [42]},
                        {"output": "min_result", "operation": "MIN", "values": [42]}
                    ]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["max_result"] == 42
        assert result.output["min_result"] == 42


class TestLogicalOperations:
    """Test logical operations (AND, OR)"""

    def test_and_all_true(self):
        """AND with all true conditions returns true"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "boolean"}],
                    "actions": [{
                        "output": "result",
                        "operation": "AND",
                        "conditions": [
                            {"operation": "EQUALS", "subject": 10, "value": 10},
                            {"operation": "GREATER_THAN", "subject": 20, "value": 15}
                        ]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] is True

    def test_and_one_false(self):
        """AND with one false condition returns false"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "boolean"}],
                    "actions": [{
                        "output": "result",
                        "operation": "AND",
                        "conditions": [
                            {"operation": "EQUALS", "subject": 10, "value": 10},
                            {"operation": "GREATER_THAN", "subject": 10, "value": 15}
                        ]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] is False

    def test_and_all_false(self):
        """AND with all false conditions returns false"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "boolean"}],
                    "actions": [{
                        "output": "result",
                        "operation": "AND",
                        "conditions": [
                            {"operation": "EQUALS", "subject": 10, "value": 20},
                            {"operation": "GREATER_THAN", "subject": 10, "value": 15}
                        ]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] is False

    def test_or_one_true(self):
        """OR with one true condition returns true"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "boolean"}],
                    "actions": [{
                        "output": "result",
                        "operation": "OR",
                        "conditions": [
                            {"operation": "EQUALS", "subject": 10, "value": 20},
                            {"operation": "GREATER_THAN", "subject": 20, "value": 15}
                        ]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] is True

    def test_or_all_false(self):
        """OR with all false conditions returns false"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "boolean"}],
                    "actions": [{
                        "output": "result",
                        "operation": "OR",
                        "conditions": [
                            {"operation": "EQUALS", "subject": 10, "value": 20},
                            {"operation": "GREATER_THAN", "subject": 10, "value": 15}
                        ]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] is False

    def test_or_all_true(self):
        """OR with all true conditions returns true"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "boolean"}],
                    "actions": [{
                        "output": "result",
                        "operation": "OR",
                        "conditions": [
                            {"operation": "EQUALS", "subject": 10, "value": 10},
                            {"operation": "GREATER_THAN", "subject": 20, "value": 15}
                        ]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] is True


class TestConditionalOperations:
    """Test conditional operations (IF, conditions)"""

    def test_if_true_test(self):
        """IF operation with true test returns then value"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "EQUALS", "subject": 10, "value": 10},
                        "then": 100,
                        "else": 200
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 100

    def test_if_false_test(self):
        """IF operation with false test returns else value"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "EQUALS", "subject": 10, "value": 20},
                        "then": 100,
                        "else": 200
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 200

    def test_if_without_else(self):
        """IF operation without else clause"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "EQUALS", "subject": 10, "value": 20},
                        "then": 100,
                        "else": None
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] is None

    def test_if_with_nested_operation_in_test(self):
        """IF with nested operation in test"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {
                            "operation": "GREATER_THAN",
                            "subject": {"operation": "ADD", "values": [10, 20]},
                            "value": 25
                        },
                        "then": "greater",
                        "else": "not_greater"
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "greater"

    def test_if_with_nested_operation_in_then(self):
        """IF with nested operation in then"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "EQUALS", "subject": 10, "value": 10},
                        "then": {"operation": "ADD", "values": [50, 25]},
                        "else": 0
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 75

    def test_if_with_nested_operation_in_else(self):
        """IF with nested operation in else"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {"operation": "EQUALS", "subject": 10, "value": 20},
                        "then": 0,
                        "else": {"operation": "MULTIPLY", "values": [5, 10]}
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 50

    def test_conditions_first_true(self):
        """Conditions (IF-THEN-ELSE chain) - first condition true"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "conditions": [
                            {
                                "test": {"operation": "EQUALS", "subject": 10, "value": 10},
                                "then": "first"
                            },
                            {
                                "test": {"operation": "EQUALS", "subject": 20, "value": 20},
                                "then": "second"
                            },
                            {"else": "none"}
                        ]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "first"

    def test_conditions_second_true(self):
        """Conditions - second condition true"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "conditions": [
                            {
                                "test": {"operation": "EQUALS", "subject": 10, "value": 20},
                                "then": "first"
                            },
                            {
                                "test": {"operation": "EQUALS", "subject": 20, "value": 20},
                                "then": "second"
                            },
                            {"else": "none"}
                        ]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "second"

    def test_conditions_all_false(self):
        """Conditions - all conditions false, returns else"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "conditions": [
                            {
                                "test": {"operation": "EQUALS", "subject": 10, "value": 20},
                                "then": "first"
                            },
                            {
                                "test": {"operation": "EQUALS", "subject": 20, "value": 30},
                                "then": "second"
                            },
                            {"else": "none"}
                        ]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "none"


class TestNestedOperations:
    """Test nested operations"""

    def test_nested_operations_2_levels(self):
        """Nested operations 2 levels deep"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "ADD",
                        "values": [
                            {"operation": "MULTIPLY", "values": [5, 3]},
                            10
                        ]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 25  # (5 * 3) + 10

    def test_nested_operations_3_levels(self):
        """Nested operations 3 levels deep"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "MULTIPLY",
                        "values": [
                            {
                                "operation": "ADD",
                                "values": [
                                    {"operation": "SUBTRACT", "values": [20, 5]},
                                    5
                                ]
                            },
                            2
                        ]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 40  # ((20 - 5) + 5) * 2

    def test_nested_operations_4_plus_levels(self):
        """Nested operations 4+ levels deep"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "ADD",
                        "values": [
                            {
                                "operation": "MULTIPLY",
                                "values": [
                                    {
                                        "operation": "SUBTRACT",
                                        "values": [
                                            {"operation": "ADD", "values": [10, 5]},
                                            5
                                        ]
                                    },
                                    3
                                ]
                            },
                            20
                        ]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 50  # (((10 + 5) - 5) * 3) + 20

    def test_mixed_operation_types(self):
        """Mixed operation types in nesting"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "string"}],
                    "actions": [{
                        "output": "result",
                        "operation": "IF",
                        "test": {
                            "operation": "GREATER_THAN",
                            "subject": {"operation": "ADD", "values": [10, 20]},
                            "value": {"operation": "MULTIPLY", "values": [5, 5]}
                        },
                        "then": "greater",
                        "else": "not_greater"
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == "greater"  # 30 > 25

    def test_variables_at_any_nesting_level(self):
        """Variables at any nesting level"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "definitions": {"BASE": 10, "MULTIPLIER": 3},
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "ADD",
                        "values": [
                            {
                                "operation": "MULTIPLY",
                                "values": ["$BASE", "$MULTIPLIER"]
                            },
                            20
                        ]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] == 50  # (10 * 3) + 20


class TestActionExecution:
    """Test action execution flow"""

    def test_multiple_actions_in_sequence(self):
        """Execute multiple actions in sequence"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [
                        {"name": "first", "type": "number"},
                        {"name": "second", "type": "number"}
                    ],
                    "actions": [
                        {"output": "first", "value": 100},
                        {"output": "second", "value": 200}
                    ]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["first"] == 100
        assert result.output["second"] == 200

    def test_action_using_previous_output(self):
        """Action using previous action's output"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [
                        {"name": "first", "type": "number"},
                        {"name": "second", "type": "number"}
                    ],
                    "actions": [
                        {"output": "first", "value": 50},
                        {"output": "second", "operation": "ADD", "values": ["$first", 25]}
                    ]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["first"] == 50
        assert result.output["second"] == 75

    def test_unknown_operation_type(self):
        """Unknown operation type logs warning and returns None"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{
                        "output": "result",
                        "operation": "UNKNOWN_OP",
                        "values": [10, 20]
                    }]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output["result"] is None

    def test_filter_with_requested_output(self):
        """Filter execution with requested_output parameter"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [
                        {"name": "first", "type": "number"},
                        {"name": "second", "type": "number"}
                    ],
                    "actions": [
                        {"output": "first", "value": 100},
                        {"output": "second", "value": 200}
                    ]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01", requested_output="first")

        assert result.output["first"] == 100
        assert "second" not in result.output

    def test_build_article_result_with_metadata(self):
        """Build ArticleResult with correct metadata"""
        article = Article({
            "number": "42",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{"output": "result", "value": 100}]
                }
            }
        })
        law = make_minimal_law("test_law", "test-uuid-12345")
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.article_number == "42"
        assert result.law_id == "test_law"
        assert result.law_uuid == "test-uuid-12345"
        assert isinstance(result.output, dict)
        assert isinstance(result.input, dict)

    def test_build_article_result_with_inputs(self):
        """Build ArticleResult with inputs dict"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "parameters": [{"name": "BSN", "type": "string"}],
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{"output": "result", "value": 100}]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({"BSN": "123456789"}, Mock(), "2025-01-01")

        assert isinstance(result.input, dict)

    def test_build_article_result_with_outputs(self):
        """Build ArticleResult with outputs dict"""
        article = Article({
            "number": "1",
            "text": "Test",
            "machine_readable": {
                "execution": {
                    "output": [{"name": "result", "type": "number"}],
                    "actions": [{"output": "result", "value": 42}]
                }
            }
        })
        law = make_minimal_law()
        engine = ArticleEngine(article, law)

        result = engine.evaluate({}, Mock(), "2025-01-01")

        assert result.output == {"result": 42}
