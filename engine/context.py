"""
Execution context for article evaluation

Manages state and value resolution during article execution.
"""

from dataclasses import dataclass, field
from typing import Any, Optional
import copy

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
        input_specs: list[dict] = None,
        output_specs: list[dict] = None,
        current_law: Any = None,
    ):
        """
        Initialize execution context

        Args:
            definitions: Article-level definitions
            parameters: Input parameters (e.g., {"BSN": "123456789"})
            service_provider: Service for resolving URIs
            calculation_date: Reference date for calculations
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
        1. Local scope (loop variables)
        2. Outputs (calculated values)
        3. Resolved inputs
        4. Definitions (constants)
        5. Parameters (direct inputs)
        6. Input with source.url (cross-law reference)

        Args:
            path: Variable name

        Returns:
            Resolved value or None
        """
        # 1. Local scope (FOREACH loop variables)
        if path in self.local:
            return self.local[path]

        # 2. Outputs (calculated values)
        if path in self.outputs:
            return self.outputs[path]

        # 3. Resolved inputs (already fetched)
        if path in self.resolved_inputs:
            return self.resolved_inputs[path]

        # 4. Definitions (constants)
        if path in self.definitions:
            return self.definitions[path]

        # 5. Parameters (direct inputs)
        if path in self.parameters:
            return self.parameters[path]

        # 6. Input with source - need to resolve
        input_spec = self._find_input_spec(path)
        if input_spec and "source" in input_spec:
            value = self._resolve_from_source(input_spec["source"], path)
            self.resolved_inputs[path] = value
            return value

        logger.warning(f"Could not resolve variable: {path}")
        return None

    def _resolve_from_source(self, source_spec: dict, input_name: str) -> Any:
        """
        Resolve value from source.url or source.ref

        Args:
            source_spec: Source specification with url/ref and optional parameters
            input_name: Name of the input being resolved

        Returns:
            Resolved value from URI/ref call
        """
        # Support both 'url' and 'ref' for backward compatibility
        uri = source_spec.get("url") or source_spec.get("ref")

        if not uri:
            logger.warning(f"No url or ref found in source spec for {input_name}")
            return None

        # Handle internal references (same-file): #output_name
        if uri.startswith("#"):
            output_name = uri[1:]  # Remove the # prefix
            logger.debug(f"Resolving internal reference: {output_name}")

            # Build full URI to current law's article that produces this output
            # We need to find which article in the current law produces this output
            full_uri = f"regulation/nl/{self.current_law.regulatory_layer.lower()}/{self.current_law.id}{uri}"
            uri = full_uri

        params_spec = source_spec.get("parameters", {})

        # Resolve parameter values ($BSN -> actual BSN value)
        resolved_params = {}
        for key, value in params_spec.items():
            if isinstance(value, str) and value.startswith("$"):
                resolved_params[key] = self._resolve_value(value[1:])
            else:
                resolved_params[key] = value

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
