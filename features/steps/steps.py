"""
Step definitions for healthcare allowance (zorgtoeslag) feature
"""

from behave import given, when, then  # type: ignore[import-untyped]
from mock_data_service import MockDataService


@given('the following {service} "{datasource}" data:')  # type: ignore[misc]
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


@when("the healthcare allowance law is executed")  # type: ignore[misc]
def step_when_healthcare_allowance_executed(context):
    """Execute the healthcare allowance law with mock data"""
    from engine.service import LawExecutionService

    # Create a wrapper service that uses mocks for external calls
    class MockLawExecutionService(LawExecutionService):
        def __init__(self, regulation_dir, mock_service):
            super().__init__(regulation_dir)
            self.mock_service = mock_service

        def evaluate_uri(
            self, uri, parameters, calculation_date=None, requested_output=None
        ):
            # All laws should now be real - no mocking needed!
            # Just use the real engine for everything
            return super().evaluate_uri(
                uri, parameters, calculation_date, requested_output
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


@when("I request the standard premium for year {year:d}")  # type: ignore[misc]
def step_when_request_standard_premium(context, year):
    """Request the standard premium for a specific year"""
    from engine.service import LawExecutionService

    # Create service
    service = LawExecutionService("regulation/nl")

    # Set calculation_date to match the year
    calculation_date = f"{year}-01-01"

    try:
        # Call the get_standaardpremie endpoint (Article 4)
        result = service.evaluate_law_endpoint(
            law_id="zorgtoeslagwet",
            endpoint="get_standaardpremie",
            parameters={},
            calculation_date=calculation_date,
        )
        context.result = result
    except Exception as e:
        context.error = e
        # Don't raise - let the Then step verify the error


@then('the standard premium is "{amount}" eurocent')  # type: ignore[misc]
def step_then_standard_premium(context, amount):
    """Verify the standard premium amount"""
    if hasattr(context, "error"):
        raise AssertionError(f"Execution failed: {context.error}")

    # Get the result
    result = context.result

    # The output should be in eurocent
    if "standaardpremie" in result.output:
        actual_amount = result.output["standaardpremie"]
    else:
        raise AssertionError(f"No 'standaardpremie' in outputs: {result.output}")

    # Compare with expected amount
    expected_amount = int(amount)

    if actual_amount != expected_amount:
        raise AssertionError(
            f"Expected premium of {expected_amount} eurocent, but got {actual_amount} eurocent"
        )


@then('the standard premium calculation should fail with "{error_message}"')  # type: ignore[misc]
def step_then_standard_premium_fails(context, error_message):
    """Verify that the calculation failed with expected error"""
    if not hasattr(context, "error"):
        raise AssertionError("Expected calculation to fail, but it succeeded")

    # Check if the error message contains the expected text
    error_str = str(context.error)
    if error_message not in error_str:
        raise AssertionError(
            f"Expected error to contain '{error_message}', but got: {error_str}"
        )


@then('the allowance amount is "{amount}" euro')  # type: ignore[misc]
def step_then_allowance_amount(context, amount):
    """Verify the calculated allowance amount"""
    if hasattr(context, "error"):
        raise AssertionError(f"Execution failed: {context.error}")

    # Get the result
    result = context.result

    # The output should be in eurocent, convert to euro
    if "HOOGTE_ZORGTOESLAG" in result.output:
        actual_amount_eurocent = result.output["HOOGTE_ZORGTOESLAG"]
        actual_amount_euro = actual_amount_eurocent / 100
    else:
        raise AssertionError(f"No 'HOOGTE_ZORGTOESLAG' in outputs: {result.output}")

    # Compare with expected amount
    expected_amount = float(amount)

    # Allow small rounding difference (0.01 euro = 1 eurocent)
    if abs(actual_amount_euro - expected_amount) > 0.01:
        raise AssertionError(
            f"Expected allowance of €{expected_amount:.2f}, but got €{actual_amount_euro:.2f}"
        )
