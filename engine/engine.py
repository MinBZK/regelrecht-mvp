"""
Article execution engine

Core engine for evaluating article-level machine_readable.execution sections.
"""

from dataclasses import dataclass
from typing import Any, Optional, TYPE_CHECKING

from engine.article_loader import Article, ArticleBasedLaw
from engine.context import RuleContext, PathNode
from engine.logging_config import logger

if TYPE_CHECKING:
    from engine.data_sources import DataSourceRegistry


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
        data_registry: Optional["DataSourceRegistry"] = None,
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
            data_registry=data_registry,
        )

        # Create root path node for execution trace
        root_node = PathNode(
            type="root",
            name=f"Evaluate {self.law.id} article {self.article.number}",
            details={
                "law_id": self.law.id,
                "article": self.article.number,
                "parameters": parameters,
            },
        )
        context.add_to_path(root_node)

        # Execute actions
        self._execute_actions(context, requested_output)

        # Filter outputs if requested_output is specified
        # All actions execute (for dependencies), but only return requested output
        if requested_output:
            filtered_outputs = {
                k: v for k, v in context.outputs.items() if k == requested_output
            }
        else:
            filtered_outputs = context.outputs

        # Build result
        result = ArticleResult(
            output=filtered_outputs,
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

        Note: All actions are executed because intermediate outputs may be
        dependencies of the requested output. TODO: Implement proper
        dependency analysis to only execute necessary actions.
        """
        for action in self.actions:
            output_name = action.get("output")
            if output_name:
                # Create action path node
                action_node = PathNode(
                    type="action",
                    name=f"Calculate {output_name}",
                    details={"output": output_name},
                )
                context.add_to_path(action_node)

                with logger.indent_block(f"Action: {output_name}"):
                    value = self._evaluate_action(action, context)
                    context.set_output(output_name, value)

                    # Update action node with result
                    action_node.result = value
                    logger.debug(f"Output {output_name} = {value}")

                context.pop_path()

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
            var_name = value[1:]
            resolved = context._resolve_value(var_name)

            # Create resolve path node
            resolve_node = PathNode(
                type="resolve",
                name=f"${var_name}",
                result=resolved,
                resolve_type=self._get_resolve_type(var_name, context),
                details={"variable": var_name},
            )
            context.add_to_path(resolve_node)
            context.pop_path()

            return resolved

        # Nested operation: {operation: ..., ...}
        if isinstance(value, dict) and "operation" in value:
            return self._evaluate_operation(value, context)

        # Literal value
        return value

    def _get_resolve_type(self, var_name: str, context: RuleContext) -> str:
        """Determine how a variable was resolved"""
        if var_name in context.parameters:
            return "PARAMETER"
        elif var_name in context.definitions:
            return "DEFINITION"
        elif var_name in context.outputs:
            return "OUTPUT"
        elif var_name in context.local:
            return "LOCAL"
        elif var_name in context.resolved_inputs:
            return "URI_CALL"
        else:
            # Check if it's an input that needs resolution
            input_spec = context._find_input_spec(var_name)
            if input_spec and "source" in input_spec:
                return "URI_CALL"
            return "UNKNOWN"

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

        # Create operation path node
        op_node = PathNode(
            type="operation",
            name=op_type,
            details={"operation": op_type},
        )
        context.add_to_path(op_node)

        try:
            # IF operation
            if op_type == "IF":
                result = self._evaluate_if(operation, context)
            # SWITCH operation
            elif op_type == "SWITCH":
                result = self._evaluate_switch(operation, context)
            # Comparison operations
            elif op_type in [
                "EQUALS",
                "NOT_EQUALS",
                "GREATER_THAN",
                "LESS_THAN",
                "GREATER_THAN_OR_EQUAL",
                "LESS_THAN_OR_EQUAL",
            ]:
                result = self._evaluate_comparison(operation, context)
            # Arithmetic operations
            elif op_type in ["ADD", "SUBTRACT", "MULTIPLY", "DIVIDE"]:
                result = self._evaluate_arithmetic(operation, context)
            # Aggregate operations
            elif op_type in ["MAX", "MIN"]:
                result = self._evaluate_aggregate(operation, context)
            # Logical operations
            elif op_type in ["AND", "OR"]:
                result = self._evaluate_logical(operation, context)
            # Null checking operations
            elif op_type in ["IS_NULL", "NOT_NULL"]:
                result = self._evaluate_null_check(operation, context)
            # Membership operations
            elif op_type in ["IN", "NOT_IN"]:
                result = self._evaluate_membership(operation, context)
            # Date operations
            elif op_type == "SUBTRACT_DATE":
                result = self._evaluate_subtract_date(operation, context)
            else:
                logger.warning(f"Unknown operation: {op_type}")
                result = None

            # Update node with result
            op_node.result = result
            return result
        finally:
            context.pop_path()

    def _evaluate_if(self, operation: dict, context: RuleContext) -> Any:
        """Evaluate IF-WHEN-THEN-ELSE operation"""
        when_condition = operation["when"]
        test_result = self._evaluate_when(when_condition, context)

        if test_result:
            return self._evaluate_value(operation["then"], context)
        else:
            return self._evaluate_value(operation.get("else"), context)

    def _evaluate_switch(self, operation: dict, context: RuleContext) -> Any:
        """Evaluate SWITCH operation (multiple conditional branches)"""
        cases = operation.get("cases", [])
        for case in cases:
            test_result = self._evaluate_value(case["when"], context)
            if test_result:
                return self._evaluate_value(case["then"], context)
        return self._evaluate_value(operation.get("default"), context)

    def _evaluate_when(self, when_condition: Any, context: RuleContext) -> bool:
        """Evaluate a when condition (can be operation, variable reference, or literal)"""
        # First evaluate the when value (handles variable references and nested operations)
        result = self._evaluate_value(when_condition, context)
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

    def _evaluate_null_check(self, operation: dict, context: RuleContext) -> bool:
        """Evaluate null checking operation (IS_NULL, NOT_NULL)"""
        op_type = operation["operation"]
        subject = self._evaluate_value(operation["subject"], context)

        is_null = subject is None
        return is_null if op_type == "IS_NULL" else not is_null

    def _evaluate_membership(self, operation: dict, context: RuleContext) -> bool:
        """Evaluate membership operation (IN, NOT_IN)"""
        op_type = operation["operation"]
        subject = self._evaluate_value(operation["subject"], context)
        values = operation.get("values", [])

        # Evaluate all values in the list
        evaluated_values = [self._evaluate_value(v, context) for v in values]

        is_member = subject in evaluated_values
        return is_member if op_type == "IN" else not is_member

    def _evaluate_subtract_date(self, operation: dict, context: RuleContext) -> int:
        """
        Evaluate date subtraction operation

        Returns the difference between two dates in the specified unit.

        Args:
            operation: Operation spec with values and unit (days, months, years)
            context: Execution context

        Returns:
            Integer difference in the specified unit
        """
        from datetime import datetime, date

        values = operation.get("values", [])
        unit = operation.get("unit", "days")

        if len(values) < 2:
            logger.warning("SUBTRACT_DATE requires exactly 2 values")
            return 0

        date1 = self._evaluate_value(values[0], context)
        date2 = self._evaluate_value(values[1], context)

        # Convert strings to dates if needed
        def to_date(val):
            if isinstance(val, datetime):
                return val.date()
            elif isinstance(val, date):
                return val
            elif isinstance(val, str):
                try:
                    return datetime.strptime(val, "%Y-%m-%d").date()
                except ValueError:
                    logger.warning(f"Invalid date format: {val}")
                    return None
            return None

        d1 = to_date(date1)
        d2 = to_date(date2)

        if d1 is None or d2 is None:
            logger.warning(f"Could not parse dates: {date1}, {date2}")
            return 0

        delta = d1 - d2

        if unit == "days":
            return delta.days
        elif unit == "months":
            # Approximate months
            return delta.days // 30
        elif unit == "years":
            # Approximate years
            return delta.days // 365
        else:
            logger.warning(f"Unknown date unit: {unit}")
            return delta.days

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

        # Collect all matching regelingen to ensure exactly one match
        # Note: This code assumes single-threaded execution
        matches: list[dict] = []

        # Try each regeling and collect all matches
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

                    # Store this match
                    matches.append(
                        {
                            "law": regeling_law,
                            "result": result.output[output_field],
                            "path": result.path,
                        }
                    )
                else:
                    logger.error(
                        f"Regeling {regeling_id}: Output field '{output_field}' not found in result"
                    )
                    continue  # Try next regeling

            except (KeyError, ValueError, TypeError) as e:
                # Expected errors during resolution - try next regeling
                logger.error(
                    f"Error resolving regeling {regeling_id}: {e}, trying next"
                )
                continue  # Try next regeling
            except (MemoryError, SystemExit, KeyboardInterrupt):
                # Critical errors - re-raise immediately
                raise

        # Validate exactly one match
        if len(matches) == 0:
            error_msg = (
                f"No matching regeling found for {self.law.id} article {self.article.number} "
                f"with criteria {match_criteria}"
            )
            logger.error(error_msg)
            raise ValueError(error_msg)

        if len(matches) > 1:
            match_ids = [m["law"].id for m in matches]
            error_msg = (
                f"Multiple regelingen match for {self.law.id} article {self.article.number} "
                f"with criteria {match_criteria}. Found: {match_ids}. "
                f"Please add more specific match criteria to ensure deterministic resolution."
            )
            logger.error(error_msg)
            raise ValueError(error_msg)

        # Exactly one match - add trace and return the result
        first_match = matches[0]
        logger.info(
            f"Successfully resolved to unique regeling: {first_match['law'].id}"
        )

        # Create resolve trace node with sub-law trace
        resolve_node = PathNode(
            type="uri_call",
            name=f"Resolve {first_match['law'].id}",
            result=first_match["result"],
            details={
                "regeling_id": first_match["law"].id,
                "output": output_field,
                "match_criteria": match_criteria,
            },
        )
        context.add_to_path(resolve_node)

        # Attach sub-law trace if available
        if first_match.get("path"):
            resolve_node.add_child(first_match["path"])

        context.pop_path()

        return first_match["result"]
