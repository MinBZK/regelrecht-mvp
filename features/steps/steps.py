"""
Step definitions for healthcare allowance (zorgtoeslag) and bijstand features
"""

import sys
import os

# Add the steps directory to the path for imports
sys.path.insert(0, os.path.dirname(__file__))

from behave import given, when, then  # type: ignore[import-untyped]
from mock_data_service import MockDataService  # type: ignore[import-not-found]


# === Bijstand step definitions ===


@given('the calculation date is "{date}"')  # type: ignore[misc]
def step_given_calculation_date(context, date):
    """Set the calculation date for the test"""
    context.calculation_date = date


@given("a citizen with the following data:")  # type: ignore[misc]
def step_given_citizen_data(context):
    """Store citizen data from table for bijstand test (key | value format)"""
    context.citizen_data = {}

    def convert_value(val):
        """Convert string value to appropriate type"""
        if val == "true":
            return True
        elif val == "false":
            return False
        elif val == "null":
            return None
        else:
            try:
                return int(val)
            except (ValueError, TypeError):
                try:
                    return float(val)
                except (ValueError, TypeError):
                    return val  # Keep as string

    # In behave, the first row becomes headings, so extract from there too
    # Format: | key | value |
    if len(context.table.headings) == 2:
        key = context.table.headings[0]
        value = convert_value(context.table.headings[1])
        context.citizen_data[key] = value

    # Process remaining rows
    for row in context.table:
        key = row[0]
        value = convert_value(row[1])
        context.citizen_data[key] = value


@when("the bijstandsaanvraag is executed for participatiewet article {article}")  # type: ignore[misc]
def step_when_bijstandsaanvraag_executed(context, article):
    """Execute the bijstandsaanvraag"""
    from engine.service import LawExecutionService

    # Create service
    service = LawExecutionService("regulation/nl")

    # Get calculation date
    calculation_date = getattr(context, "calculation_date", "2024-01-01")

    # Build parameters from citizen data
    parameters = context.citizen_data.copy()

    # Ensure BSN is present (generate test BSN if not provided)
    if "bsn" not in parameters:
        parameters["bsn"] = "123456789"

    # Ensure gedragscategorie has a default value if not provided
    if "gedragscategorie" not in parameters:
        parameters["gedragscategorie"] = 0

    # Ensure is_gelijkgestelde_vreemdeling has a default value
    if "is_gelijkgestelde_vreemdeling" not in parameters:
        parameters["is_gelijkgestelde_vreemdeling"] = False

    try:
        # Call Article 43 via uitkering_bedrag to execute all actions
        result = service.evaluate_law_output(
            law_id="participatiewet",
            output_name="uitkering_bedrag",
            parameters=parameters,
            calculation_date=calculation_date,
        )
        context.result = result
        context.error = None
    except Exception as e:
        context.error = e
        context.result = None


@then("the citizen has the right to bijstand")  # type: ignore[misc]
def step_then_has_right_to_bijstand(context):
    """Verify the citizen has the right to bijstand"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    if "heeft_recht_op_bijstand" not in result.output:
        raise AssertionError(
            f"No 'heeft_recht_op_bijstand' in outputs: {result.output}"
        )

    if not result.output["heeft_recht_op_bijstand"]:
        reden = result.output.get("reden_afwijzing", "onbekend")
        raise AssertionError(
            f"Expected citizen to have right to bijstand, but was denied: {reden}"
        )


@then("the citizen does not have the right to bijstand")  # type: ignore[misc]
def step_then_no_right_to_bijstand(context):
    """Verify the citizen does not have the right to bijstand"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    if "heeft_recht_op_bijstand" not in result.output:
        raise AssertionError(
            f"No 'heeft_recht_op_bijstand' in outputs: {result.output}"
        )

    if result.output["heeft_recht_op_bijstand"]:
        raise AssertionError(
            "Expected citizen to NOT have right to bijstand, but was approved"
        )


@then('the normbedrag is "{amount}" eurocent')  # type: ignore[misc]
def step_then_normbedrag(context, amount):
    """Verify the normbedrag"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    actual = result.output.get("normbedrag")
    expected = int(amount)

    if actual != expected:
        raise AssertionError(f"Expected normbedrag {expected}, but got {actual}")


@then('the verlaging_percentage is "{percentage}"')  # type: ignore[misc]
def step_then_verlaging_percentage(context, percentage):
    """Verify the verlaging percentage"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    actual = result.output.get("verlaging_percentage")
    expected = int(percentage)

    if actual != expected:
        raise AssertionError(
            f"Expected verlaging_percentage {expected}, but got {actual}"
        )


