"""
Behave environment configuration

This file is run before and after test scenarios to set up and tear down
the test environment.
"""

import sys
import os

# Add project root to Python path so we can import engine module
project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if project_root not in sys.path:
    sys.path.insert(0, project_root)


def before_all(context):
    """Run before all tests"""
    # Set up any global test configuration here
    pass


def before_scenario(context, scenario):
    """Run before each scenario"""
    # Clean up context for each scenario
    if hasattr(context, "mock_service"):
        delattr(context, "mock_service")
    if hasattr(context, "bsn"):
        delattr(context, "bsn")
    if hasattr(context, "result"):
        delattr(context, "result")
    if hasattr(context, "error"):
        delattr(context, "error")


def after_scenario(context, scenario):
    """Run after each scenario"""
    # Clean up after each scenario
    pass


def after_all(context):
    """Run after all tests"""
    # Clean up global test resources
    pass
