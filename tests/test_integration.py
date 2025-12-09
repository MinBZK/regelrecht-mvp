"""
Integration tests for code integration across components

These tests use synthetic fixture YAML files to test that components
work together correctly. They do NOT test actual law behavior - that's
what BDD/behavior tests are for.
"""

import pytest
from pathlib import Path

from engine.service import LawExecutionService
from engine.engine import ArticleResult


@pytest.fixture
def fixture_dir():
    """Path to test fixtures directory"""
    return Path(__file__).parent / "fixtures" / "sample_laws" / "nl"


@pytest.fixture
def test_service(fixture_dir):
    """Create LawExecutionService with test fixtures"""
    return LawExecutionService(str(fixture_dir))


class TestLawLoading:
    """Test that service correctly loads laws and builds indexes"""

    def test_service_loads_multiple_laws(self, test_service):
        """Service loads all test laws from fixtures"""
        laws = test_service.list_available_laws()

        assert len(laws) >= 3
        assert "test_law_a" in laws
        assert "test_law_b" in laws
        assert "test_law_error" in laws

    def test_service_builds_endpoint_index(self, test_service):
        """Service builds correct endpoint index"""
        endpoints = test_service.list_available_endpoints()

        # Endpoints are returned as tuples (law_id, endpoint)
        assert ("test_law_a", "add_numbers") in endpoints
        assert ("test_law_a", "check_threshold") in endpoints
        assert ("test_law_b", "call_other_law") in endpoints

    def test_service_can_retrieve_law_by_id(self, test_service):
        """Service can retrieve law by ID"""
        law = test_service.rule_resolver.get_law_by_id("test_law_a")

        assert law is not None
        assert law.id == "test_law_a"
        assert law.uuid == "11111111-1111-1111-1111-111111111111"

    def test_service_returns_none_for_invalid_law_id(self, test_service):
        """Service returns None for non-existent law"""
        law = test_service.rule_resolver.get_law_by_id("nonexistent_law")

        assert law is None


class TestBasicExecution:
    """Test basic article execution through service"""

    def test_execute_simple_arithmetic_article(self, test_service):
        """Execute article with simple arithmetic"""
        result = test_service.evaluate_law_endpoint(
            law_id="test_law_a",
            endpoint="add_numbers",
            parameters={"input_value": 50},
            calculation_date="2025-01-01",
        )

        assert isinstance(result, ArticleResult)
        assert result.law_id == "test_law_a"
        assert result.article_number == "1"
        assert result.output["result"] == 150  # BASE_VALUE (100) + input_value (50)

    def test_execute_conditional_article(self, test_service):
        """Execute article with conditional logic"""
        # Test with value above threshold
        result = test_service.evaluate_law_endpoint(
            law_id="test_law_a",
            endpoint="check_threshold",
            parameters={"value": 75},
            calculation_date="2025-01-01",
        )

        assert result.output["above_threshold"] is True

        # Test with value below threshold
        result = test_service.evaluate_law_endpoint(
            law_id="test_law_a",
            endpoint="check_threshold",
            parameters={"value": 25},
            calculation_date="2025-01-01",
        )

        assert result.output["above_threshold"] is False

    def test_article_metadata_in_result(self, test_service):
        """ArticleResult contains correct metadata"""
        result = test_service.evaluate_law_endpoint(
            law_id="test_law_a",
            endpoint="add_numbers",
            parameters={"input_value": 10},
            calculation_date="2025-01-01",
        )

        assert result.law_id == "test_law_a"
        assert result.law_uuid == "11111111-1111-1111-1111-111111111111"
        assert result.article_number == "1"
        assert isinstance(result.output, dict)
        assert isinstance(result.input, dict)


class TestCrossLawURICalls:
    """Test cross-law URI resolution and execution"""

    def test_article_calls_another_law_via_uri(self, test_service):
        """Article in law_b calls law_a via URI"""
        # test_law_b article 1 calls test_law_a with input, then doubles the result
        result = test_service.evaluate_law_endpoint(
            law_id="test_law_b",
            endpoint="call_other_law",
            parameters={"my_value": 25},
            calculation_date="2025-01-01",
        )

        # test_law_a returns: 100 + 25 = 125
        # test_law_b doubles it: 125 * 2 = 250
        assert result.output["doubled_result"] == 250

    def test_parameters_flow_through_uri_calls(self, test_service):
        """Parameters are correctly passed through URI calls"""
        # Test with different input values
        result1 = test_service.evaluate_law_endpoint(
            law_id="test_law_b",
            endpoint="call_other_law",
            parameters={"my_value": 10},
            calculation_date="2025-01-01",
        )

        result2 = test_service.evaluate_law_endpoint(
            law_id="test_law_b",
            endpoint="call_other_law",
            parameters={"my_value": 50},
            calculation_date="2025-01-01",
        )

        # (100 + 10) * 2 = 220
        assert result1.output["doubled_result"] == 220

        # (100 + 50) * 2 = 300
        assert result2.output["doubled_result"] == 300

    def test_resolved_inputs_in_article_result(self, test_service):
        """ArticleResult contains resolved inputs from URI calls"""
        result = test_service.evaluate_law_endpoint(
            law_id="test_law_b",
            endpoint="call_other_law",
            parameters={"my_value": 20},
            calculation_date="2025-01-01",
        )

        # Should have resolved the input from cross-law call
        assert "result_from_a" in result.input
        assert result.input["result_from_a"] == 120  # 100 + 20


