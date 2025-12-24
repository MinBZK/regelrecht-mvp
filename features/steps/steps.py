"""
Step definitions for healthcare allowance (zorgtoeslag) and bijstand features
"""

import sys
import os

# Add the steps directory to the path for imports
sys.path.insert(0, os.path.dirname(__file__))

# Get project root for absolute paths (works from any working directory)
_steps_dir = os.path.dirname(os.path.abspath(__file__))
_features_dir = os.path.dirname(_steps_dir)
# Handle both features/steps/ and regulation/nl/steps/ (symlink)
if os.path.basename(_features_dir) == "nl":
    # Running from regulation/nl/steps -> go up to project root
    PROJECT_ROOT = os.path.dirname(os.path.dirname(_features_dir))
else:
    # Running from features/steps -> go up 2 levels to project root
    PROJECT_ROOT = os.path.dirname(os.path.dirname(_steps_dir))

# Add project root to Python path for engine imports
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

REGULATION_DIR = os.path.join(PROJECT_ROOT, "regulation", "nl")

from behave import given, when, then  # noqa: E402  # type: ignore[import-untyped]
from mock_data_service import MockDataService  # noqa: E402  # type: ignore[import-not-found]


def print_execution_trace(result, title: str = "Execution Trace") -> None:
    """
    Print the execution trace from an ArticleResult if available.

    Args:
        result: ArticleResult with optional path (PathNode)
        title: Title to display above the trace
    """
    if result is None:
        return

    if not hasattr(result, "path") or result.path is None:
        return

    print(f"\n{'=' * 60}")
    print(f"** {title} **")
    print(f"   Law: {result.law_id} | Article: {result.article_number}")
    print(f"{'=' * 60}")
    print(result.path.render())
    print(f"{'=' * 60}\n")


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
    service = LawExecutionService(REGULATION_DIR)

    # Get calculation date
    calculation_date = getattr(context, "calculation_date", "2024-01-01")

    # Build parameters from citizen data
    parameters = context.citizen_data.copy()

    # Ensure BSN is present (generate test BSN if not provided)
    if "bsn" not in parameters:
        parameters["bsn"] = "123456789"

    # Set gedragscategorie in uitvoerder data (not as direct parameter)
    # The engine will resolve this from uitvoerder data
    gemeente_code = parameters.get("gemeente_code", "")
    gedragscategorie = parameters.pop("gedragscategorie", 0)
    LawExecutionService.set_gedragscategorie(
        parameters["bsn"], gemeente_code, gedragscategorie
    )

    try:
        # Call Article 43 - use URI without field to get all outputs
        uri = "regelrecht://participatiewet/heeft_recht_op_bijstand"
        result = service.evaluate_uri(
            uri=uri,
            parameters=parameters,
            calculation_date=calculation_date,
        )
        context.result = result
        context.error = None

        # Print execution trace
        print_execution_trace(result, "Bijstand Execution Trace")
    except Exception as e:
        context.error = e
        context.result = None
    finally:
        # Clean up uitvoerder data
        LawExecutionService.clear_uitvoerder_data()


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
    """Execute the healthcare allowance law with mock data via data sources"""
    from datetime import datetime
    from engine.service import LawExecutionService
    from engine.data_sources import DictDataSource

    # Create service
    service = LawExecutionService(REGULATION_DIR)

    # Convert mock data to data sources
    if hasattr(context, "mock_service"):
        mock = context.mock_service

        # Helper to convert string values to appropriate types
        def convert_value(v):
            if v == "null" or v is None:
                return None
            if v == "true":
                return True
            if v == "false":
                return False
            # Try to convert to int or float
            try:
                if "." in str(v):
                    return float(v)
                return int(v)
            except (ValueError, TypeError):
                return v

        # Create a single data source per service/datasource combination
        for service_name, datasources in mock.services.items():
            for datasource_name, bsn_data in datasources.items():
                source_name = f"{service_name.lower()}_{datasource_name}"
                source = DictDataSource(name=source_name, priority=100)

                for bsn, record in bsn_data.items():
                    # Convert string values to proper types
                    converted_record = {k: convert_value(v) for k, v in record.items()}
                    source.store(bsn, converted_record)

                service.add_data_source(source)

        # Add derived fields that zorgtoeslag needs directly
        # (inputs that have NO source spec in the zorgtoeslag law)
        derived = DictDataSource(name="derived", priority=200)

        for bsn in [context.bsn]:
            derived_record = {}

            # LEEFTIJD - from personal_data (zorgtoeslag needs this directly)
            personal = mock.services.get("RVIG", {}).get("personal_data", {}).get(bsn)
            if personal and "geboortedatum" in personal:
                birth_date = datetime.strptime(
                    personal["geboortedatum"], "%Y-%m-%d"
                ).date()
                today = datetime.now().date()
                age = (
                    today.year
                    - birth_date.year
                    - ((today.month, today.day) < (birth_date.month, birth_date.day))
                )
                derived_record["leeftijd"] = age

            # HEEFT_VERZEKERING - from insurance data
            insurance = mock.services.get("RVZ", {}).get("insurance", {}).get(bsn)
            derived_record["heeft_verzekering"] = (
                insurance.get("polis_status") == "ACTIEF" if insurance else False
            )

            # HEEFT_VERDRAGSVERZEKERING, IS_GEDETINEERD, IS_FORENSISCH - defaults
            derived_record["heeft_verdragsverzekering"] = False
            derived_record["is_gedetineerd"] = False
            derived_record["is_forensisch"] = False

            # HEEFT_PARTNER - from relationship data
            relationship = (
                mock.services.get("RVIG", {}).get("relationship_data", {}).get(bsn)
            )
            derived_record["heeft_partner"] = (
                relationship.get("partnerschap_type") != "GEEN"
                if relationship
                else False
            )

            # PARTNER_INKOMEN - 0 for tests without partner
            derived_record["partner_inkomen"] = 0

            derived.store(bsn, derived_record)

        service.add_data_source(derived)

    # Execute the law
    parameters = {"bsn": context.bsn}

    try:
        # Call the zorgtoeslag calculation output
        result = service.evaluate_law_output(
            law_id="wet_op_de_zorgtoeslag",
            output_name="hoogte_toeslag",
            parameters=parameters,
        )
        context.result = result

        # Print execution trace
        print_execution_trace(result, "Healthcare Allowance Execution Trace")
    except Exception as e:
        context.error = e
        raise


