"""
URI Resolution for regelrecht:// URIs

Parses and resolves regelrecht:// URIs to law articles and fields.

URI Format: regelrecht://{law_id}/{endpoint}#{field}

Examples:
  - regelrecht://zvw/is_verzekerd#is_verzekerd
  - regelrecht://zorgtoeslagwet/bereken_zorgtoeslag#heeft_recht_op_zorgtoeslag
  - regelrecht://regeling_standaardpremie/standaardpremie#standaardpremie
"""

from dataclasses import dataclass


@dataclass
class RegelrechtURI:
    """Parsed regelrecht:// URI"""

    uri: str
    law_id: str
    endpoint: str
    field: str | None

    def __init__(self, uri: str):
        self.uri = uri
        self.law_id, self.endpoint, self.field = self._parse(uri)

    @staticmethod
    def _parse(uri: str) -> tuple[str, str, str | None]:
        """
        Parse regelrecht:// URI into components

        Args:
            uri: URI string like "regelrecht://law_id/endpoint#field"

        Returns:
            Tuple of (law_id, endpoint, field)

        Raises:
            ValueError: If URI format is invalid
        """
        if not uri.startswith("regelrecht://"):
            raise ValueError(
                f"Invalid regelrecht URI: must start with 'regelrecht://', got: {uri}"
            )

        # Remove scheme
        path = uri[len("regelrecht://") :]

        # Split on fragment (#)
        if "#" in path:
            path_part, field = path.split("#", 1)
        else:
            path_part = path
            field = None

        # Split path on first /
        if "/" not in path_part:
            raise ValueError(
                f"Invalid regelrecht URI: must contain law_id/endpoint, got: {uri}"
            )

        law_id, endpoint = path_part.split("/", 1)

        if not law_id or not endpoint:
            raise ValueError(
                f"Invalid regelrecht URI: law_id and endpoint cannot be empty, got: {uri}"
            )

        return law_id, endpoint, field

    def __str__(self) -> str:
        return self.uri

    def __repr__(self) -> str:
        return f"RegelrechtURI({self.uri!r})"

    def to_dict(self) -> dict:
        """Convert to dictionary representation"""
        return {
            "uri": self.uri,
            "law_id": self.law_id,
            "endpoint": self.endpoint,
            "field": self.field,
        }
