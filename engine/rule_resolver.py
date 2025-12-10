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
from engine.logging_config import logger


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
        self._legal_basis_index: dict[
            tuple[str, str], list[ArticleBasedLaw]
        ] = {}  # (law_id, article) -> [regelingen]
        self._yaml_cache: dict[str, dict] = {}

        # Load all laws
        self._load_laws()

    def _load_laws(self):
        """Load all law files from regulation directory"""
        if not self.regulation_dir.exists():
            raise FileNotFoundError(
                f"Regulation directory not found: {self.regulation_dir}"
            )

        # Scan wet/, ministeriele_regeling/, and gemeentelijke_verordening/ directories
        for category in ["wet", "ministeriele_regeling", "gemeentelijke_verordening"]:
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
                    logger.warning(f"Failed to load {item}: {e}")

    def _load_law_file(self, file_path: Path):
        """Load a single law file"""
        # Load YAML
        yaml_data = self._load_yaml(file_path)

        # Parse as ArticleBasedLaw
        law = ArticleBasedLaw(yaml_data)

        # Register by $id
        if law.id in self._law_registry:
            logger.warning(f"Duplicate law ID '{law.id}', overwriting previous")

        self._law_registry[law.id] = law

        # Index endpoints
        for endpoint, article in law.get_all_endpoints().items():
            # Extract local endpoint name (after dot) for indexing
            # Endpoints in YAML are fully qualified like "law_id.endpoint_name"
            # but URIs use "law_id/endpoint_name" format
            local_endpoint = endpoint.split(".")[-1] if "." in endpoint else endpoint

            key = (law.id, local_endpoint)
            if key in self._endpoint_index:
                logger.warning(
                    f"Duplicate endpoint '{law.id}/{local_endpoint}', overwriting"
                )
            self._endpoint_index[key] = article

        # Index by legal_basis if present (all regulatory layers)
        if "legal_basis" in yaml_data:
            legal_basis_data = yaml_data["legal_basis"]

            # Support both single legal_basis (dict) and multiple legal_basis (list)
            legal_basis_list = []
            if isinstance(legal_basis_data, dict):
                # Single legal_basis - convert to list
                legal_basis_list = [legal_basis_data]
            elif isinstance(legal_basis_data, list):
                # Multiple legal_basis - use as is
                legal_basis_list = legal_basis_data

            # Index each legal_basis entry
            for legal_basis in legal_basis_list:
                if (
                    isinstance(legal_basis, dict)
                    and "law_id" in legal_basis
                    and "article" in legal_basis
                ):
                    basis_key = (legal_basis["law_id"], legal_basis["article"])
                    if basis_key not in self._legal_basis_index:
                        self._legal_basis_index[basis_key] = []
                    self._legal_basis_index[basis_key].append(law)

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
            logger.error(f"Invalid URI: {e}")
            return (None, None, None)

        law = self.get_law_by_id(parsed.law_id)
        if not law:
            logger.error(f"Law not found: {parsed.law_id}")
            return (None, None, None)

        article = law.find_article_by_endpoint(parsed.endpoint)
        if not article:
            logger.error(f"Endpoint not found: {parsed.law_id}/{parsed.endpoint}")
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

    def find_regelingen_by_legal_basis(
        self, law_id: str, article: str
    ) -> list[ArticleBasedLaw]:
        """
        Find ministeriele regelingen that declare a specific law article as their legal basis

        All law types are indexed, but only ministeriele regelingen
        (regulatory_layer == "MINISTERIELE_REGELING") are returned.

        Args:
            law_id: The law ID (e.g., "zorgtoeslagwet")
            article: The article number (e.g., "4")

        Returns:
            List of ministeriele regeling ArticleBasedLaw objects with this legal basis
        """
        basis_key = (law_id, article)
        all_laws = self._legal_basis_index.get(basis_key, [])
        # Filter to only return ministeriele regelingen
        return [
            law for law in all_laws if law.regulatory_layer == "MINISTERIELE_REGELING"
        ]

    def find_delegated_regulation(
        self, law_id: str, article: str, select_criteria: list[dict]
    ) -> Optional[ArticleBasedLaw]:
        """
        Find a regulation that implements delegated authority from a specific article.

        Uses a generic select_on mechanism to match regulations based on arbitrary
        criteria (e.g., gemeente_code, provincie_code, jaar).

        Args:
            law_id: The delegating law ID (e.g., "participatiewet")
            article: The delegating article number (e.g., "8")
            select_criteria: List of {name: str, value: Any} criteria to match

        Returns:
            The matching ArticleBasedLaw or None if not found
        """
        basis_key = (law_id, article)
        candidates = self._legal_basis_index.get(basis_key, [])

        for law in candidates:
            if self._matches_all_criteria(law, select_criteria):
                return law

        return None

    def _matches_all_criteria(self, law: ArticleBasedLaw, criteria: list[dict]) -> bool:
        """
        Check if a law matches all selection criteria.

        Args:
            law: Law to check
            criteria: List of {name: str, value: Any} criteria

        Returns:
            True if law matches all criteria
        """
        for criterion in criteria:
            prop_name = criterion["name"]
            expected = criterion["value"]
            actual = getattr(law, prop_name, None)
            if actual != expected:
                return False
        return True

    def find_gemeentelijke_verordening(
        self, law_id: str, article: str, gemeente_code: str
    ) -> Optional[ArticleBasedLaw]:
        """
        Find a gemeentelijke verordening that:
        - Has the specified law article as legal basis
        - Belongs to the specified gemeente

        This is used for resolving delegated legislation, where a national law
        (like Participatiewet art. 8) delegates authority to municipalities
        to create their own regulations.

        DEPRECATED: Use find_delegated_regulation() with select_on criteria instead.

        Args:
            law_id: The delegating law ID (e.g., "participatiewet")
            article: The delegating article number (e.g., "8")
            gemeente_code: Municipality code (e.g., "GM0384" for Diemen)

        Returns:
            The matching ArticleBasedLaw or None if not found
        """
        # Delegate to generic method with gemeente_code criterion
        return self.find_delegated_regulation(
            law_id, article, [{"name": "gemeente_code", "value": gemeente_code}]
        )

    def find_all_gemeentelijke_verordeningen(
        self, law_id: str, article: str
    ) -> list[ArticleBasedLaw]:
        """
        Find all gemeentelijke verordeningen that declare a specific law article
        as their legal basis.

        Args:
            law_id: The delegating law ID (e.g., "participatiewet")
            article: The delegating article number (e.g., "8")

        Returns:
            List of gemeentelijke verordening ArticleBasedLaw objects
        """
        basis_key = (law_id, article)
        all_laws = self._legal_basis_index.get(basis_key, [])
        return [
            law
            for law in all_laws
            if law.regulatory_layer == "GEMEENTELIJKE_VERORDENING"
        ]

    def get_endpoint_count(self) -> int:
        """Get number of indexed endpoints"""
        return len(self._endpoint_index)
