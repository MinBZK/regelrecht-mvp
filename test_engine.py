"""
Simple test script to verify the engine works
"""

from engine.service import LawExecutionService
from engine.logging_config import setup_logging
import logging

# Setup logging
setup_logging(logging.DEBUG)

# Initialize service
print("Initializing law execution service...")
service = LawExecutionService("regulation/nl")

# List available laws
print(f"\nLoaded laws: {service.list_available_laws()}")
print(f"\nAvailable endpoints: {service.list_available_endpoints()}")

# Get info about zorgtoeslagwet
print("\n" + "=" * 80)
print("Zorgtoeslagwet Info:")
print("=" * 80)
info = service.get_law_info("zorgtoeslagwet")
for key, value in info.items():
    print(f"  {key}: {value}")

# Get info about regeling_standaardpremie
print("\n" + "=" * 80)
print("Regeling Standaardpremie Info:")
print("=" * 80)
info = service.get_law_info("regeling_standaardpremie")
for key, value in info.items():
    print(f"  {key}: {value}")

# Test evaluating the standaardpremie endpoint
print("\n" + "=" * 80)
print("Testing regeling_standaardpremie/standaardpremie endpoint:")
print("=" * 80)

try:
    result = service.evaluate_uri(
        uri="regelrecht://regeling_standaardpremie/standaardpremie#standaardpremie",
        parameters={},
        reference_date="2025-01-01",
    )

    print("\nResult:")
    print(f"  Article: {result.article_number}")
    print(f"  Law: {result.law_id}")
    print(f"  Outputs: {result.output}")

    # Format the amount
    standaardpremie = result.output.get("standaardpremie")
    if standaardpremie:
        euros = standaardpremie / 100
        print(f"  Standaardpremie: €{euros:,.2f}")

except Exception as e:
    print(f"Error: {e}")
    import traceback

    traceback.print_exc()

print("\n" + "=" * 80)
print("Test complete!")
print("=" * 80)
