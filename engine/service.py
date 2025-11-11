"""
Law Execution Service

Top-level service for executing article-based laws via URI resolution.
"""

from typing import Optional
from datetime import datetime

from engine.rule_resolver import RuleResolver
from engine.engine import ArticleEngine, ArticleResult
from engine.logging_config import logger


class LawExecutionService:
    """Service for executing article-based laws"""

    def __init__(self, regulation_dir: str = "regulation/nl"):
        """
        Initialize law execution service

        Args:
            regulation_dir: Path to regulation directory
        """
        self.resolver = RuleResolver(regulation_dir)
        self.rule_resolver = self.resolver  # Alias for backward compatibility
        self.engine_cache: dict[tuple[str, str], ArticleEngine] = {}

        logger.info(
            f"Loaded {self.resolver.get_law_count()} laws with {self.resolver.get_endpoint_count()} endpoints"
        )

    def evaluate_uri(
        self,
        uri: str,
        parameters: dict,
        reference_date: Optional[str] = None,
        requested_output: Optional[str] = None,
    ) -> ArticleResult:
        """
        Evaluate a regelrecht:// URI

        Args:
            uri: regelrecht:// URI to evaluate
            parameters: Input parameters (e.g., {"BSN": "123456789"})
            reference_date: Reference date for calculations (defaults to today)
            requested_output: Specific output field to calculate (optional)

        Returns:
            ArticleResult with outputs

        Raises:
            ValueError: If URI cannot be resolved
        """
        logger.info(f"Evaluating URI: {uri}")

        # Default reference date to today
        if reference_date is None:
            reference_date = datetime.now().date().isoformat()

        # Resolve URI to law, article, field
        law, article, field = self.resolver.resolve_uri(uri)

        if not law or not article:
            raise ValueError(f"Could not resolve URI: {uri}")

        # Get or create engine for this article
        endpoint = article.get_endpoint()
        cache_key = (law.id, endpoint)

        if cache_key not in self.engine_cache:
            logger.debug(f"Creating engine for {law.id}/{endpoint}")
            self.engine_cache[cache_key] = ArticleEngine(article, law)

        engine = self.engine_cache[cache_key]

        # Determine what to calculate
        output_to_calculate = requested_output
        if output_to_calculate is None:
            # Use field from URI if no requested_output specified
            output_to_calculate = field

        # Execute article
        result = engine.evaluate(
            parameters, self, reference_date, output_to_calculate
        )

        return result

    def evaluate_law_endpoint(
        self,
        law_id: str,
        endpoint: str,
        parameters: dict,
        reference_date: Optional[str] = None,
    ) -> ArticleResult:
        """
        Evaluate by law ID and endpoint directly

        Args:
            law_id: Law identifier (e.g., "zorgtoeslagwet")
            endpoint: Endpoint name (e.g., "bereken_zorgtoeslag")
            parameters: Input parameters
            reference_date: Reference date for calculations

        Returns:
            ArticleResult with outputs
        """
        uri = f"regelrecht://{law_id}/{endpoint}"
        return self.evaluate_uri(uri, parameters, reference_date)

    def list_available_laws(self) -> list[str]:
        """Get list of all loaded law IDs"""
        return self.resolver.list_all_laws()

    def list_available_endpoints(self) -> list[tuple[str, str]]:
        """Get list of all (law_id, endpoint) pairs"""
        return self.resolver.list_all_endpoints()

    def get_law_info(self, law_id: str) -> dict:
        """
        Get information about a law

        Args:
            law_id: Law identifier

        Returns:
            Dictionary with law metadata
        """
        law = self.resolver.get_law_by_id(law_id)
        if not law:
            return {}

        return {
            "id": law.id,
            "uuid": law.uuid,
            "regulatory_layer": law.regulatory_layer,
            "publication_date": law.publication_date,
            "bwb_id": law.get_bwb_id(),
            "url": law.get_url(),
            "endpoints": list(law.get_all_endpoints().keys()),
            "article_count": len(law.articles),
        }