@when("I request the standard premium for year {year:d}")  # type: ignore[misc]
def step_when_request_standard_premium(context, year):
    """Request the standard premium for a specific year"""
    from engine.service import LawExecutionService

    # Create service
    service = LawExecutionService(REGULATION_DIR)

    # Set calculation_date to match the year
    calculation_date = f"{year}-01-01"

    try:
        # Call the get_standaardpremie output (from regeling_standaardpremie)
        result = service.evaluate_law_output(
            law_id="regeling_standaardpremie",
            output_name="standaardpremie",
            parameters={},
            calculation_date=calculation_date,
        )
        context.result = result

        # Print execution trace
        print_execution_trace(result, "Standard Premium Execution Trace")
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
    if "hoogte_toeslag" in result.output:
        actual_amount_eurocent = result.output["hoogte_toeslag"]
        actual_amount_euro = actual_amount_eurocent / 100
    else:
        raise AssertionError(f"No 'hoogte_toeslag' in outputs: {result.output}")

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

    service = LawExecutionService(REGULATION_DIR)
    calculation_date = getattr(context, "calculation_date", "2024-01-01")
    parameters = context.query_data.copy()

    try:
        # Use URI without specific field to get all outputs
        uri = f"regelrecht://{law_id}/minimale_afstand_cm"
        result = service.evaluate_uri(
            uri=uri,
            parameters=parameters,
            calculation_date=calculation_date,
        )
        context.result = result
        context.error = None

        # Print execution trace
        print_execution_trace(result, "Erfgrensbeplanting Execution Trace")
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


# === Nederlandse zorgtoeslag step definitions ===


@given('de datum is "{date}"')  # type: ignore[misc]
def step_given_datum(context, date):
    """Set the calculation date (Dutch)"""
    context.calculation_date = date


