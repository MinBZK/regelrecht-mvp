"""Preamble element handlers for aanhef (preamble) sections.

These handlers process elements found in the aanhef (preamble) of Dutch laws,
including the royal introduction (wij), considerations (considerans),
and proclamation (afkondiging).
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from harvester.parsers.registry.protocols import (
    ElementType,
    ParseContext,
    ParseResult,
    RecurseFn,
    extract_text_with_tail,
)

if TYPE_CHECKING:
    from lxml import etree


@dataclass
class WijHandler:
    """Handler for <wij> (royal introduction) elements.

    Contains the formal royal introduction text like
    "Wij Beatrix, bij de gratie Gods, Koningin der Nederlanden..."
    """

    @property
    def element_type(self) -> ElementType:
        return ElementType.INLINE

    def can_handle(self, elem: etree._Element, context: ParseContext) -> bool:
        return True

    def handle(
        self,
        elem: etree._Element,
        context: ParseContext,
        recurse: RecurseFn,
    ) -> ParseResult:
        text = elem.text.strip() if elem.text else ""
        return ParseResult(text=text)


@dataclass
class ConsideransHandler:
    """Handler for <considerans> (considerations) elements.

    Contains the considerations/recitals of the law. Processes all
    <considerans.al> child elements.
    """

    @property
    def element_type(self) -> ElementType:
        return ElementType.STRUCTURAL

    def can_handle(self, elem: etree._Element, context: ParseContext) -> bool:
        return True

    def handle(
        self,
        elem: etree._Element,
        context: ParseContext,
        recurse: RecurseFn,
    ) -> ParseResult:
        parts: list[str] = []

        for al in elem.findall(".//considerans.al"):
            result = recurse(al, context)
            if result.text:
                parts.append(result.text)

        return ParseResult(text="\n\n".join(parts))


@dataclass
class ConsideransAlHandler:
    """Handler for <considerans.al> (consideration paragraph) elements.

    Individual paragraph within the considerations section.
    Processes inline content including references.
    """

    @property
    def element_type(self) -> ElementType:
        return ElementType.INLINE

    def can_handle(self, elem: etree._Element, context: ParseContext) -> bool:
        return True

    def handle(
        self,
        elem: etree._Element,
        context: ParseContext,
        recurse: RecurseFn,
    ) -> ParseResult:
        return ParseResult(text=extract_text_with_tail(elem, context, recurse))


@dataclass
class AfkondigingHandler:
    """Handler for <afkondiging> (proclamation) elements.

    Contains the formal proclamation text. Processes all <al> child elements.
    """

    @property
    def element_type(self) -> ElementType:
        return ElementType.STRUCTURAL

    def can_handle(self, elem: etree._Element, context: ParseContext) -> bool:
        return True

    def handle(
        self,
        elem: etree._Element,
        context: ParseContext,
        recurse: RecurseFn,
    ) -> ParseResult:
        parts: list[str] = []

        for al in elem.findall(".//al"):
            result = recurse(al, context)
            if result.text:
                parts.append(result.text)

        return ParseResult(text="\n\n".join(parts))


@dataclass
class AanhefHandler:
    """Handler for <aanhef> (preamble) elements.

    The preamble contains the royal introduction, considerations,
    and proclamation. Processes all child elements in order.
    """

    @property
    def element_type(self) -> ElementType:
        return ElementType.STRUCTURAL

    def can_handle(self, elem: etree._Element, context: ParseContext) -> bool:
        return True

    def handle(
        self,
        elem: etree._Element,
        context: ParseContext,
        recurse: RecurseFn,
    ) -> ParseResult:
        parts: list[str] = []

        # Process wij
        wij_elem = elem.find("wij")
        if wij_elem is not None:
            result = recurse(wij_elem, context)
            if result.text:
                parts.append(result.text)

        # Process considerans
        considerans_elem = elem.find("considerans")
        if considerans_elem is not None:
            result = recurse(considerans_elem, context)
            if result.text:
                parts.append(result.text)

        # Process afkondiging
        afkondiging_elem = elem.find("afkondiging")
        if afkondiging_elem is not None:
            result = recurse(afkondiging_elem, context)
            if result.text:
                parts.append(result.text)

        return ParseResult(text="\n\n".join(parts))