@then('the uitkering_bedrag is "{amount}" eurocent')  # type: ignore[misc]
def step_then_uitkering_bedrag(context, amount):
    """Verify the final uitkering amount"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    actual = result.output.get("uitkering_bedrag")
    # Round to nearest integer since we're dealing with eurocent
    if isinstance(actual, float):
        actual = round(actual)
    expected = int(amount)

    if actual != expected:
        raise AssertionError(f"Expected uitkering_bedrag {expected}, but got {actual}")


@then('the reden_afwijzing contains "{text}"')  # type: ignore[misc]
def step_then_reden_afwijzing_contains(context, text):
    """Verify the rejection reason contains expected text"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    reden = result.output.get("reden_afwijzing", "")

    if reden is None:
        raise AssertionError(
            f"Expected reden_afwijzing to contain '{text}', but was None"
        )

    if text.lower() not in reden.lower():
        raise AssertionError(
            f"Expected reden_afwijzing to contain '{text}', but got: {reden}"
        )


@then('the execution fails with "{error_text}"')  # type: ignore[misc]
def step_then_execution_fails_with(context, error_text):
    """Verify that the execution failed with expected error message"""
    if not context.error:
        raise AssertionError(
            f"Expected execution to fail with '{error_text}', but it succeeded"
        )

    error_str = str(context.error)
    if error_text.lower() not in error_str.lower():
        raise AssertionError(
            f"Expected error to contain '{error_text}', but got: {error_str}"
        )