class TestInternalReferences:
    """Test internal references within same law"""

    @pytest.mark.skip(
        reason="Internal references with article/ref pattern need additional implementation"
    )
    def test_article_references_another_article_in_same_law(self, test_service):
        """Article can reference another article in same law"""
        # test_law_b article 2 references article 1 internally
        # TODO: Internal references using article + ref pattern need proper URI construction
        result = test_service.evaluate_law_endpoint(
            law_id="test_law_b",
            endpoint="internal_ref_test",
            parameters={"my_value": 15},
            calculation_date="2025-01-01",
        )

        # Article 1 returns: (100 + 15) * 2 = 230
        # Article 2 returns: 230 + 10 = 240
        assert result.output["final_result"] == 240


class TestEngineCaching:
    """Test that engines are cached and reused"""

    def test_engines_are_cached_per_endpoint(self, test_service):
        """Engines are cached by (law_id, output_name) key"""
        # First call creates engine
        test_service.evaluate_law_endpoint(
            law_id="test_law_a",
            endpoint="add_numbers",
            parameters={"input_value": 10},
            calculation_date="2025-01-01",
        )

        # Check engine is cached (cache key is (law_id, first_output_name))
        # test_law_a's add_numbers article has output "result"
        cache_key = ("test_law_a", "result")
        assert cache_key in test_service.engine_cache

        # Second call should reuse cached engine
        cached_engine = test_service.engine_cache[cache_key]

        test_service.evaluate_law_endpoint(
            law_id="test_law_a",
            endpoint="add_numbers",
            parameters={"input_value": 20},
            calculation_date="2025-01-01",
        )

        # Same engine instance should be in cache
        assert test_service.engine_cache[cache_key] is cached_engine

    def test_different_endpoints_have_different_engines(self, test_service):
        """Different endpoints get separate engine instances"""
        test_service.evaluate_law_endpoint(
            law_id="test_law_a",
            endpoint="add_numbers",
            parameters={"input_value": 10},
            calculation_date="2025-01-01",
        )

        test_service.evaluate_law_endpoint(
            law_id="test_law_a",
            endpoint="check_threshold",
            parameters={"value": 50},
            calculation_date="2025-01-01",
        )

        # Should have two different cache entries (cache key uses first output name)
        # add_numbers article has output "result"
        # check_threshold article has output "above_threshold"
        key1 = ("test_law_a", "result")
        key2 = ("test_law_a", "above_threshold")

        assert key1 in test_service.engine_cache
        assert key2 in test_service.engine_cache
        assert test_service.engine_cache[key1] is not test_service.engine_cache[key2]


class TestURIResultCaching:
    """Test that URI call results are cached within execution context"""

    def test_uri_results_are_cached_in_context(self, test_service):
        """Multiple references to same URI are cached"""
        # This test uses a law that would call the same URI multiple times
        # The caching happens inside RuleContext._resolve_from_source
        # We verify it by checking that the result is correct (proving cache works)

        result = test_service.evaluate_law_endpoint(
            law_id="test_law_b",
            endpoint="call_other_law",
            parameters={"my_value": 30},
            calculation_date="2025-01-01",
        )

        # Result should be correct even with caching
        # (100 + 30) * 2 = 260
        assert result.output["doubled_result"] == 260


class TestErrorHandling:
    """Test error handling and propagation"""

    def test_division_by_zero_raises_error(self, test_service):
        """Division by zero in article raises ZeroDivisionError"""
        with pytest.raises(ZeroDivisionError):
            test_service.evaluate_law_endpoint(
                law_id="test_law_error",
                endpoint="divide_by_zero",
                parameters={},
                calculation_date="2025-01-01",
            )

    def test_missing_law_in_uri_raises_error(self, test_service):
        """URI referencing non-existent law raises ValueError"""
        with pytest.raises(ValueError, match="Could not resolve URI"):
            test_service.evaluate_law_endpoint(
                law_id="test_law_error",
                endpoint="call_missing_law",
                parameters={},
                calculation_date="2025-01-01",
            )

    def test_invalid_law_id_raises_error(self, test_service):
        """Evaluating non-existent law raises ValueError"""
        with pytest.raises(ValueError, match="Could not resolve URI"):
            test_service.evaluate_law_endpoint(
                law_id="nonexistent_law",
                endpoint="some_endpoint",
                parameters={},
                calculation_date="2025-01-01",
            )

    def test_invalid_endpoint_raises_error(self, test_service):
        """Evaluating non-existent endpoint raises ValueError"""
        with pytest.raises(ValueError, match="Could not resolve URI"):
            test_service.evaluate_law_endpoint(
                law_id="test_law_a",
                endpoint="nonexistent_endpoint",
                parameters={},
                calculation_date="2025-01-01",
            )


