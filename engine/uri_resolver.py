"""
URI Resolution for regelrecht:// URIs and file path references

Parses and constructs references to law articles and fields.

Supported formats:
1. regelrecht:// URI: regelrecht://{law_id}/{output}#{field}
2. File path reference: regulation/nl/{layer}/{law_id}#{field}

Examples:
  - regelrecht://zvw/is_verzekerd#is_verzekerd
  - regulation/nl/wet/zvw#is_verzekerd
  - regulation/nl/ministeriele_regeling/regeling_standaardpremie#standaardpremie
"""

from dataclasses import dataclass


class RegelrechtURIBuilder:
    """Builder for constructing regelrecht:// URIs in a type-safe way"""

    @staticmethod
    def build(law_id: str, output: str, field: str | None = None) -> str:
        """
        Build a regelrecht:// URI from components

        This is the counterpart to RegelrechtURI parser - constructs URIs
        in a type-safe way instead of using f-strings.

        Args:
            law_id: Law identifier (e.g., "zorgtoeslagwet")
            output: Output name (e.g., "bereken_zorgtoeslag")
            field: Optional field name for fragment (e.g., "heeft_recht_op_zorgtoeslag")

        Returns:
            Formatted regelrecht:// URI string

        Examples:
            >>> RegelrechtURIBuilder.build("zorgtoeslagwet", "bereken_zorgtoeslag")
            'regelrecht://zorgtoeslagwet/bereken_zorgtoeslag'
            >>> RegelrechtURIBuilder.build("zvw", "is_verzekerd", "is_verzekerd")
            'regelrecht://zvw/is_verzekerd#is_verzekerd'
        """
        uri = f"regelrecht://{law_id}/{output}"
        if field:
            uri += f"#{field}"
        return uri


@dataclass
class RegelrechtURI:
    """Parsed regelrecht:// URI"""

    uri: str
    law_id: str
    output: str
    field: str | None

    def __init__(self, uri: str):
        self.uri = uri
        self.law_id, self.output, self.field = self._parse(uri)

    @staticmethod
    def _parse(uri: str) -> tuple[str, str, str | None]:
        """
        Parse URI into components

        Supports two formats:
        1. regelrecht://law_id/output#field
        2. regulation/nl/layer/law_id#field

        Args:
            uri: URI string

        Returns:
            Tuple of (law_id, output, field)
            For file paths without explicit output, output is the field name

        Raises:
            ValueError: If URI format is invalid
        """
        # Split on fragment (#) first
        if "#" in uri:
            path_part, field = uri.split("#", 1)
        else:
            path_part = uri
            field = None

        # Check if it's a regelrecht:// URI
        if path_part.startswith("regelrecht://"):
            # Remove scheme
            path = path_part[len("regelrecht://") :]

            # Split path on first /
            if "/" not in path:
                raise ValueError(
                    f"Invalid regelrecht URI: must contain law_id/output, got: {uri}"
                )

            law_id, output = path.split("/", 1)

            if not law_id or not output:
                raise ValueError(
                    f"Invalid regelrecht URI: law_id and output cannot be empty, got: {uri}"
                )

            return law_id, output, field

        # Otherwise, treat as file path: regulation/nl/layer/law_id
        elif path_part.startswith("regulation/nl/"):
            # Parse path parts
            parts = path_part.split("/")
            if len(parts) < 4:
                raise ValueError(
                    f"Invalid file path reference: expected regulation/nl/layer/law_id, got: {uri}"
                )

            # Extract law_id (last part of path)
            law_id = parts[-1]

            # For file path references, the output is the field name
            # (we look up the article that produces this output)
            output = field if field else law_id

            return law_id, output, field

        else:
            raise ValueError(
                f"Invalid URI format: must be regelrecht:// or regulation/nl/..., got: {uri}"
            )

    def __str__(self) -> str:
        return self.uri

    def __repr__(self) -> str:
        return f"RegelrechtURI({self.uri!r})"

    def to_dict(self) -> dict:
        """Convert to dictionary representation"""
        return {
            "uri": self.uri,
            "law_id": self.law_id,
            "output": self.output,
            "field": self.field,
        }
