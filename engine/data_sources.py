"""
Data Source Resolution System

Provides automatic data resolution from external sources based on field names.
Laws declare WHAT they need, the engine searches WHERE to find it.
"""

from dataclasses import dataclass, field
from typing import Any, Protocol, runtime_checkable

from engine.logging_config import logger


@dataclass
class DataSourceMatch:
    """Result of a successful data source query"""

    value: Any
    source_name: str
    source_type: str


@runtime_checkable
class DataSource(Protocol):
    """Protocol for data source implementations"""

    @property
    def name(self) -> str:
        """Unique identifier for this source"""
        ...

    @property
    def priority(self) -> int:
        """Priority for disambiguation (higher = preferred)"""
        ...

    def has_field(self, field: str) -> bool:
        """Check if this source can provide the given field"""
        ...

    def get(self, field: str, criteria: dict[str, Any]) -> Any | None:
        """
        Retrieve value matching field and selection criteria

        Args:
            field: Field name to retrieve (lowercase)
            criteria: Selection criteria (e.g., {"bsn": "123456789"})

        Returns:
            Value if found, None otherwise
        """
        ...


@dataclass
class DictDataSource:
    """
    Dict-based data source with key-based lookup.

    Stores records indexed by a primary key (typically BSN).
    Fields are matched case-insensitively.
    """

    name: str
    priority: int = 100
    _data: dict[str, dict[str, Any]] = field(default_factory=dict)
    _field_index: set[str] = field(default_factory=set)

    def store(self, key: str, record: dict[str, Any]) -> None:
        """
        Store a record indexed by key

        Args:
            key: Primary key (e.g., BSN)
            record: Data record with field names and values
        """
        # Normalize field names to lowercase
        normalized = {k.lower(): v for k, v in record.items()}
        self._data[str(key)] = normalized
        self._field_index.update(normalized.keys())

    def has_field(self, field: str) -> bool:
        """Check if any stored record has the given field"""
        return field.lower() in self._field_index

    def get(self, field: str, criteria: dict[str, Any]) -> Any | None:
        """
        Retrieve value by field name and selection criteria

        Uses first criterion value as the primary key lookup.
        """
        if not criteria:
            return None

        # Use first criterion as primary key (typically bsn)
        key = str(list(criteria.values())[0])

        if key not in self._data:
            return None

        record = self._data[key]
        field_lower = field.lower()

        if field_lower not in record:
            return None

        return record[field_lower]

    def get_all_fields(self) -> set[str]:
        """Get all available field names"""
        return self._field_index.copy()


class DataSourceRegistry:
    """
    Registry for data sources with priority-based resolution.

    Sources are searched in priority order (highest first).
    First match wins.
    """

    def __init__(self) -> None:
        self._sources: dict[str, DataSource] = {}

    def register(self, source: DataSource) -> None:
        """Register a data source"""
        self._sources[source.name] = source
        logger.debug(
            f"Registered data source: {source.name} (priority {source.priority})"
        )

    def unregister(self, name: str) -> None:
        """Remove a data source by name"""
        if name in self._sources:
            del self._sources[name]
            logger.debug(f"Unregistered data source: {name}")

    def get_sources_sorted(self) -> list[DataSource]:
        """Get all sources sorted by priority (highest first)"""
        return sorted(
            self._sources.values(),
            key=lambda s: s.priority,
            reverse=True,
        )

    def resolve(
        self,
        field: str,
        criteria: dict[str, Any],
    ) -> DataSourceMatch | None:
        """
        Resolve a value from registered data sources

        Searches sources in priority order, returns first match.

        Args:
            field: Field name to resolve (will be lowercased)
            criteria: Selection criteria (e.g., {"bsn": "123456789"})

        Returns:
            DataSourceMatch if found, None otherwise
        """
        field_lower = field.lower()

        for source in self.get_sources_sorted():
            if not source.has_field(field_lower):
                continue

            value = source.get(field_lower, criteria)
            if value is not None:
                logger.debug(f"Resolved {field} from {source.name}: {value}")
                return DataSourceMatch(
                    value=value,
                    source_name=source.name,
                    source_type=type(source).__name__,
                )

        return None

    def list_sources(self) -> list[str]:
        """Get list of registered source names"""
        return list(self._sources.keys())

    def get_source(self, name: str) -> DataSource | None:
        """Get a specific source by name"""
        return self._sources.get(name)
