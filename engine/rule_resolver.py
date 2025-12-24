"""
Rule Resolver - Law discovery and loading

Handles loading article-based laws from the regulation directory
and indexing them by $id and output names.

Supports multiple versions of the same law (same $id, different valid_from dates).
When resolving, selects the version where valid_from <= reference_date.
"""

from datetime import datetime
from pathlib import Path
from typing import Optional
import yaml

from engine.article_loader import ArticleBasedLaw, Article
from engine.uri_resolver import RegelrechtURI
from engine.logging_config import logger


class RuleResolver:
    """Resolves and loads article-based laws with date-based version selection"""

    def __init__(self, regulation_dir: str):
        """
        Initialize rule resolver

        Args:
            regulation_dir: Path to regulation directory (e.g., "regulation/nl")
        """
        self.regulation_dir = Path(regulation_dir)
        # Store ALL versions of each law, keyed by $id
        self._law_versions: dict[str, list[ArticleBasedLaw]] = {}
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

        # Add to versions list (multiple versions of same law allowed)
        if law.id not in self._law_versions:
            self._law_versions[law.id] = []
        self._law_versions[law.id].append(law)

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

    def _select_version_for_date(
        self, versions: list[ArticleBasedLaw], reference_date: datetime | None
    ) -> ArticleBasedLaw | None:
        """
        Select the appropriate law version for a given reference date.

        Selects the version where valid_from <= reference_date,
        preferring the most recent valid_from date.

        Args:
            versions: List of law versions (same $id, different valid_from)
            reference_date: The date for which to select the version

        Returns:
            The appropriate ArticleBasedLaw version, or None if no valid version
        """
        if not versions:
            return None

        # If no reference date, return the most recent version
        if reference_date is None:
            # Sort by valid_from descending, return first
            sorted_versions = sorted(
                versions,
                key=lambda v: v.valid_from or "0000-00-00",
                reverse=True,
            )
            return sorted_versions[0]

        # Convert reference_date to string for comparison (YYYY-MM-DD format)
        ref_date_str = reference_date.strftime("%Y-%m-%d")

        # Filter versions where valid_from <= reference_date
        valid_versions = [
            v for v in versions if v.valid_from is None or v.valid_from <= ref_date_str
        ]

        if not valid_versions:
            # No version valid for this date - return None or fall back to earliest?
            logger.warning(
                f"No version valid for date {ref_date_str}, "
                f"available: {[v.valid_from for v in versions]}"
            )
            return None

        # Sort by valid_from descending to get the most recent valid version
        sorted_versions = sorted(
            valid_versions,
            key=lambda v: v.valid_from or "0000-00-00",
            reverse=True,
        )
        return sorted_versions[0]

    def get_law_by_id(
        self, law_id: str, reference_date: datetime | None = None
    ) -> Optional[ArticleBasedLaw]:
        """
        Get law by $id slug, selecting the appropriate version for the reference date.

        Args:
            law_id: Law identifier (e.g., "zorgtoeslagwet")
            reference_date: Date for version selection (uses most recent if None)

        Returns:
            ArticleBasedLaw or None if not found
        """
        versions = self._law_versions.get(law_id)
        if not versions:
            return None
        return self._select_version_for_date(versions, reference_date)

    def get_article_by_output(
        self, law_id: str, output_name: str, reference_date: datetime | None = None
    ) -> Optional[Article]:
        """
        Get article by law ID and output name

        Args:
            law_id: Law identifier
            output_name: Output name
            reference_date: Date for version selection

        Returns:
            Article or None if not found
        """
        law = self.get_law_by_id(law_id, reference_date)
        if not law:
            return None
        return law.find_article_by_output(output_name)

    def resolve_uri(
        self, uri: str, reference_date: datetime | None = None
    ) -> tuple[Optional[ArticleBasedLaw], Optional[Article], Optional[str]]:
        """
        Resolve regelrecht:// URI to law, article, and field

        Args:
            uri: regelrecht:// URI string
            reference_date: Date for version selection

        Returns:
            Tuple of (law, article, field) or (None, None, None) if not found
        """
        try:
            parsed = RegelrechtURI(uri)
        except ValueError as e:
            logger.error(f"Invalid URI: {e}")
            return (None, None, None)

        law = self.get_law_by_id(parsed.law_id, reference_date)
        if not law:
            logger.error(f"Law not found: {parsed.law_id}")
            return (None, None, None)

        article = law.find_article_by_output(parsed.output)
        if not article:
            logger.error(f"Output not found: {parsed.law_id}/{parsed.output}")
            return (None, None, None)

        return (law, article, parsed.field)

    def list_all_laws(self) -> list[str]:
        """Get list of all loaded law IDs (unique, regardless of versions)"""
        return list(self._law_versions.keys())

    def list_all_outputs(
        self, reference_date: datetime | None = None
    ) -> list[tuple[str, str]]:
        """
        Get list of all (law_id, output_name) pairs for the given reference date.

        Args:
            reference_date: Date for version selection

        Returns:
            List of (law_id, output_name) tuples
        """
        outputs = []
        for law_id in self._law_versions:
            law = self.get_law_by_id(law_id, reference_date)
            if law:
                for output_name in law.get_all_outputs().keys():
                    outputs.append((law_id, output_name))
        return outputs

    def get_law_count(self) -> int:
        """Get number of unique law IDs (not counting versions)"""
        return len(self._law_versions)

    def get_version_count(self) -> int:
        """Get total number of law versions loaded"""
        return sum(len(versions) for versions in self._law_versions.values())

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

    def get_output_count(self, reference_date: datetime | None = None) -> int:
        """Get number of outputs for the given reference date"""
        return len(self.list_all_outputs(reference_date))

    def find_delegated_regulation(
        self,
        law_id: str,
        article: str,
        criteria: list[dict],
        reference_date: datetime | None = None,
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
            reference_date: Date for version selection

        Returns:
            Matching ArticleBasedLaw or None if no match found
        """
        basis_key = (law_id, article)
        all_candidates = self._legal_basis_index.get(basis_key, [])

        # Filter candidates by reference_date if provided
        if reference_date:
            ref_date_str = reference_date.strftime("%Y-%m-%d")
            candidates = [
                c
                for c in all_candidates
                if c.valid_from is None or c.valid_from <= ref_date_str
            ]
        else:
            candidates = all_candidates

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

                # Invalid criteria without name - log warning and reject match
                if crit_name is None:
                    logger.warning(
                        f"Invalid criterion missing 'name' in criteria: {criterion}. "
                        f"Full criteria list: {criteria}"
                    )
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
