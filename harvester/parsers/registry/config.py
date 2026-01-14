"""Registry configuration for BWB XML parsing."""

from harvester.parsers.registry.engine import ParseEngine
from harvester.parsers.registry.handlers.inline import (
    AlHandler,
    ExtrefHandler,
    IntrefHandler,
    NadrukHandler,
    RedactieHandler,
)
from harvester.parsers.registry.handlers.preamble import (
    AanhefHandler,
    AfkondigingHandler,
    ConsideransAlHandler,
    ConsideransHandler,
    WijHandler,
)
from harvester.parsers.registry.handlers.structural import (
    LidHandler,
    LidnrHandler,
    LiHandler,
    LijstHandler,
    LiNrHandler,
    PassthroughHandler,
    SkipHandler,
)
from harvester.parsers.registry.registry import ElementRegistry

# Tags to skip completely (contain metadata, not content)
SKIP_TAGS = {
    "meta-data",
    "jcis",
    "jci",
    "brondata",
    "kop",
    # Image/illustration elements (skip - not text content)
    "plaatje",
    "illustratie",
    # BWB metadata elements
    "bwb-inputbestand",
    "bwb-wijzigingen",
    # Editorial/rectification metadata
    "redactionele-correcties",
    "redactionele-correctie",
    "rectificatie",
    # Publication metadata
    "publicatie",
    "publicatiejaar",
    "publicatienr",
    "uitgiftedatum",
    "ondertekeningsdatum",
    "inwerkingtreding",
    "inwerkingtreding.datum",
    "terugwerkend.datum",
    "oorspronkelijk",
    "uitgifte",
    "opmerkingen-inhoud",
    "dossierref",
    "juncto",
}


def create_content_registry() -> ElementRegistry:
    """Create registry configured for content parsing.

    Returns:
        ElementRegistry with all content handlers registered
    """
    registry = ElementRegistry()

    # Skip metadata elements
    registry.skip(*SKIP_TAGS)

    # Register inline handlers
    registry.register("al", AlHandler())
    registry.register("nadruk", NadrukHandler())
    registry.register("extref", ExtrefHandler())
    registry.register("intref", IntrefHandler())
    registry.register("redactie", RedactieHandler())

    # Register structural handlers
    registry.register("lid", LidHandler())
    registry.register("lidnr", LidnrHandler())
    registry.register("lijst", LijstHandler())
    registry.register("li", LiHandler())
    registry.register("li.nr", LiNrHandler())

    # Register preamble handlers
    registry.register("aanhef", AanhefHandler())
    registry.register("wij", WijHandler())
    registry.register("considerans", ConsideransHandler())
    registry.register("considerans.al", ConsideransAlHandler())
    registry.register("afkondiging", AfkondigingHandler())

    # Register passthrough handlers for structural containers
    # These extract text from their children
    passthrough = PassthroughHandler()
    registry.register("slotformulering", passthrough)
    registry.register("gegeven", passthrough)
    registry.register("dagtekening", passthrough)
    registry.register("plaats", passthrough)
    registry.register("datum", passthrough)
    registry.register("naam", passthrough)
    registry.register("achternaam", passthrough)
    registry.register("functie", passthrough)
    registry.register("kenmerk", passthrough)

    # Skip handlers for structural elements that don't contribute text
    skip = SkipHandler()
    registry.register("ondertekening", skip)
    registry.register("wetsluiting", skip)
    registry.register("koning", skip)

    return registry


def create_content_engine() -> ParseEngine:
    """Create a ParseEngine configured for content parsing.

    Returns:
        ParseEngine with content registry
    """
    registry = create_content_registry()
    return ParseEngine(registry)
