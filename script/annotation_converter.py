"""
Annotation Converter - W3C Annotation to Schema Conversion

Converts W3C Web Annotations to machine-readable law schema structures.

Mapping:
- classification: definition → definitions.{NAME}
- classification: input → execution.input[]
- classification: output → execution.output[]
- classification: open_norm → open_norms[]
- classification: parameter → execution.parameters[]
"""

from dataclasses import dataclass
from datetime import datetime
from typing import Any


@dataclass
class ConversionResult:
    """Result of converting an annotation to schema format"""

    success: bool
    target_path: str  # e.g., "definitions.gezamenlijke_huishouding"
    schema_data: dict[str, Any] | None
    error: str | None = None


class AnnotationConverter:
    """
    Converts W3C annotations to law schema structures.

    Handles the mapping between annotation classifications and
    the appropriate schema locations.
    """

    # Valid classification types
    VALID_CLASSIFICATIONS = {"definition", "input", "output", "open_norm", "parameter"}

    # Valid data types in schema
    VALID_DATA_TYPES = {
        "boolean",
        "integer",
        "number",
        "string",
        "date",
        "amount",
        "object",
        "array",
    }

    def convert(self, annotation: dict) -> ConversionResult:
        """
        Convert a W3C annotation to schema format.

        Args:
            annotation: W3C annotation dict with body.classification or body.type

        Returns:
            ConversionResult with target path and schema data
        """
        body = annotation.get("body", {})
        body_type = body.get("type")

        # Handle linking annotations (SpecificResource type)
        if body_type == "SpecificResource":
            return self._convert_link(annotation)

        classification = body.get("classification")

        if not classification:
            return ConversionResult(
                success=False,
                target_path="",
                schema_data=None,
                error="Missing classification in annotation body",
            )

        if classification not in self.VALID_CLASSIFICATIONS:
            return ConversionResult(
                success=False,
                target_path="",
                schema_data=None,
                error=f"Invalid classification: {classification}",
            )

        # Dispatch to appropriate converter
        converters = {
            "definition": self._convert_definition,
            "input": self._convert_input,
            "output": self._convert_output,
            "open_norm": self._convert_open_norm,
            "parameter": self._convert_parameter,
        }

        return converters[classification](annotation)

    def _convert_definition(self, annotation: dict) -> ConversionResult:
        """Convert definition annotation to definitions.{NAME} structure."""
        body = annotation.get("body", {})
        target = annotation.get("target", {})
        selector = target.get("selector", {})

        # Get name from body or derive from exact text
        name = body.get("name")
        if not name:
            exact = selector.get("exact", "")
            name = self._text_to_variable_name(exact)

        if not name:
            return ConversionResult(
                success=False,
                target_path="",
                schema_data=None,
                error="Cannot determine definition name",
            )

        # Build definition structure
        definition_data: dict[str, Any] = {}

        if body.get("value") is not None:
            definition_data["value"] = body["value"]

        if body.get("description"):
            definition_data["description"] = body["description"]

        # If no value, use the exact text as description
        if not definition_data:
            definition_data["description"] = selector.get("exact", name)

        return ConversionResult(
            success=True,
            target_path=f"definitions.{name}",
            schema_data=definition_data,
        )

    def _convert_input(self, annotation: dict) -> ConversionResult:
        """Convert input annotation to execution.input[] structure."""
        body = annotation.get("body", {})
        target = annotation.get("target", {})
        selector = target.get("selector", {})

        name = body.get("name")
        if not name:
            exact = selector.get("exact", "")
            name = self._text_to_variable_name(exact)

        if not name:
            return ConversionResult(
                success=False,
                target_path="",
                schema_data=None,
                error="Cannot determine input name",
            )

        data_type = body.get("data_type", "boolean")
        if data_type not in self.VALID_DATA_TYPES:
            data_type = "boolean"

        input_data: dict[str, Any] = {
            "name": name,
            "type": data_type,
        }

        if body.get("description"):
            input_data["description"] = body["description"]

        # Handle source reference
        source = body.get("source", {})
        if source:
            input_data["source"] = {}
            if source.get("regulation"):
                input_data["source"]["regulation"] = source["regulation"]
            if source.get("output"):
                input_data["source"]["output"] = source["output"]
            if source.get("human_input"):
                input_data["source"]["human_input"] = True
            if source.get("parameters"):
                input_data["source"]["parameters"] = source["parameters"]

        return ConversionResult(
            success=True,
            target_path="execution.input",
            schema_data=input_data,
        )

    def _convert_output(self, annotation: dict) -> ConversionResult:
        """Convert output annotation to execution.output[] structure."""
        body = annotation.get("body", {})
        target = annotation.get("target", {})
        selector = target.get("selector", {})

        name = body.get("name")
        if not name:
            exact = selector.get("exact", "")
            name = self._text_to_variable_name(exact)

        if not name:
            return ConversionResult(
                success=False,
                target_path="",
                schema_data=None,
                error="Cannot determine output name",
            )

        data_type = body.get("data_type", "boolean")
        if data_type not in self.VALID_DATA_TYPES:
            data_type = "boolean"

        output_data: dict[str, Any] = {
            "name": name,
            "type": data_type,
        }

        if body.get("description"):
            output_data["description"] = body["description"]

        return ConversionResult(
            success=True,
            target_path="execution.output",
            schema_data=output_data,
        )

    def _convert_open_norm(self, annotation: dict) -> ConversionResult:
        """Convert open_norm annotation to open_norms[] structure."""
        body = annotation.get("body", {})
        target = annotation.get("target", {})
        selector = target.get("selector", {})

        # For open norms, the term is the exact selected text
        term = selector.get("exact", "")
        if not term:
            return ConversionResult(
                success=False,
                target_path="",
                schema_data=None,
                error="Cannot determine open norm term (missing selector.exact)",
            )

        norm_data: dict[str, Any] = {
            "term": term,
        }

        description = body.get("description") or body.get("value")
        if description:
            norm_data["description"] = description
        else:
            norm_data["description"] = f"Open norm: {term}"

        return ConversionResult(
            success=True,
            target_path="open_norms",
            schema_data=norm_data,
        )

    def _convert_parameter(self, annotation: dict) -> ConversionResult:
        """Convert parameter annotation to execution.parameters[] structure."""
        body = annotation.get("body", {})
        target = annotation.get("target", {})
        selector = target.get("selector", {})

        name = body.get("name")
        if not name:
            exact = selector.get("exact", "")
            name = self._text_to_variable_name(exact)

        if not name:
            return ConversionResult(
                success=False,
                target_path="",
                schema_data=None,
                error="Cannot determine parameter name",
            )

        data_type = body.get("data_type", "string")
        if data_type not in self.VALID_DATA_TYPES:
            data_type = "string"

        param_data: dict[str, Any] = {
            "name": name,
            "type": data_type,
        }

        if body.get("required"):
            param_data["required"] = True

        if body.get("description"):
            param_data["description"] = body["description"]

        return ConversionResult(
            success=True,
            target_path="execution.parameters",
            schema_data=param_data,
        )

    def _convert_link(self, annotation: dict) -> ConversionResult:
        """Convert linking annotation to references[] structure."""
        body = annotation.get("body", {})
        target = annotation.get("target", {})
        selector = target.get("selector", {})

        source = body.get("source", "")
        if not source:
            return ConversionResult(
                success=False,
                target_path="",
                schema_data=None,
                error="Missing source in linking annotation",
            )

        exact_text = selector.get("exact", "")

        link_data: dict[str, Any] = {
            "source": source,
            "text": exact_text,
        }

        if body.get("description"):
            link_data["description"] = body["description"]

        return ConversionResult(
            success=True,
            target_path="references",
            schema_data=link_data,
        )

    def _text_to_variable_name(self, text: str) -> str:
        """
        Convert human-readable text to a valid variable name.

        Examples:
            "gezamenlijke huishouding" → "gezamenlijke_huishouding"
            "Is verzekerd" → "is_verzekerd"
        """
        if not text:
            return ""

        # Lowercase and replace spaces with underscores
        name = text.lower().strip()
        name = name.replace(" ", "_")

        # Remove non-alphanumeric characters except underscores
        import re

        name = re.sub(r"[^a-z0-9_]", "", name)

        # Remove leading underscores and numbers
        name = re.sub(r"^[_0-9]+", "", name)

        # Collapse multiple underscores
        name = re.sub(r"_+", "_", name)

        return name.strip("_")


