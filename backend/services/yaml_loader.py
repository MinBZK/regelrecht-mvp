"""
YAML Loader Service

Loads law/regulation YAML files from the regulation/ directory and
parses them into Pydantic models.
"""

import yaml
from pathlib import Path
from typing import Dict
from uuid import UUID

from backend.models.law import Law, LawSummary


class YAMLLoaderService:
    """Service for loading laws from YAML files"""

    def __init__(self, regulation_dir: Path):
        """
        Initialize the YAML loader.

        Args:
            regulation_dir: Path to the regulation directory (e.g., regulation/nl/)
        """
        self.regulation_dir = Path(regulation_dir)
        self._laws_cache: Dict[UUID, Law] = {}
        self._load_all_laws()

    def _load_all_laws(self) -> None:
        """Scan the regulation directory and load all YAML files"""
        # Pattern: regulation/nl/{layer}/{law_name}/{date}.yaml
        for yaml_file in self.regulation_dir.rglob("*.yaml"):
            try:
                law = self._load_law_from_file(yaml_file)
                self._laws_cache[law.uuid] = law
                print(f"✓ Loaded: {yaml_file.relative_to(self.regulation_dir)}")
            except Exception as e:
                print(f"✗ Failed to load {yaml_file}: {e}")

        print(f"\nTotal laws loaded: {len(self._laws_cache)}")

    def _load_law_from_file(self, filepath: Path) -> Law:
        """
        Load and validate a single law from YAML file.

        Args:
            filepath: Path to the YAML file

        Returns:
            Validated Law model

        Raises:
            ValidationError: If YAML doesn't match schema
        """
        with open(filepath, 'r', encoding='utf-8') as f:
            data = yaml.safe_load(f)

        # Pydantic will validate against the schema
        law = Law(**data)
        return law

    def get_all_laws(self) -> list[Law]:
        """
        Get all loaded laws.

        Returns:
            List of all laws
        """
        return list(self._laws_cache.values())

    def get_all_law_summaries(self) -> list[LawSummary]:
        """
        Get summaries of all loaded laws.

        Returns:
            List of law summaries
        """
        return [LawSummary.from_law(law) for law in self._laws_cache.values()]

    def get_law_by_uuid(self, uuid: UUID) -> Law | None:
        """
        Get a specific law by its UUID.

        Args:
            uuid: The law's UUID

        Returns:
            Law if found, None otherwise
        """
        return self._laws_cache.get(uuid)

    def get_law_by_bwb_id(self, bwb_id: str) -> Law | None:
        """
        Get a specific law by its BWB ID.

        Args:
            bwb_id: The BWB identification number

        Returns:
            Law if found, None otherwise
        """
        for law in self._laws_cache.values():
            if law.bwb_id == bwb_id:
                return law
        return None

    def reload_laws(self) -> None:
        """Reload all laws from the filesystem"""
        self._laws_cache.clear()
        self._load_all_laws()


# Global instance (will be initialized in main.py)
_yaml_loader: YAMLLoaderService | None = None


def init_yaml_loader(regulation_dir: Path) -> None:
    """Initialize the global YAML loader instance"""
    global _yaml_loader
    _yaml_loader = YAMLLoaderService(regulation_dir)


def get_yaml_loader() -> YAMLLoaderService:
    """
    Get the global YAML loader instance.

    Returns:
        The initialized YAMLLoaderService

    Raises:
        RuntimeError: If loader hasn't been initialized
    """
    if _yaml_loader is None:
        raise RuntimeError("YAML loader not initialized. Call init_yaml_loader() first.")
    return _yaml_loader
