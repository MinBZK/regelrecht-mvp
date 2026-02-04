"""
Law Execution Service

Top-level service for executing article-based laws via URI resolution.
"""

from typing import Any, Optional
from datetime import datetime

from engine.rule_resolver import RuleResolver
from engine.engine import ArticleEngine, ArticleResult
from engine.logging_config import logger
from engine.data_sources import DataSource, DataSourceRegistry, DictDataSource, UitvoerderDataSource

# Maximum recursion depth for law references to prevent stack overflow
MAX_RECURSION_DEPTH = 50


class RecursionDepthError(Exception):
    """Raised when law reference recursion exceeds maximum depth"""

    pass


class LawExecutionService:
    """Service for executing article-based laws"""

    def __init__(self, regulation_dir: str = "regulation/nl"):
        """
        Initialize law execution service

        Args:
            regulation_dir: Path to regulation directory
        """
        self.rule_resolver = RuleResolver(regulation_dir)
        self.engine_cache: dict[tuple[str, str], ArticleEngine] = {}
        self.data_registry = DataSourceRegistry()

        logger.info(
            f"Loaded {self.rule_resolver.get_law_count()} laws "
            f"({self.rule_resolver.get_version_count()} versions)"
        )

    def evaluate_uri(
        self,
        uri: str,
        parameters: dict,
        calculation_date: Optional[str] = None,
        requested_output: Optional[str] = None,
        _depth: int = 0,
    ) -> ArticleResult:
        """
        Evaluate a regelrecht:// URI

        Args:
            uri: regelrecht:// URI to evaluate
            parameters: Input parameters (e.g., {"BSN": "123456789"})
            calculation_date: Date for which calculations are performed (defaults to today)
            requested_output: Specific output field to calculate (optional)
            _depth: Internal recursion depth counter (do not set manually)

        Returns:
            ArticleResult with outputs

        Raises:
            ValueError: If URI cannot be resolved
            RecursionDepthError: If recursion depth exceeds MAX_RECURSION_DEPTH
        """
        # Check recursion depth to prevent stack overflow from circular references
        if _depth > MAX_RECURSION_DEPTH:
            raise RecursionDepthError(
                f"Maximum recursion depth ({MAX_RECURSION_DEPTH}) exceeded while evaluating {uri}. "
                f"This likely indicates circular law references."
            )

        logger.info(f"Evaluating URI: {uri} (depth={_depth})")

        # Default calculation date to today
        if calculation_date is None:
            calculation_date = datetime.now().date().isoformat()

        # Parse calculation_date for version selection
        reference_date = datetime.strptime(calculation_date, "%Y-%m-%d")

        # Resolve URI to law, article, field (using reference_date for version selection)
        law, article, field = self.rule_resolver.resolve_uri(uri, reference_date)

        if not law or not article:
            raise ValueError(f"Could not resolve URI: {uri}")

        # Get or create engine for this article
        output_names = article.get_output_names()
        if not output_names:
            raise ValueError(f"Article has no outputs: {article.number}")

        # Use first output name as cache key
        cache_key = (law.id, output_names[0])

        if cache_key not in self.engine_cache:
            logger.debug(f"Creating engine for {law.id}/{output_names[0]}")
            self.engine_cache[cache_key] = ArticleEngine(article, law)

        engine = self.engine_cache[cache_key]

        # Determine what to calculate
        output_to_calculate = requested_output
        if output_to_calculate is None:
            # Use field from URI if no requested_output specified
            output_to_calculate = field

        # Execute article
        result = engine.evaluate(
            parameters,
            self,
            calculation_date,
            output_to_calculate,
            data_registry=self.data_registry,
            _depth=_depth,
        )

        return result

    def evaluate_law_output(
        self,
        law_id: str,
        output_name: str,
        parameters: dict,
        calculation_date: Optional[str] = None,
    ) -> ArticleResult:
        """
        Evaluate by law ID and output name directly

        Args:
            law_id: Law identifier (e.g., "zorgtoeslagwet")
            output_name: Output name (e.g., "bereken_zorgtoeslag")
            parameters: Input parameters
            calculation_date: Date for which calculations are performed

        Returns:
            ArticleResult with outputs
        """
        from engine.uri_resolver import RegelrechtURIBuilder

        uri = RegelrechtURIBuilder.build(law_id, output_name, output_name)
        return self.evaluate_uri(uri, parameters, calculation_date)

    def list_available_laws(self) -> list[str]:
        """Get list of all loaded law IDs"""
        return self.rule_resolver.list_all_laws()

    def list_available_outputs(self) -> list[tuple[str, str]]:
        """Get list of all (law_id, output_name) pairs"""
        return self.rule_resolver.list_all_outputs()

    def get_law_info(self, law_id: str, reference_date: datetime | None = None) -> dict:
        """
        Get information about a law

        Args:
            law_id: Law identifier
            reference_date: Date for version selection (uses most recent if None)

        Returns:
            Dictionary with law metadata
        """
        law = self.rule_resolver.get_law_by_id(law_id, reference_date)
        if not law:
            return {}

        return {
            "id": law.id,
            "regulatory_layer": law.regulatory_layer,
            "publication_date": law.publication_date,
            "valid_from": law.valid_from,
            "bwb_id": law.get_bwb_id(),
            "url": law.get_url(),
            "outputs": list(law.get_all_outputs().keys()),
            "article_count": len(law.articles),
        }

    # === Data Source Management ===

    def add_data_source(self, source: DataSource) -> None:
        """
        Add a data source to the registry

        Args:
            source: DataSource implementation to register
        """
        self.data_registry.register(source)

    def add_dict_source(
        self,
        name: str,
        data: dict[str, dict[str, Any]],
        priority: int = 100,
    ) -> None:
        """
        Add a dict-based data source

        Args:
            name: Unique name for the source
            data: Dict of {key: {field: value}} records
            priority: Priority for disambiguation (higher = preferred)
        """
        source = DictDataSource(name=name, priority=priority)
        for key, record in data.items():
            source.store(key, record)
        self.data_registry.register(source)

    def clear_data_sources(self) -> None:
        """Remove all registered data sources"""
        self.data_registry = DataSourceRegistry()

    def get_uitvoerder_source(self, gemeente_code: str) -> UitvoerderDataSource:
        """Get or create an uitvoerder data source for a gemeente."""
        source_name = f"uitvoerder_{gemeente_code}"
        existing = self.data_registry.get_source(source_name)
        if existing is not None:
            return existing  # type: ignore[return-value]
        # Create new source
        source = UitvoerderDataSource(gemeente_code=gemeente_code)
        self.data_registry.register(source)
        return source