def apply_conversion_to_article(
    article_data: dict, conversion: ConversionResult
) -> dict:
    """
    Apply a conversion result to an article's machine_readable section.

    Args:
        article_data: The article dict from the law YAML
        conversion: The ConversionResult from AnnotationConverter

    Returns:
        Updated article_data dict
    """
    if not conversion.success or not conversion.schema_data:
        return article_data

    # Ensure machine_readable exists
    if "machine_readable" not in article_data:
        article_data["machine_readable"] = {}

    mr = article_data["machine_readable"]
    path = conversion.target_path

    if path.startswith("definitions."):
        # definitions.{name}
        if "definitions" not in mr:
            mr["definitions"] = {}
        name = path.split(".", 1)[1]
        mr["definitions"][name] = conversion.schema_data

    elif path == "execution.input":
        if "execution" not in mr:
            mr["execution"] = {}
        if "input" not in mr["execution"]:
            mr["execution"]["input"] = []
        # Check for duplicate
        name = conversion.schema_data.get("name")
        existing = [i for i in mr["execution"]["input"] if i.get("name") == name]
        if not existing:
            mr["execution"]["input"].append(conversion.schema_data)

    elif path == "execution.output":
        if "execution" not in mr:
            mr["execution"] = {}
        if "output" not in mr["execution"]:
            mr["execution"]["output"] = []
        name = conversion.schema_data.get("name")
        existing = [o for o in mr["execution"]["output"] if o.get("name") == name]
        if not existing:
            mr["execution"]["output"].append(conversion.schema_data)

    elif path == "execution.parameters":
        if "execution" not in mr:
            mr["execution"] = {}
        if "parameters" not in mr["execution"]:
            mr["execution"]["parameters"] = []
        name = conversion.schema_data.get("name")
        existing = [p for p in mr["execution"]["parameters"] if p.get("name") == name]
        if not existing:
            mr["execution"]["parameters"].append(conversion.schema_data)

    elif path == "open_norms":
        if "open_norms" not in mr:
            mr["open_norms"] = []
        term = conversion.schema_data.get("term")
        existing = [n for n in mr["open_norms"] if n.get("term") == term]
        if not existing:
            mr["open_norms"].append(conversion.schema_data)

    elif path == "references":
        if "references" not in mr:
            mr["references"] = []
        source = conversion.schema_data.get("source")
        text = conversion.schema_data.get("text")
        existing = [
            r for r in mr["references"] if r.get("source") == source and r.get("text") == text
        ]
        if not existing:
            mr["references"].append(conversion.schema_data)

    return article_data


def mark_annotation_promoted(annotation: dict) -> dict:
    """
    Mark an annotation as promoted with timestamp.

    Args:
        annotation: The annotation dict to update

    Returns:
        Updated annotation with status=promoted and promoted_at timestamp
    """
    annotation["status"] = "promoted"
    annotation["promoted_at"] = datetime.now().isoformat()
    return annotation
