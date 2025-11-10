"""
Step definitions for healthcare allowance (zorgtoeslag) feature
"""

from behave import given, when, then
from mock_data_service import MockDataService


@given("the following RvIG personal data:")
def step_given_rvig_personal_data(context):
    """Store RvIG personal data from table"""
    if not hasattr(context, "mock_service"):
        context.mock_service = MockDataService()

    for row in context.table:
        data = {
            "bsn": row["bsn"],
            "geboortedatum": row["geboortedatum"],
            "verblijfsadres": row["verblijfsadres"],
            "land_verblijf": row["land_verblijf"],
        }
        context.mock_service.store_rvig_personal_data(data)
        # Store BSN for later use
        context.bsn = row["bsn"]


@given("the following RvIG relationship data:")
def step_given_rvig_relationship_data(context):
    """Store RvIG relationship data from table"""
    if not hasattr(context, "mock_service"):
        context.mock_service = MockDataService()

    for row in context.table:
        data = {
            "bsn": row["bsn"],
            "partnerschap_type": row["partnerschap_type"],
            "partner_bsn": row["partner_bsn"] if row["partner_bsn"] != "null" else None,
        }
        context.mock_service.store_rvig_relationship_data(data)


@given("the following RVZ insurance data:")
def step_given_rvz_insurance_data(context):
    """Store RVZ insurance data from table"""
    if not hasattr(context, "mock_service"):
        context.mock_service = MockDataService()

    for row in context.table:
        data = {
            "bsn": row["bsn"],
            "polis_status": row["polis_status"],
        }
        context.mock_service.store_rvz_insurance_data(data)


@given("the following BELASTINGDIENST box1 data:")
def step_given_belastingdienst_box1_data(context):
    """Store Belastingdienst box 1 data from table"""
    if not hasattr(context, "mock_service"):
        context.mock_service = MockDataService()

    for row in context.table:
        data = {
            "bsn": row["bsn"],
            "loon_uit_dienstbetrekking": float(row["loon_uit_dienstbetrekking"]),
            "uitkeringen_en_pensioenen": float(row["uitkeringen_en_pensioenen"]),
            "winst_uit_onderneming": float(row["winst_uit_onderneming"]),
            "resultaat_overige_werkzaamheden": float(
                row["resultaat_overige_werkzaamheden"]
            ),
            "eigen_woning": float(row["eigen_woning"]),
        }
        context.mock_service.store_belastingdienst_box1_data(data)


@given("the following BELASTINGDIENST box2 data:")
def step_given_belastingdienst_box2_data(context):
    """Store Belastingdienst box 2 data from table"""
    if not hasattr(context, "mock_service"):
        context.mock_service = MockDataService()

    for row in context.table:
        data = {
            "bsn": row["bsn"],
            "reguliere_voordelen": float(row["reguliere_voordelen"]),
            "vervreemdingsvoordelen": float(row["vervreemdingsvoordelen"]),
        }
        context.mock_service.store_belastingdienst_box2_data(data)


@given("the following BELASTINGDIENST box3 data:")
def step_given_belastingdienst_box3_data(context):
    """Store Belastingdienst box 3 data from table"""
    if not hasattr(context, "mock_service"):
        context.mock_service = MockDataService()

    for row in context.table:
        data = {
            "bsn": row["bsn"],
            "spaargeld": float(row["spaargeld"]),
            "beleggingen": float(row["beleggingen"]),
            "onroerend_goed": float(row["onroerend_goed"]),
            "schulden": float(row["schulden"]),
        }
        context.mock_service.store_belastingdienst_box3_data(data)


@when("the healthcare allowance law is executed")
def step_when_healthcare_allowance_executed(context):
    """Execute the healthcare allowance law with mock data"""
    from engine.service import LawExecutionService

    # Create a wrapper service that uses mocks for external calls
    class MockLawExecutionService(LawExecutionService):
        def __init__(self, regulation_dir, mock_service):
            super().__init__(regulation_dir)
            self.mock_service = mock_service

        def evaluate_uri(
            self, uri, parameters, reference_date=None, requested_output=None
        ):
            # If this is an external law call (not in our regulation directory), use mock
            if any(
                external in uri
                for external in [
                    "wet_brp",
                    "zvw",
                    "awir",
                    "belastingdienst",
                    "inkomstenbelasting",
                    "toeslagpartner",
                ]
            ):
                print(f"Mock call: {uri} -> {requested_output}")
                result = self.mock_service.get_mock_result(
                    uri, parameters, requested_output
                )
                print(f"Mock result: {result.output}")
                return result
            # Otherwise, use the real engine
            return super().evaluate_uri(
                uri, parameters, reference_date, requested_output
            )

    # Create service with mocks
    service = MockLawExecutionService("regulation/nl", context.mock_service)

    # Execute the law
    parameters = {"BSN": context.bsn}

    try:
        # Call the zorgtoeslag calculation endpoint
        result = service.evaluate_law_endpoint(
            law_id="zorgtoeslagwet",
            endpoint="bereken_zorgtoeslag",
            parameters=parameters,
        )
        context.result = result
    except Exception as e:
        context.error = e
        raise


@then('the allowance amount is "{amount}" euro')
def step_then_allowance_amount(context, amount):
    """Verify the calculated allowance amount"""
    if hasattr(context, "error"):
        raise AssertionError(f"Execution failed: {context.error}")

    # Get the result
    result = context.result

    # The output should be in eurocent, convert to euro
    if "hoogte_zorgtoeslag" in result.output:
        actual_amount_eurocent = result.output["hoogte_zorgtoeslag"]
        actual_amount_euro = actual_amount_eurocent / 100
    else:
        raise AssertionError(f"No 'hoogte_zorgtoeslag' in outputs: {result.output}")

    # Compare with expected amount
    expected_amount = float(amount)

    # Allow small rounding difference (0.01 euro = 1 eurocent)
    if abs(actual_amount_euro - expected_amount) > 0.01:
        raise AssertionError(
            f"Expected allowance of €{expected_amount:.2f}, but got €{actual_amount_euro:.2f}"
        )