@given('een persoon met BSN "{bsn}"')  # type: ignore[misc]
def step_given_persoon_met_bsn(context, bsn):
    """Set the BSN for the current person (Dutch)"""
    context.bsn = bsn
    if not hasattr(context, "mock_service"):
        context.mock_service = MockDataService()


@given("de volgende {service} {table_name} gegevens:")  # type: ignore[misc]
def step_given_dutch_service_data(context, service, table_name):
    """
    Generic Dutch step to store service data from table

    Maps Dutch table names to English equivalents:
    - personen -> personal_data
    - relaties -> relationship_data
    - verzekeringen -> insurance
    - box1, box2, box3 -> box1, box2, box3
    - detenties -> detention
    """
    if not hasattr(context, "mock_service"):
        context.mock_service = MockDataService()

    # Map Dutch table names to English
    table_mapping = {
        "personen": "personal_data",
        "relaties": "relationship_data",
        "verzekeringen": "insurance",
        "detenties": "detention",
        "inschrijvingen": "enrollment",
        "studiefinanciering": "student_finance",
    }
    datasource = table_mapping.get(table_name, table_name)

    for row in context.table:
        # Convert all values to appropriate types
        data = {"bsn": row["bsn"]}

        for key in row.headings:
            if key != "bsn":
                value = row[key]
                if value == "null":
                    data[key] = None
                else:
                    try:
                        data[key] = float(value)
                    except ValueError:
                        data[key] = value

        context.mock_service.store_data(service, datasource, data)
        context.bsn = row["bsn"]


@when("de zorgtoeslagwet wordt uitgevoerd door TOESLAGEN")  # type: ignore[misc]
def step_when_zorgtoeslagwet_uitgevoerd(context):
    """Execute the zorgtoeslag law (Dutch)"""
    from engine.service import LawExecutionService
    from engine.data_sources import DictDataSource

    # Create service
    service = LawExecutionService(REGULATION_DIR)

    # Get calculation date
    calculation_date = getattr(context, "calculation_date", "2025-01-01")

    # Convert mock data to data sources
    if hasattr(context, "mock_service"):
        mock = context.mock_service

        def convert_value(v):
            if v == "null" or v is None:
                return None
            if v == "true":
                return True
            if v == "false":
                return False
            try:
                if "." in str(v):
                    return float(v)
                return int(v)
            except (ValueError, TypeError):
                return v

        # Create a single data source per service/datasource combination
        for service_name, datasources in mock.services.items():
            for datasource_name, bsn_data in datasources.items():
                source_name = f"{service_name.lower()}_{datasource_name}"
                source = DictDataSource(name=source_name, priority=100)

                for bsn, record in bsn_data.items():
                    converted_record = {k: convert_value(v) for k, v in record.items()}
                    source.store(bsn, converted_record)

                service.add_data_source(source)

        # Add derived fields needed by zorgtoeslag
        derived = DictDataSource(name="derived", priority=200)

        for bsn in [context.bsn]:
            derived_record = {}

            # Get personal data
            personal = mock.services.get("RvIG", {}).get("personal_data", {}).get(bsn)

            # geboortedatum - needed for leeftijd calculation in BRP wet
            if personal and "geboortedatum" in personal:
                derived_record["geboortedatum"] = personal["geboortedatum"]

            # PARTNERSCHAP_TYPE - needed for heeft_partner calculation in BRP wet
            relationship = (
                mock.services.get("RvIG", {}).get("relationship_data", {}).get(bsn)
            )
            if relationship:
                derived_record["partnerschap_type"] = relationship.get(
                    "partnerschap_type", "GEEN"
                )
            else:
                derived_record["partnerschap_type"] = "GEEN"

            # POLIS_STATUS - needed for is_verzekerde calculation in Zvw
            insurance = mock.services.get("RVZ", {}).get("insurance", {}).get(bsn)
            if insurance:
                derived_record["polis_status"] = insurance.get(
                    "polis_status", "INACTIEF"
                )
            else:
                derived_record["polis_status"] = "INACTIEF"

            # Income fields for wet_inkomstenbelasting_2001
            box1 = mock.services.get("BELASTINGDIENST", {}).get("box1", {}).get(bsn, {})
            derived_record["loon_uit_dienstbetrekking"] = box1.get(
                "loon_uit_dienstbetrekking", 0
            )
            derived_record["uitkeringen_en_pensioenen"] = box1.get(
                "uitkeringen_en_pensioenen", 0
            )
            derived_record["winst_uit_onderneming"] = box1.get(
                "winst_uit_onderneming", 0
            )
            derived_record["resultaat_overige_werkzaamheden"] = box1.get(
                "resultaat_overige_werkzaamheden", 0
            )
            derived_record["eigen_woning"] = box1.get("eigen_woning", 0)

            # Box 2 fields (aanmerkelijk belang)
            box2 = mock.services.get("BELASTINGDIENST", {}).get("box2", {}).get(bsn, {})
            derived_record["reguliere_voordelen"] = box2.get("reguliere_voordelen", 0)
            derived_record["vervreemdingsvoordelen"] = box2.get(
                "vervreemdingsvoordelen", 0
            )

            # Box 3 fields (vermogen)
            box3 = mock.services.get("BELASTINGDIENST", {}).get("box3", {}).get(bsn, {})
            derived_record["spaargeld"] = box3.get("spaargeld", 0)
            derived_record["beleggingen"] = box3.get("beleggingen", 0)
            derived_record["onroerend_goed"] = box3.get("onroerend_goed", 0)
            derived_record["schulden"] = box3.get("schulden", 0)

            # Partner fields (default to 0)
            derived_record["partner_inkomen"] = 0
            derived_record["partner_vermogen"] = 0

            derived.store(bsn, derived_record)

        service.add_data_source(derived)

    # Execute the law
    parameters = {"bsn": context.bsn}

    try:
        result = service.evaluate_law_output(
            law_id="wet_op_de_zorgtoeslag",
            output_name="hoogte_toeslag",
            parameters=parameters,
            calculation_date=calculation_date,
        )
        context.result = result
        context.error = None

        # Print execution trace
        print_execution_trace(result, "Zorgtoeslag Execution Trace")
    except Exception as e:
        context.error = e
        context.result = None


