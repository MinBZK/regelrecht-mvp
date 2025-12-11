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
        self.rule_resolver = RuleResolver(regulation_dir)
        self.engine_cache: dict[tuple[str, str], ArticleEngine] = {}

        logger.info(
            f"Loaded {self.rule_resolver.get_law_count()} laws with {self.rule_resolver.get_output_count()} outputs"
        )

    def evaluate_uri(
        self,
        uri: str,
        parameters: dict,
        calculation_date: Optional[str] = None,
        requested_output: Optional[str] = None,
    ) -> ArticleResult:
        """
        Evaluate a regelrecht:// URI

        Args:
            uri: regelrecht:// URI to evaluate
            parameters: Input parameters (e.g., {"BSN": "123456789"})
            calculation_date: Date for which calculations are performed (defaults to today)
            requested_output: Specific output field to calculate (optional)

        Returns:
            ArticleResult with outputs

        Raises:
            ValueError: If URI cannot be resolved
        """
        logger.info(f"Evaluating URI: {uri}")

        # Default calculation date to today
        if calculation_date is None:
            calculation_date = datetime.now().date().isoformat()

        # Resolve URI to law, article, field
        law, article, field = self.rule_resolver.resolve_uri(uri)

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
            parameters, self, calculation_date, output_to_calculate
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

        uri = RegelrechtURIBuilder.build(law_id, output_name)
        return self.evaluate_uri(uri, parameters, calculation_date)

    def list_available_laws(self) -> list[str]:
        """Get list of all loaded law IDs"""
        return self.rule_resolver.list_all_laws()

    def list_available_outputs(self) -> list[tuple[str, str]]:
        """Get list of all (law_id, output_name) pairs"""
        return self.rule_resolver.list_all_outputs()

    def get_law_info(self, law_id: str) -> dict:
        """
        Get information about a law

        Args:
            law_id: Law identifier

        Returns:
            Dictionary with law metadata
        """
        law = self.rule_resolver.get_law_by_id(law_id)
        if not law:
            return {}

        return {
            "id": law.id,
            "uuid": law.uuid,
            "regulatory_layer": law.regulatory_layer,
            "publication_date": law.publication_date,
            "bwb_id": law.get_bwb_id(),
            "url": law.get_url(),
            "outputs": list(law.get_all_outputs().keys()),
            "article_count": len(law.articles),
        }

    # TODO: Generiek mechanisme voor uitvoerder data - nu hardcoded voor Diemen
    # Dit moet later vervangen worden door een service provider pattern
    _uitvoerder_data: dict[str, dict[str, int]] = {}

    @classmethod
    def set_gedragscategorie(cls, bsn: str, gemeente_code: str, categorie: int) -> None:
        """
        Set gedragscategorie for a BSN (test/mock data)

        Args:
            bsn: Burgerservicenummer
            gemeente_code: Gemeente code (e.g., "GM0384")
            categorie: Gedragscategorie (0, 1, 2, or 3)
        """
        key = f"{gemeente_code}:{bsn}"
        cls._uitvoerder_data[key] = {"gedragscategorie": categorie}

    @classmethod
    def get_gedragscategorie(cls, bsn: str, gemeente_code: str) -> int:
        """
        Get gedragscategorie for a BSN from uitvoerder data

        Args:
            bsn: Burgerservicenummer
            gemeente_code: Gemeente code (e.g., "GM0384")

        Returns:
            Gedragscategorie (0 if not set)
        """
        key = f"{gemeente_code}:{bsn}"
        data = cls._uitvoerder_data.get(key, {})
        return data.get("gedragscategorie", 0)

    @classmethod
    def clear_uitvoerder_data(cls) -> None:
        """Clear all uitvoerder test data"""
        cls._uitvoerder_data = {}
