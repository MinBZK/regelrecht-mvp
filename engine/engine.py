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
    law_uuid: str | None
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
        calculation_date: str,
        requested_output: Optional[str] = None,
    ) -> ArticleResult:
        """
        Execute this article's logic

        Args:
            parameters: Input parameters (e.g., {"BSN": "123456789"})
            service_provider: Service for resolving URIs
            calculation_date: Date for which calculations are performed
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
            calculation_date=calculation_date,
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

        # Check for resolve (cross-law reference with matching)
        if "resolve" in action:
            return self._evaluate_resolve(action["resolve"], context)

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

        # SWITCH operation
        if op_type == "SWITCH":
            return self._evaluate_switch(operation, context)

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
            return self._evaluate_value(operation.get("else"), context)

    def _evaluate_switch(self, operation: dict, context: RuleContext) -> Any:
        """Evaluate SWITCH operation (multiple conditional branches)"""
        cases = operation.get("cases", [])
        for case in cases:
            when_result = self._evaluate_value(case["when"], context)
            # Evaluate the when clause as a boolean test
            if isinstance(when_result, dict) and "operation" in when_result:
                when_result = self._evaluate_operation(when_result, context)
            if when_result:
                return self._evaluate_value(case["then"], context)
        # No case matched, return default
        return self._evaluate_value(operation.get("default"), context)

    def _evaluate_test(self, test: Any, context: RuleContext) -> bool:
        """Evaluate a test condition (can be operation, variable reference, or literal)"""
        # First evaluate the test value (handles variable references and nested operations)
        result = self._evaluate_value(test, context)
        return bool(result)

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

        def evaluate_condition(condition: Any) -> bool:
            """Evaluate a single condition in a logical operation"""
            if isinstance(condition, dict) and "operation" in condition:
                # Nested operation (e.g., {operation: EQUALS, ...})
                return bool(self._evaluate_operation(condition, context))
            else:
                # Variable reference or literal (e.g., $voldoet_aan_nationaliteit or True)
                resolved = self._evaluate_value(condition, context)
                return bool(resolved)

        if op_type == "AND":
            for condition in conditions:
                if not evaluate_condition(condition):
                    return False
            return True
        elif op_type == "OR":
            for condition in conditions:
                if evaluate_condition(condition):
                    return True
            return False

        return False

    def _evaluate_resolve(self, resolve_spec: dict, context: RuleContext) -> Any:
        """
        Evaluate a resolve action - find and call a law/regulation matching criteria

        This uses legal_basis-based resolution: it finds regelingen that explicitly
        declare the current article as their legal basis.

        Args:
            resolve_spec: Resolve specification with:
                - type: Type of regulation (e.g., "ministeriele_regeling")
                - output: Which output field to extract
                - match: Optional matching criteria (field: value pairs)
            context: Execution context

        Returns:
            Resolved value from the matched law
        """
        resolve_type = resolve_spec.get("type")
        output_field = resolve_spec.get("output")
        match_criteria = resolve_spec.get("match", {})

        logger.debug(
            f"Resolving from legal_basis: type={resolve_type}, current_law={self.law.id}, "
            f"current_article={self.article.number}, output={output_field}, match={match_criteria}"
        )

        # Find regelingen that have this article as their legal_basis
        # Access the rule_resolver through the service_provider
        if not hasattr(context.service_provider, "rule_resolver"):
            logger.error("Service provider does not have rule_resolver")
            return None

        regelingen = (
            context.service_provider.rule_resolver.find_regelingen_by_legal_basis(
                law_id=self.law.id, article=self.article.number
            )
        )

        if not regelingen:
            logger.warning(
                f"No regelingen found with legal_basis {self.law.id} article {self.article.number}"
            )
            return None

        regeling_ids = [law.id for law in regelingen]
        logger.debug(
            f"Found {len(regelingen)} regelingen with matching legal_basis: {regeling_ids}"
        )

        # Evaluate expected match value if it's a variable reference
        expected_match_value = None
        if match_criteria and "value" in match_criteria:
            expected_match_value = self._evaluate_value(
                match_criteria["value"], context
            )
            logger.debug(f"Expected match value: {expected_match_value}")

        # Track the first matching regeling to ensure a single match
        first_match = None

        # Try each regeling - error immediately if we find a second match
        for regeling_law in regelingen:
            regeling_id = regeling_law.id

            # Find the article that produces the requested output
            regeling_article = regeling_law.find_article_by_output(output_field)
            if not regeling_article:
                logger.warning(
                    f"Regeling {regeling_id}: No article found with output '{output_field}'"
                )
                continue  # Try next regeling

            try:
                # Create engine for this regeling article
                regeling_engine = ArticleEngine(regeling_article, regeling_law)

                # Phase 1: If we have match criteria, first verify the match
                # This avoids calculating expensive outputs until we know it's the right regeling
                if match_criteria and "output" in match_criteria:
                    match_output = match_criteria["output"]

                    logger.debug(f"Phase 1: Checking match criteria for {regeling_id}")
                    match_result = regeling_engine.evaluate(
                        parameters={},
                        service_provider=context.service_provider,
                        calculation_date=context.calculation_date,
                        requested_output=match_output,  # Only calculate the match field
                    )

                    if match_output not in match_result.output:
                        logger.warning(
                            f"Regeling {regeling_id}: Match output field '{match_output}' not found"
                        )
                        continue  # Try next regeling

                    regeling_match_value = match_result.output[match_output]
                    if regeling_match_value != expected_match_value:
                        logger.debug(
                            f"Regeling {regeling_id}: Match criteria not met: "
                            f"{match_output}={regeling_match_value} != {expected_match_value}, trying next"
                        )
                        continue  # Try next regeling

                    logger.debug(f"Phase 1: Match criteria satisfied for {regeling_id}")

                # Phase 2: Now calculate the actual requested output
                logger.debug(
                    f"Phase 2: Calculating output '{output_field}' for {regeling_id}"
                )
                result = regeling_engine.evaluate(
                    parameters={},
                    service_provider=context.service_provider,
                    calculation_date=context.calculation_date,
                    requested_output=output_field,  # Only calculate the requested output
                )

                # Extract the requested output field
                if output_field in result.output:
                    logger.info(f"Regeling {regeling_id} matches criteria")

                    # Check if we already found a match
                    if first_match is not None:
                        # Multiple matches - error immediately
                        error_msg = (
                            f"Multiple regelingen match for {self.law.id} article {self.article.number} "
                            f"with criteria {match_criteria}. Found at least: [{first_match['law'].id}, {regeling_id}]. "
                            f"Please add more specific match criteria to ensure deterministic resolution."
                        )
                        logger.error(error_msg)
                        raise ValueError(error_msg)

                    # Store first match
                    first_match = {
                        "law": regeling_law,
                        "result": result.output[output_field],
                    }
                else:
                    logger.error(
                        f"Regeling {regeling_id}: Output field '{output_field}' not found in result"
                    )
                    continue  # Try next regeling

            except Exception as e:
                logger.error(
                    f"Error resolving regeling {regeling_id}: {e}, trying next"
                )
                continue  # Try next regeling

        # Check if we found exactly one match
        if first_match is None:
            error_msg = (
                f"No matching regeling found for {self.law.id} article {self.article.number} "
                f"with criteria {match_criteria}"
            )
            logger.error(error_msg)
            raise ValueError(error_msg)

        # Exactly one match - return the result
        logger.info(
            f"Successfully resolved to unique regeling: {first_match['law'].id}"
        )
        return first_match["result"]
