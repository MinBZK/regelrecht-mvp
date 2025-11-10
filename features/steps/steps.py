"""
Step definitions for healthcare allowance (zorgtoeslag) feature
"""

from behave import given, when, then
from mock_data_service import MockDataService


@given('the following {service} "{datasource}" data:')
def step_given_service_datasource_data(context, service, datasource):
    """
    Generic step to store service datasource data from table

    Args:
        service: Service name (e.g., "RVIG", "BELASTINGDIENST")
        datasource: Datasource name (e.g., "personal_data", "box1")
    """
    if not hasattr(context, "mock_service"):
        context.mock_service = MockDataService()

    for row in context.table:
        # Convert all values to appropriate types
        data = {"bsn": row["bsn"]}

        # Add all other columns
        for key in row.headings:
            if key != "bsn":
                value = row[key]
                # Handle null values
                if value == "null":
                    data[key] = None
                # Try to convert to float if it looks numeric
                else:
                    try:
                        data[key] = float(value)
                    except ValueError:
                        data[key] = value

        context.mock_service.store_data(service, datasource, data)
        # Store BSN for later use
        context.bsn = row["bsn"]


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
