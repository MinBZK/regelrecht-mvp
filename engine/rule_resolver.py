"""
Rule Resolver - Law discovery and loading

Handles loading article-based laws from the regulation directory
and indexing them by $id and output names.
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
        self._output_index: dict[tuple[str, str], Article] = {}
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

        # Index outputs
        for output_name, article in law.get_all_outputs().items():
            key = (law.id, output_name)
            if key in self._output_index:
                logger.warning(
                    f"Duplicate output '{law.id}/{output_name}', overwriting"
                )
            self._output_index[key] = article

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

    def get_article_by_output(self, law_id: str, output_name: str) -> Optional[Article]:
        """
        Get article by law ID and output name

        Args:
            law_id: Law identifier
            output_name: Output name

        Returns:
            Article or None if not found
        """
        return self._output_index.get((law_id, output_name))

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

        article = law.find_article_by_output(parsed.output)
        if not article:
            logger.error(f"Output not found: {parsed.law_id}/{parsed.output}")
            return (None, None, None)

        return (law, article, parsed.field)

    def list_all_laws(self) -> list[str]:
        """Get list of all loaded law IDs"""
        return list(self._law_registry.keys())

    def list_all_outputs(self) -> list[tuple[str, str]]:
        """Get list of all (law_id, output_name) pairs"""
        return list(self._output_index.keys())

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

    def get_output_count(self) -> int:
        """Get number of indexed outputs"""
        return len(self._output_index)

    def find_delegated_regulation(
        self, law_id: str, article: str, criteria: list[dict]
    ) -> Optional[ArticleBasedLaw]:
        """
        Find a delegated regulation that matches the given criteria.

        Searches through all regulations that declare the specified law+article
        as their legal basis, and returns the first one that matches ALL criteria.

        Criteria are matched against properties on the law object (e.g., gemeente_code, jaar).

        Args:
            law_id: The delegating law ID (e.g., "test_delegation_law")
            article: The article number in the delegating law (e.g., "1")
            criteria: List of criteria dicts with 'name' and 'value' keys
                     e.g., [{"name": "gemeente_code", "value": "GM0384"}]

        Returns:
            Matching ArticleBasedLaw or None if no match found
        """
        basis_key = (law_id, article)
        candidates = self._legal_basis_index.get(basis_key, [])

        logger.debug(
            f"Finding delegated regulation for {law_id}.{article} with criteria {criteria}"
        )
        logger.debug(f"Found {len(candidates)} candidate regulations")

        for law in candidates:
            # Check if ALL criteria match
            all_match = True
            for criterion in criteria:
                crit_name: str | None = criterion.get("name")
                crit_value = criterion.get("value")

                # Skip invalid criteria without name
                if crit_name is None:
                    all_match = False
                    break

                # Get property from law object
                law_value = getattr(law, crit_name, None)

                logger.debug(
                    f"Checking {crit_name}: law has {law_value}, looking for {crit_value}"
                )

                if law_value != crit_value:
                    all_match = False
                    break

            if all_match:
                logger.debug(f"Found matching regulation: {law.id}")
                return law

        logger.debug("No matching regulation found")
        return None
