"""
Article execution engine

Core engine for evaluating article-level machine_readable.execution sections.
"""

from dataclasses import dataclass
from typing import Any, Optional

from engine.article_loader import Article, ArticleBasedLaw
from engine.context import RuleContext, PathNode
from engine.logging_config import logger


@dataclass
class ArticleResult:
    """Result of article execution"""

    output: dict[str, Any]
    input: dict[str, Any]
    article_number: str
    law_id: str
    law_uuid: str
    path: Optional[PathNode] = None


class ArticleEngine:
    """Executes a single article's machine_readable.execution section"""

    def __init__(self, article: Article, law: ArticleBasedLaw):
        """
        Initialize article engine

        Args:
            article: Article to execute
            law: Law containing the article
        """
        self.article = article
        self.law = law
        self.machine_readable = article.machine_readable
        self.execution_spec = article.get_execution_spec()

        # Parse execution sections - can be in execution or at machine_readable level
        self.parameters = self.execution_spec.get("parameters", [])
        self.inputs = self.execution_spec.get("input", [])
        self.outputs_spec = self.execution_spec.get(
            "output", self.machine_readable.get("output", [])
        )
        self.actions = self.execution_spec.get(
            "actions", self.machine_readable.get("actions", [])
        )

        # Get article-level definitions
        self.definitions = article.get_definitions()

    def evaluate(
        self,
        parameters: dict,
        service_provider: Any,
        reference_date: str,
        requested_output: Optional[str] = None,
    ) -> ArticleResult:
        """
        Execute this article's logic

        Args:
            parameters: Input parameters (e.g., {"BSN": "123456789"})
            service_provider: Service for resolving URIs
            reference_date: Reference date for calculations
            requested_output: Specific output to calculate (optional, calculates all if None)

        Returns:
            ArticleResult with outputs and metadata
        """
        logger.info(f"Evaluating article {self.article.number} of law {self.law.id}")

        # Create execution context
        context = RuleContext(
            definitions=self.definitions,
            parameters=parameters,
            service_provider=service_provider,
            calculation_date=reference_date,
            input_specs=self.inputs,
            output_specs=self.outputs_spec,
            current_law=self.law,
        )

        # Execute actions
        self._execute_actions(context, requested_output)

        # Build result
        result = ArticleResult(
            output=context.outputs,
            input=context.resolved_inputs,
            article_number=self.article.number,
            law_id=self.law.id,
            law_uuid=self.law.uuid,
            path=context.path,
        )

        logger.info(
            f"Article evaluation complete. Outputs: {list(result.output.keys())}"
        )
        return result

    def _execute_actions(
        self, context: RuleContext, requested_output: Optional[str] = None
    ):
        """
        Execute all actions in order

        Args:
            context: Execution context
            requested_output: Specific output to calculate (optional)
        """
        # If requested_output specified, only execute actions needed for that output
        # For now, execute all actions in order
        # TODO: Implement dependency analysis and topological sort

        for action in self.actions:
            output_name = action.get("output")
            if output_name:
                # Check if we need to calculate this output
                if requested_output and output_name != requested_output:
                    continue

                logger.debug(f"Executing action for output: {output_name}")
                value = self._evaluate_action(action, context)
                context.set_output(output_name, value)
                logger.debug(f"Output {output_name} = {value}")

    def _evaluate_action(self, action: dict, context: RuleContext) -> Any:
        """
        Evaluate a single action

        Args:
            action: Action specification
            context: Execution context

        Returns:
            Calculated value
        """
        # Check for direct value
        if "value" in action:
            return self._evaluate_value(action["value"], context)

        # Check for operation
        if "operation" in action:
            return self._evaluate_operation(action, context)

        # Check for conditions (IF-THEN-ELSE)
        if "conditions" in action:
            return self._evaluate_conditions(action["conditions"], context)

        logger.warning(f"Unknown action type: {action}")
        return None

    def _evaluate_value(self, value: Any, context: RuleContext) -> Any:
        """
        Evaluate a value (may be literal, variable reference, or nested operation)

        Args:
            value: Value to evaluate
            context: Execution context

        Returns:
            Evaluated value
        """
        # Variable reference: $VARIABLE_NAME
        if isinstance(value, str) and value.startswith("$"):
            return context._resolve_value(value[1:])

        # Nested operation: {operation: ..., ...}
        if isinstance(value, dict) and "operation" in value:
            return self._evaluate_operation(value, context)

        # Literal value
        return value

    def _evaluate_operation(self, operation: dict, context: RuleContext) -> Any:
        """
        Evaluate an operation

        Args:
            operation: Operation specification
            context: Execution context

        Returns:
            Operation result
        """
        op_type = operation["operation"]

        # IF operation
        if op_type == "IF":
            return self._evaluate_if(operation, context)

        # Comparison operations
        if op_type in [
            "EQUALS",
            "NOT_EQUALS",
            "GREATER_THAN",
            "LESS_THAN",
            "GREATER_THAN_OR_EQUAL",
            "LESS_THAN_OR_EQUAL",
        ]:
            return self._evaluate_comparison(operation, context)

        # Arithmetic operations
        if op_type in ["ADD", "SUBTRACT", "MULTIPLY", "DIVIDE"]:
            return self._evaluate_arithmetic(operation, context)

        # Aggregate operations
        if op_type in ["MAX", "MIN"]:
            return self._evaluate_aggregate(operation, context)

        # Logical operations
        if op_type in ["AND", "OR"]:
            return self._evaluate_logical(operation, context)

        logger.warning(f"Unknown operation: {op_type}")
        return None

    def _evaluate_if(self, operation: dict, context: RuleContext) -> Any:
        """Evaluate IF-THEN-ELSE operation"""
        test = operation["test"]
        test_result = self._evaluate_test(test, context)

        if test_result:
            return self._evaluate_value(operation["then"], context)
        else:
            return self._evaluate_value(operation["else"], context)

    def _evaluate_test(self, test: dict, context: RuleContext) -> bool:
        """Evaluate a test condition"""
        if "operation" in test:
            return self._evaluate_operation(test, context)
        return bool(test)

    def _evaluate_comparison(self, operation: dict, context: RuleContext) -> bool:
        """Evaluate comparison operation"""
        op_type = operation["operation"]
        subject = self._evaluate_value(operation["subject"], context)
        value = self._evaluate_value(operation["value"], context)

        if op_type == "EQUALS":
            return subject == value
        elif op_type == "NOT_EQUALS":
            return subject != value
        elif op_type == "GREATER_THAN":
            return subject > value
        elif op_type == "LESS_THAN":
            return subject < value
        elif op_type == "GREATER_THAN_OR_EQUAL":
            return subject >= value
        elif op_type == "LESS_THAN_OR_EQUAL":
            return subject <= value

        return False

    def _evaluate_arithmetic(self, operation: dict, context: RuleContext) -> Any:
        """Evaluate arithmetic operation"""
        op_type = operation["operation"]
        values = operation["values"]

        # Evaluate all values
        evaluated = [self._evaluate_value(v, context) for v in values]

        # Handle nested operations
        evaluated = [
            self._evaluate_operation(v, context)
            if isinstance(v, dict) and "operation" in v
            else v
            for v in evaluated
        ]

        if op_type == "ADD":
            return sum(evaluated)
        elif op_type == "SUBTRACT":
            result = evaluated[0]
            for v in evaluated[1:]:
                result -= v
            return result
        elif op_type == "MULTIPLY":
            result = evaluated[0]
            for v in evaluated[1:]:
                result *= v
            return result
        elif op_type == "DIVIDE":
            result = evaluated[0]
            for v in evaluated[1:]:
                result /= v
            return result

        return None

    def _evaluate_aggregate(self, operation: dict, context: RuleContext) -> Any:
        """Evaluate aggregate operation (MAX, MIN)"""
        op_type = operation["operation"]
        values = operation["values"]

        # Evaluate all values
        evaluated = [self._evaluate_value(v, context) for v in values]

        # Handle nested operations
        evaluated = [
            self._evaluate_operation(v, context)
            if isinstance(v, dict) and "operation" in v
            else v
            for v in evaluated
        ]

        if op_type == "MAX":
            return max(evaluated)
        elif op_type == "MIN":
            return min(evaluated)

        return None

    def _evaluate_logical(self, operation: dict, context: RuleContext) -> bool:
        """Evaluate logical operation (AND, OR)"""
        op_type = operation["operation"]
        conditions = operation.get("conditions", [])

        if op_type == "AND":
            for condition in conditions:
                if not self._evaluate_operation(condition, context):
                    return False
            return True
        elif op_type == "OR":
            for condition in conditions:
                if self._evaluate_operation(condition, context):
                    return True
            return False

        return False

    def _evaluate_conditions(self, conditions: list, context: RuleContext) -> Any:
        """Evaluate conditions list (IF-THEN-ELSE chain)"""
        for condition in conditions:
            if "test" in condition:
                test_result = self._evaluate_test(condition["test"], context)
                if test_result:
                    return self._evaluate_value(condition["then"], context)
            elif "else" in condition:
                return self._evaluate_value(condition["else"], context)

        return None