@then("is niet voldaan aan de voorwaarden")  # type: ignore[misc]
def step_then_niet_voldaan(context):
    """Verify that the applicant does not meet the requirements (Dutch)"""
    if context.error:
        # Check if error indicates no right
        if "geen recht" in str(context.error).lower():
            return
        raise AssertionError(f"Unexpected error: {context.error}")

    if context.result is None:
        return  # No result means requirements not met

    # Check if the output indicates no right
    result = context.result
    hoogte = result.output.get("hoogte_toeslag", 0)

    # For zorgtoeslag: if the amount is 0 or there's a rejection reason, not met
    if hoogte == 0:
        return

    # Check for is_verzekerde output
    if "is_verzekerde_zorgtoeslag" in result.output:
        if not result.output["is_verzekerde_zorgtoeslag"]:
            return

    raise AssertionError(
        f"Expected requirements not met, but got hoogte_toeslag: {hoogte}"
    )


@then("heeft de persoon recht op zorgtoeslag")  # type: ignore[misc]
def step_then_heeft_recht(context):
    """Verify that the applicant has the right to healthcare allowance (Dutch)"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    hoogte = result.output.get("hoogte_toeslag", 0)

    if hoogte <= 0:
        raise AssertionError(
            f"Expected person to have right to zorgtoeslag, but hoogte_toeslag is {hoogte}"
        )


@then('is het toeslagbedrag "{amount}" euro')  # type: ignore[misc]
def step_then_toeslagbedrag(context, amount):
    """Verify the allowance amount in euro (Dutch)"""
    if context.error:
        raise AssertionError(f"Execution failed: {context.error}")

    result = context.result
    actual_eurocent = result.output.get("hoogte_toeslag", 0)
    actual_euro = actual_eurocent / 100

    expected_euro = float(amount)

    # Allow small rounding difference
    if abs(actual_euro - expected_euro) > 0.02:
        raise AssertionError(
            f"Expected toeslagbedrag €{expected_euro:.2f}, but got €{actual_euro:.2f}"
        )