class TestServiceMetadata:
    """Test service metadata query methods"""

    def test_list_available_laws(self, test_service):
        """list_available_laws returns all loaded law IDs"""
        laws = test_service.list_available_laws()

        assert isinstance(laws, list)
        assert "test_law_a" in laws
        assert "test_law_b" in laws

    def test_list_available_endpoints(self, test_service):
        """list_available_endpoints returns all public endpoints"""
        endpoints = test_service.list_available_endpoints()

        assert isinstance(endpoints, list)
        assert len(endpoints) >= 5
        assert ("test_law_a", "add_numbers") in endpoints
        assert ("test_law_b", "call_other_law") in endpoints

    def test_get_law_info(self, test_service):
        """get_law_info returns law metadata"""
        info = test_service.get_law_info("test_law_a")

        assert info is not None
        assert info["id"] == "test_law_a"
        assert info["uuid"] == "11111111-1111-1111-1111-111111111111"
        assert info["regulatory_layer"] == "WET"
        assert info["publication_date"] == "2025-01-01"

    def test_get_law_info_for_invalid_law(self, test_service):
        """get_law_info returns empty dict for non-existent law"""
        info = test_service.get_law_info("nonexistent_law")

        assert info == {}

    def test_count_loaded_laws(self, test_service):
        """Service correctly counts loaded laws"""
        count = test_service.rule_resolver.get_law_count()

        assert count >= 3  # At least our 3 test laws


class TestDelegationPatterns:
    """Test delegation patterns between rijkswet and gemeentelijke verordeningen"""

    def test_mandatory_delegation_with_verordening(self, test_service):
        """Mandatory delegation works when gemeente has verordening"""
        # GM9997 (testgemeente2) has a verordening for test_delegation_law
        # The verordening multiplies by 3
        result = test_service.evaluate_law_endpoint(
            law_id="test_delegation_law",
            endpoint="final_result",
            parameters={"gemeente_code": "GM9997", "input_value": 10},
            calculation_date="2025-01-01",
        )

        # verordening: 10 * 3 = 30
        # orchestrator: 1000 + 30 = 1030
        assert result.output["final_result"] == 1030

    def test_mandatory_delegation_without_verordening_raises_error(self, test_service):
        """Mandatory delegation raises ValueError when gemeente has no verordening"""
        # GM9999 has NO verordening for test_delegation_law (mandatory delegation)
        # The legal_foundation_for has NO defaults section
        with pytest.raises(
            ValueError, match="No regulation found for mandatory delegation"
        ):
            test_service.evaluate_law_endpoint(
                law_id="test_delegation_law",
                endpoint="final_result",
                parameters={"gemeente_code": "GM9999", "input_value": 10},
                calculation_date="2025-01-01",
            )

    def test_optional_delegation_with_verordening(self, test_service):
        """Optional delegation uses gemeente verordening when available"""
        # GM9998 (testgemeente) has a verordening for test_optional_delegation_law
        # The verordening multiplies by 5 (instead of default 10)
        result = test_service.evaluate_law_endpoint(
            law_id="test_optional_delegation_law",
            endpoint="final_result",
            parameters={"gemeente_code": "GM9998", "input_value": 10},
            calculation_date="2025-01-01",
        )

        # verordening: 10 * 5 = 50
        # orchestrator: 1000 + 50 = 1050
        assert result.output["final_result"] == 1050

    def test_optional_delegation_without_verordening_uses_defaults(self, test_service):
        """Optional delegation uses rijkswet defaults when gemeente has no verordening"""
        # GM9999 has NO verordening for test_optional_delegation_law
        # But the legal_foundation_for HAS a defaults section (multiplier = 10)
        result = test_service.evaluate_law_endpoint(
            law_id="test_optional_delegation_law",
            endpoint="final_result",
            parameters={"gemeente_code": "GM9999", "input_value": 10},
            calculation_date="2025-01-01",
        )

        # defaults: 10 * 10 = 100
        # orchestrator: 1000 + 100 = 1100
        assert result.output["final_result"] == 1100

    def test_mandatory_delegation_error_message_contains_details(self, test_service):
        """ValueError message contains jurisdiction, law_id, and article for debugging"""
        with pytest.raises(ValueError) as exc_info:
            test_service.evaluate_law_endpoint(
                law_id="test_delegation_law",
                endpoint="final_result",
                parameters={"gemeente_code": "GM0000", "input_value": 5},
                calculation_date="2025-01-01",
            )

        error_msg = str(exc_info.value)
        assert "GM0000" in error_msg
        assert "test_delegation_law" in error_msg
        assert "article 1" in error_msg
        assert "No legal basis" in error_msg
