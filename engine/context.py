"""
Execution context for article evaluation

Manages state and value resolution during article execution.
"""

from dataclasses import dataclass, field
from typing import Any, Optional
import copy
from datetime import datetime

from engine.logging_config import logger


@dataclass
class PathNode:
    """Represents a node in the execution trace"""

    type: str  # "resolve", "operation", "action", "requirement", "uri_call"
    name: str
    result: Any = None
    resolve_type: Optional[str] = None  # "URI", "PARAMETER", "DEFINITION", "OUTPUT"
    required: bool = False
    details: dict = field(default_factory=dict)
    children: list["PathNode"] = field(default_factory=list)

    def add_child(self, child: "PathNode"):
        """Add a child node to the trace"""
        self.children.append(child)


class RuleContext:
    """Execution context for article evaluation"""

    def __init__(
        self,
        definitions: dict[str, Any],
        parameters: dict[str, Any],
        service_provider: Any,
        calculation_date: str,
        input_specs: list[dict] | None = None,
        output_specs: list[dict] | None = None,
        current_law: Any = None,
    ):
        """
        Initialize execution context

        Args:
            definitions: Article-level definitions
            parameters: Input parameters (e.g., {"BSN": "123456789"})
            service_provider: Service for resolving URIs
            calculation_date: Reference date for calculations (YYYY-MM-DD)
            input_specs: Input specifications from execution section
            output_specs: Output specifications from execution section
            current_law: The law being executed (for resolving # references)
        """
        self.definitions = self._process_definitions(definitions)
        self.parameters = parameters
        self.service_provider = service_provider
        self.calculation_date = calculation_date
        self.input_specs = input_specs or []
        self.output_specs = output_specs or []
        self.current_law = current_law

        # Parse calculation date as datetime object for use as $referencedate context variable
        try:
            self.reference_date = datetime.strptime(calculation_date, "%Y-%m-%d")
        except (ValueError, TypeError):
            logger.warning(
                f"Invalid calculation_date format: {calculation_date}, using current date"
            )
            self.reference_date = datetime.now()

        # Execution state
        self.outputs: dict[str, Any] = {}
        self.local: dict[str, Any] = {}  # For loop variables
        self.resolved_inputs: dict[str, Any] = {}

        # Caching
        self._uri_cache: dict[str, Any] = {}

        # Execution trace
        self.path: Optional[PathNode] = None
        self.current_path: Optional[PathNode] = None

    def _process_definitions(self, definitions: dict) -> dict:
        """
        Process definitions to extract values

        Definitions can be:
        - Simple: CONSTANT: 123
        - Complex: CONSTANT: {value: 123, legal_basis: {...}}
        """
        processed = {}
        for key, value in definitions.items():
            if isinstance(value, dict) and "value" in value:
                processed[key] = value["value"]
            else:
                processed[key] = value
        return processed

    def _find_input_spec(self, name: str) -> Optional[dict]:
        """Find input specification by name"""
        for spec in self.input_specs:
            if spec.get("name") == name:
                return spec
        return None

    def _resolve_value(self, path: str) -> Any:
        """
        Resolve a variable reference

        Resolution priority:
        1. Context variables (referencedate)
        2. Local scope (loop variables)
        3. Outputs (calculated values)
        4. Resolved inputs
        5. Definitions (constants)
        6. Parameters (direct inputs)
        7. Input with source.url (cross-law reference)

        Supports dot notation for property access (e.g., referencedate.year)

        Args:
            path: Variable name or path (e.g., "referencedate.year")

        Returns:
            Resolved value or None
        """
        # Handle dot notation for property access
        if "." in path:
            parts = path.split(".", 1)
            base_var = parts[0]
            property_path = parts[1]

            # Resolve the base variable
            base_value = self._resolve_value(base_var)
            if base_value is None:
                logger.warning(f"Could not resolve base variable: {base_var}")
                return None

            # Navigate the property path
            return self._get_property(base_value, property_path)

        # 1. Context variables (special built-in variables)
        if path == "referencedate":
            return self.reference_date

        # 2. Local scope (FOREACH loop variables)
        if path in self.local:
            return self.local[path]

        # 3. Outputs (calculated values)
        if path in self.outputs:
            return self.outputs[path]

        # 4. Resolved inputs (already fetched)
        if path in self.resolved_inputs:
            return self.resolved_inputs[path]

        # 5. Definitions (constants)
        if path in self.definitions:
            return self.definitions[path]

        # 6. Parameters (direct inputs)
        if path in self.parameters:
            return self.parameters[path]

        # 7. Input with source - need to resolve
        input_spec = self._find_input_spec(path)
        if input_spec and "source" in input_spec:
            value = self._resolve_from_source(input_spec["source"], path)
            self.resolved_inputs[path] = value
            return value

        logger.warning(f"Could not resolve variable: {path}")
        return None

    def _get_property(self, obj: Any, property_path: str) -> Any:
        """
        Get a property from an object, supporting nested properties

        Args:
            obj: Object to get property from
            property_path: Property path (e.g., "year" or "date.year")

        Returns:
            Property value or None
        """
        if "." in property_path:
            parts = property_path.split(".", 1)
            first_prop = parts[0]
            remaining = parts[1]
            intermediate = self._get_property(obj, first_prop)
            if intermediate is None:
                return None
            return self._get_property(intermediate, remaining)

        # Get the property
        if hasattr(obj, property_path):
            return getattr(obj, property_path)
        elif isinstance(obj, dict) and property_path in obj:
            return obj[property_path]
        else:
            logger.warning(f"Property {property_path} not found on {type(obj)}")
            return None

    def _resolve_from_source(self, source_spec: dict, input_name: str) -> Any:
        """
        Resolve value from source specification

        Args:
            source_spec: Source specification with regulation/output or url/ref
            input_name: Name of the input being resolved

        Returns:
            Resolved value from regulation call or external source
        """
        # Schema v0.2.0 format: regulation + output
        regulation = source_spec.get("regulation")
        output_name = source_spec.get("output")

        if output_name:
            if regulation:
                # Cross-law reference: build URI from regulation + output
                from engine.uri_resolver import RegelrechtURIBuilder

                uri = RegelrechtURIBuilder.build(regulation, output_name, output_name)
            else:
                # External data source (no regulation) - delegate to service provider
                logger.debug(
                    f"External data source for {input_name}: output={output_name}"
                )
                # For now, return None - service provider should handle this
                return None
        else:
            # Backward compatibility: article, url, ref
            article_ref = source_spec.get("article")
            uri = source_spec.get("url") or source_spec.get("ref") or article_ref

            if not uri:
                logger.warning(
                    f"No regulation/output or article/url/ref found in source spec for {input_name}"
                )
                return None

            # Convert article reference format to URI format
            # article: "law_id.endpoint" -> regelrecht://law_id/endpoint#input_name
            if (
                article_ref
                and not uri.startswith("#")
                and not uri.startswith("regelrecht://")
                and not uri.startswith("regulation/")
            ):
                # Parse article reference: "law_id.endpoint"
                if "." in article_ref:
                    law_id, endpoint = article_ref.rsplit(".", 1)
                    # Add input_name as field to extract from output
                    from engine.uri_resolver import RegelrechtURIBuilder

                    uri = RegelrechtURIBuilder.build(law_id, endpoint, input_name)
                else:
                    # Just an endpoint name, assume internal reference
                    uri = f"#{article_ref}"

        # Resolve parameter values ($BSN -> actual BSN value)
        params_spec = source_spec.get("parameters", {})
        resolved_params = {}
        for key, value in params_spec.items():
            if isinstance(value, str) and value.startswith("$"):
                resolved_params[key] = self._resolve_value(value[1:])
            else:
                resolved_params[key] = value

        # Handle internal references (same-law): #endpoint
        if uri.startswith("#"):
            endpoint = uri[1:]  # Remove the # prefix
            logger.debug(f"Resolving internal reference: #{endpoint}")

            # Create cache key for internal reference (use original uri which includes #)
            cache_key = self._make_cache_key(uri, resolved_params)

            # Check cache
            if cache_key in self._uri_cache:
                logger.debug(f"Cache hit for internal reference #{endpoint}")
                return self._uri_cache[cache_key]

            # Find the article by endpoint in current law
            article = self.current_law.find_article_by_endpoint(endpoint)
            if not article:
                logger.error(
                    f"Internal reference #{endpoint} not found in law {self.current_law.id}"
                )
                return None

            # Execute the article directly
            from engine.engine import ArticleEngine

            engine = ArticleEngine(article, self.current_law)
            result = engine.evaluate(
                parameters=resolved_params,
                service_provider=self.service_provider,
                calculation_date=self.calculation_date,
                requested_output=endpoint,
            )

            # Extract the endpoint output
            value = result.output.get(endpoint)

            # Cache result
            self._uri_cache[cache_key] = value
            logger.debug(f"Resolved internal reference #{endpoint} -> {value}")
            return value

        # Handle external references (cross-law URIs)
        # Create cache key
        cache_key = self._make_cache_key(uri, resolved_params)

        # Check cache
        if cache_key in self._uri_cache:
            logger.debug(f"Cache hit for {uri}")
            return self._uri_cache[cache_key]

        # Call service provider
        logger.debug(f"Resolving URI: {uri} with params {resolved_params}")
        result = self.service_provider.evaluate_uri(
            uri, resolved_params, self.calculation_date
        )

        # Extract field from URI
        from engine.uri_resolver import RegelrechtURI

        parsed_uri = RegelrechtURI(uri)
        if parsed_uri.field:
            value = result.output.get(parsed_uri.field)
        else:
            # No field specified, return first output or entire output dict
            if len(result.output) == 1:
                value = list(result.output.values())[0]
            else:
                value = result.output

        # Cache result
        self._uri_cache[cache_key] = value

        logger.debug(f"Resolved {uri} -> {value}")
        return value

    def _make_cache_key(self, uri: str, parameters: dict) -> str:
        """Create cache key for URI call"""
        param_str = ",".join(f"{k}:{v}" for k, v in sorted(parameters.items()))
        return f"{uri}({param_str},{self.calculation_date})"

    def set_output(self, name: str, value: Any):
        """Set an output value"""
        self.outputs[name] = value

    def get_output(self, name: str) -> Any:
        """Get an output value"""
        return self.outputs.get(name)

    def set_local(self, name: str, value: Any):
        """Set a local variable (for loops)"""
        self.local[name] = value

    def clear_local(self):
        """Clear local variables"""
        self.local = {}

    def create_child_context(self) -> "RuleContext":
        """Create a child context for nested evaluation (e.g., FOREACH)"""
        child = copy.copy(self)
        child.local = copy.copy(self.local)
        child.outputs = copy.copy(self.outputs)
        return child