# === Zorgtoeslag step definitions ===


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
    parameters = {"bsn": context.bsn}

    try:
        # Call the zorgtoeslag calculation output
        result = service.evaluate_law_output(
            law_id="zorgtoeslagwet",
            output_name="hoogte_zorgtoeslag",
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
        # Call the get_standaardpremie output (Article 4)
        result = service.evaluate_law_output(
            law_id="zorgtoeslagwet",
            output_name="standaardpremie",
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


# === Erfgrensbeplanting step definitions ===


@given("a query with the following data:")  # type: ignore[misc]
def step_given_query_data(context):
    """Store query data from table (key | value format)"""
    context.query_data = {}

    def convert_value(val):
        """Convert string value to appropriate type"""
        if val == "true":
            return True
        elif val == "false":
            return False
        elif val == "null":
            return None
        else:
            try:
                return int(val)
            except (ValueError, TypeError):
                try:
                    return float(val)
                except (ValueError, TypeError):
                    return val

    if len(context.table.headings) == 2:
        key = context.table.headings[0]
        value = convert_value(context.table.headings[1])
        context.query_data[key] = value

    for row in context.table:
        key = row[0]
        value = convert_value(row[1])
        context.query_data[key] = value


@when("the erfgrensbeplanting is requested for {law_id} article {article}")  # type: ignore[misc]
def step_when_erfgrensbeplanting_requested(context, law_id, article):
    """Execute the erfgrensbeplanting query"""
    from engine.service import LawExecutionService

    service = LawExecutionService("regulation/nl")
    calculation_date = getattr(context, "calculation_date", "2024-01-01")
    parameters = context.query_data.copy()

    try:
        result = service.evaluate_law_output(
            law_id=law_id,
            output_name="minimale_afstand_cm",
            parameters=parameters,
            calculation_date=calculation_date,
        )
        context.result = result
        context.error = None
    except Exception as e:
        context.error = e
        context.result = None


@then('the minimale_afstand_cm is "{amount}"')  # type: ignore[misc]
def step_then_minimale_afstand_cm(context, amount):
    """Verify the minimale afstand in centimeters"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    actual = result.output.get("minimale_afstand_cm")
    expected = int(amount)

    if actual != expected:
        raise AssertionError(
            f"Expected minimale_afstand_cm {expected}, but got {actual}"
        )


@then('the minimale_afstand_m is "{amount}"')  # type: ignore[misc]
def step_then_minimale_afstand_m(context, amount):
    """Verify the minimale afstand in meters"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    actual = result.output.get("minimale_afstand_m")
    expected = float(amount)

    if abs(actual - expected) > 0.01:
        raise AssertionError(
            f"Expected minimale_afstand_m {expected}, but got {actual}"
        )


# === Gewone Bijstand step definitions ===


@given("a citizen with the following profile:")  # type: ignore[misc]
def step_given_citizen_profile(context):
    """Store citizen profile data from table (key | value format)"""
    context.citizen_profile = {}

    def convert_value(val):
        """Convert string value to appropriate type"""
        if val == "true":
            return True
        elif val == "false":
            return False
        elif val == "null":
            return None
        else:
            try:
                return int(val)
            except (ValueError, TypeError):
                try:
                    return float(val)
                except (ValueError, TypeError):
                    return val

    if len(context.table.headings) == 2:
        key = context.table.headings[0]
        value = convert_value(context.table.headings[1])
        context.citizen_profile[key] = value

    for row in context.table:
        key = row[0]
        value = convert_value(row[1])
        context.citizen_profile[key] = value


@when("I determine the huishouden_type for participatiewet")  # type: ignore[misc]
def step_when_determine_huishouden_type(context):
    """Determine household type via Article 4"""
    from engine.service import LawExecutionService

    service = LawExecutionService("regulation/nl")
    calculation_date = getattr(context, "calculation_date", "2025-01-01")
    parameters = context.citizen_profile.copy()

    try:
        result = service.evaluate_law_output(
            law_id="participatiewet",
            output_name="huishouden_type",
            parameters=parameters,
            calculation_date=calculation_date,
        )
        context.result = result
        context.error = None
    except Exception as e:
        context.error = e
        context.result = None


@when("I calculate the norm_21_65 for participatiewet")  # type: ignore[misc]
def step_when_calculate_norm_21_65(context):
    """Calculate norm for 21-65 year olds via Article 21"""
    from engine.service import LawExecutionService

    service = LawExecutionService("regulation/nl")
    calculation_date = getattr(context, "calculation_date", "2025-01-01")
    parameters = context.citizen_profile.copy()

    try:
        result = service.evaluate_law_output(
            law_id="participatiewet",
            output_name="norm_21_65",
            parameters=parameters,
            calculation_date=calculation_date,
        )
        context.result = result
        context.error = None
    except Exception as e:
        context.error = e
        context.result = None


@when("I calculate the kostendelersnorm for participatiewet")  # type: ignore[misc]
def step_when_calculate_kostendelersnorm(context):
    """Calculate kostendelersnorm via Article 22a"""
    from engine.service import LawExecutionService

    service = LawExecutionService("regulation/nl")
    calculation_date = getattr(context, "calculation_date", "2025-01-01")
    parameters = context.citizen_profile.copy()

    try:
        result = service.evaluate_law_output(
            law_id="participatiewet",
            output_name="norm_kostendeler",
            parameters=parameters,
            calculation_date=calculation_date,
        )
        context.result = result
        context.error = None
    except Exception as e:
        context.error = e
        context.result = None


@when("I check heeft_recht_op_bijstand for participatiewet")  # type: ignore[misc]
def step_when_check_recht_op_bijstand(context):
    """Check right to benefits via Article 11"""
    from engine.service import LawExecutionService

    service = LawExecutionService("regulation/nl")
    calculation_date = getattr(context, "calculation_date", "2025-01-01")
    parameters = context.citizen_profile.copy()

    try:
        result = service.evaluate_law_output(
            law_id="participatiewet",
            output_name="heeft_recht_op_bijstand",
            parameters=parameters,
            calculation_date=calculation_date,
        )
        context.result = result
        context.error = None
    except Exception as e:
        context.error = e
        context.result = None


@when("I check is_uitgesloten_van_bijstand for participatiewet")  # type: ignore[misc]
def step_when_check_uitgesloten(context):
    """Check exclusion grounds via Article 13"""
    from engine.service import LawExecutionService

    service = LawExecutionService("regulation/nl")
    calculation_date = getattr(context, "calculation_date", "2025-01-01")
    parameters = context.citizen_profile.copy()

    try:
        result = service.evaluate_law_output(
            law_id="participatiewet",
            output_name="is_uitgesloten_van_bijstand",
            parameters=parameters,
            calculation_date=calculation_date,
        )
        context.result = result
        context.error = None
    except Exception as e:
        context.error = e
        context.result = None


@when("I check heeft_recht_op_algemene_bijstand for participatiewet")  # type: ignore[misc]
def step_when_check_recht_algemene_bijstand(context):
    """Check right to general benefits via Article 19"""
    from engine.service import LawExecutionService

    service = LawExecutionService("regulation/nl")
    calculation_date = getattr(context, "calculation_date", "2025-01-01")
    parameters = context.citizen_profile.copy()

    try:
        result = service.evaluate_law_output(
            law_id="participatiewet",
            output_name="heeft_recht_op_algemene_bijstand",
            parameters=parameters,
            calculation_date=calculation_date,
        )
        context.result = result
        context.error = None
    except Exception as e:
        context.error = e
        context.result = None


@when("I calculate the norm_inrichting for participatiewet")  # type: ignore[misc]
def step_when_calculate_norm_inrichting(context):
    """Calculate norm for institution stay via Article 23"""
    from engine.service import LawExecutionService

    service = LawExecutionService("regulation/nl")
    calculation_date = getattr(context, "calculation_date", "2025-01-01")
    parameters = context.citizen_profile.copy()

    try:
        result = service.evaluate_law_output(
            law_id="participatiewet",
            output_name="norm_inrichting",
            parameters=parameters,
            calculation_date=calculation_date,
        )
        context.result = result
        context.error = None
    except Exception as e:
        context.error = e
        context.result = None


@then('the huishouden_type is "{expected_type}"')  # type: ignore[misc]
def step_then_huishouden_type(context, expected_type):
    """Verify the household type"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    actual = result.output.get("huishouden_type")

    if actual != expected_type:
        raise AssertionError(
            f"Expected huishouden_type '{expected_type}', but got '{actual}'"
        )


@then("{output_name} is true")  # type: ignore[misc]
def step_then_output_is_true(context, output_name):
    """Verify a boolean output or resolved input is true"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    # Check both output and input (for values resolved from other articles)
    actual = result.output.get(output_name)
    if actual is None:
        actual = result.input.get(output_name)

    if actual is not True:
        raise AssertionError(
            f"Expected {output_name} to be true, but got {actual}"
        )


@then("{output_name} is false")  # type: ignore[misc]
def step_then_output_is_false(context, output_name):
    """Verify a boolean output or resolved input is false"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    # Check both output and input (for values resolved from other articles)
    actual = result.output.get(output_name)
    if actual is None:
        actual = result.input.get(output_name)

    if actual is not False:
        raise AssertionError(
            f"Expected {output_name} to be false, but got {actual}"
        )


@then('the norm_21_65 is "{amount}" eurocent')  # type: ignore[misc]
def step_then_norm_21_65(context, amount):
    """Verify the norm amount for 21-65"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    actual = result.output.get("norm_21_65")
    if isinstance(actual, float):
        actual = round(actual)
    expected = int(amount)

    if actual != expected:
        raise AssertionError(
            f"Expected norm_21_65 {expected}, but got {actual}"
        )


@then("{output_name} is {value:d}")  # type: ignore[misc]
def step_then_output_is_number(context, output_name, value):
    """Verify a numeric output"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    actual = result.output.get(output_name)
    if isinstance(actual, float):
        actual = round(actual)

    if actual != value:
        raise AssertionError(
            f"Expected {output_name} to be {value}, but got {actual}"
        )


@then('the norm_inrichting is "{amount}" eurocent')  # type: ignore[misc]
def step_then_norm_inrichting(context, amount):
    """Verify the institution norm amount"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    actual = result.output.get("norm_inrichting")
    if isinstance(actual, float):
        actual = round(actual)
    expected = int(amount)

    if actual != expected:
        raise AssertionError(
            f"Expected norm_inrichting {expected}, but got {actual}"
        )


@then('the bijstand_bedrag is "{amount}" eurocent')  # type: ignore[misc]
def step_then_bijstand_bedrag(context, amount):
    """Verify the calculated benefit amount"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    actual = result.output.get("bijstand_bedrag")
    if isinstance(actual, float):
        actual = round(actual)
    expected = int(amount)

    if actual != expected:
        raise AssertionError(
            f"Expected bijstand_bedrag {expected}, but got {actual}"
        )
