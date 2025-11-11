"""
Rule Resolver - Law discovery and loading

Handles loading article-based laws from the regulation directory
and indexing them by $id and endpoint.
"""

from pathlib import Path
from typing import Optional
import yaml

from engine.article_loader import ArticleBasedLaw, Article
from engine.uri_resolver import RegelrechtURI


class RuleResolver:
    """Resolves and loads article-based laws"""

    def __init__(self, regulation_dir: str):
        """
        Initialize rule resolver

        Args:
            regulation_dir: Path to regulation directory (e.g., "regulation/nl")
        """
        self.regulation_dir = Path(regulation_dir)
        self._law_registry: dict[str, ArticleBasedLaw] = {}
        self._endpoint_index: dict[tuple[str, str], Article] = {}
        self._grondslag_index: dict[tuple[str, str], list[str]] = {}  # (law_id, article) -> [regeling_ids]
        self._yaml_cache: dict[str, dict] = {}

        # Load all laws
        self._load_laws()

    def _load_laws(self):
        """Load all law files from regulation directory"""
        if not self.regulation_dir.exists():
            raise FileNotFoundError(
                f"Regulation directory not found: {self.regulation_dir}"
            )

        # Scan wet/ and ministeriele_regeling/ directories
        for category in ["wet", "ministeriele_regeling"]:
            category_dir = self.regulation_dir / category
            if category_dir.exists():
                self._load_laws_from_directory(category_dir)

    def _load_laws_from_directory(self, directory: Path):
        """Recursively load laws from a directory"""
        for item in directory.iterdir():
            if item.is_dir():
                self._load_laws_from_directory(item)
            elif item.suffix == ".yaml" and item.name.endswith(".yaml"):
                try:
                    self._load_law_file(item)
                except Exception as e:
                    print(f"Warning: Failed to load {item}: {e}")

    def _load_law_file(self, file_path: Path):
        """Load a single law file"""
        # Load YAML
        yaml_data = self._load_yaml(file_path)

        # Parse as ArticleBasedLaw
        law = ArticleBasedLaw(yaml_data)

        # Register by $id
        if law.id in self._law_registry:
            print(f"Warning: Duplicate law ID '{law.id}', overwriting previous")

        self._law_registry[law.id] = law

        # Index endpoints
        for endpoint, article in law.get_all_endpoints().items():
            # Extract local endpoint name (after dot) for indexing
            # Endpoints in YAML are fully qualified like "law_id.endpoint_name"
            # but URIs use "law_id/endpoint_name" format
            local_endpoint = endpoint.split(".")[-1] if "." in endpoint else endpoint

            key = (law.id, local_endpoint)
            if key in self._endpoint_index:
                print(
                    f"Warning: Duplicate endpoint '{law.id}/{local_endpoint}', overwriting"
                )
            self._endpoint_index[key] = article

        # Index by grondslag if present
        if "grondslag" in yaml_data:
            grondslag_data = yaml_data["grondslag"]

            # Support both single grondslag (dict) and multiple grondslag (list)
            grondslag_list = []
            if isinstance(grondslag_data, dict):
                # Single grondslag - convert to list
                grondslag_list = [grondslag_data]
            elif isinstance(grondslag_data, list):
                # Multiple grondslag - use as is
                grondslag_list = grondslag_data

            # Index each grondslag entry
            for grondslag in grondslag_list:
                if isinstance(grondslag, dict) and "law_id" in grondslag and "article" in grondslag:
                    grondslag_key = (grondslag["law_id"], grondslag["article"])
                    if grondslag_key not in self._grondslag_index:
                        self._grondslag_index[grondslag_key] = []
                    self._grondslag_index[grondslag_key].append(law.id)

    def _load_yaml(self, file_path: Path) -> dict:
        """Load YAML file with caching"""
        file_key = str(file_path)

        if file_key not in self._yaml_cache:
            with open(file_path, "r", encoding="utf-8") as f:
                self._yaml_cache[file_key] = yaml.safe_load(f)

        return self._yaml_cache[file_key]

    def get_law_by_id(self, law_id: str) -> Optional[ArticleBasedLaw]:
        """
        Get law by $id slug

        Args:
            law_id: Law identifier (e.g., "zorgtoeslagwet")

        Returns:
            ArticleBasedLaw or None if not found
        """
        return self._law_registry.get(law_id)

    def get_article_by_endpoint(self, law_id: str, endpoint: str) -> Optional[Article]:
        """
        Get article by law ID and endpoint

        Args:
            law_id: Law identifier
            endpoint: Endpoint name

        Returns:
            Article or None if not found
        """
        return self._endpoint_index.get((law_id, endpoint))

    def resolve_uri(
        self, uri: str
    ) -> tuple[Optional[ArticleBasedLaw], Optional[Article], Optional[str]]:
        """
        Resolve regelrecht:// URI to law, article, and field

        Args:
            uri: regelrecht:// URI string

        Returns:
            Tuple of (law, article, field) or (None, None, None) if not found
        """
        try:
            parsed = RegelrechtURI(uri)
        except ValueError as e:
            print(f"Invalid URI: {e}")
            return (None, None, None)

        law = self.get_law_by_id(parsed.law_id)
        if not law:
            print(f"Law not found: {parsed.law_id}")
            return (None, None, None)

        article = law.find_article_by_endpoint(parsed.endpoint)
        if not article:
            print(f"Endpoint not found: {parsed.law_id}/{parsed.endpoint}")
            return (None, None, None)

        return (law, article, parsed.field)

    def list_all_laws(self) -> list[str]:
        """Get list of all loaded law IDs"""
        return list(self._law_registry.keys())

    def list_all_endpoints(self) -> list[tuple[str, str]]:
        """Get list of all (law_id, endpoint) pairs"""
        return list(self._endpoint_index.keys())

    def get_law_count(self) -> int:
        """Get number of loaded laws"""
        return len(self._law_registry)

    def find_regelingen_by_grondslag(self, law_id: str, article: str) -> list[str]:
        """
        Find all ministeriele regelingen based on a specific law article

        Args:
            law_id: The law ID (e.g., "zorgtoeslagwet")
            article: The article number (e.g., "4")

        Returns:
            List of law IDs that have this law+article as grondslag
        """
        grondslag_key = (law_id, article)
        return self._grondslag_index.get(grondslag_key, [])

    def get_endpoint_count(self) -> int:
        """Get number of indexed endpoints"""
        return len(self._endpoint_index)
