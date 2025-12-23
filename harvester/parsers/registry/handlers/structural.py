"""Structural element handlers for container elements.

These handlers process structural elements that contain other elements,
such as lid (paragraph), lijst (list), and li (list item).
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from harvester.parsers.registry.protocols import (
    ElementType,
    ParseContext,
    ParseResult,
    RecurseFn,
)
from harvester.parsers.registry.registry import get_tag_name

if TYPE_CHECKING:
    from lxml import etree


@dataclass
class LidnrHandler:
    """Handler for <lidnr> (paragraph number) elements.

    Returns the paragraph number text.
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
        text = elem.text.strip() if elem.text else ""
        return ParseResult(text=text)


@dataclass
class LiNrHandler:
    """Handler for <li.nr> (list item number) elements.

    Returns the list item marker (a., b., 1., etc.) without trailing period.
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
        nr = elem.text.strip() if elem.text else ""
        if nr.endswith("."):
            nr = nr[:-1]
        return ParseResult(text=nr)


@dataclass
class LidHandler:
    """Handler for <lid> (paragraph/subdivision) elements.

    Extracts text from a lid, processing all child <al> elements.
    Skips lidnr and meta-data elements.
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

        for child in elem:
            child_tag = get_tag_name(child)
            # Skip lidnr (handled separately for numbering)
            if child_tag in {"lidnr", "meta-data"}:
                continue

            result = recurse(child, context)
            if result.text:
                parts.append(result.text)

        return ParseResult(text=" ".join(parts).strip())


@dataclass
class LijstHandler:
    """Handler for <lijst> (list) elements.

    Processes each <li> child element. Handles both marked and
    unmarked (ongemarkeerd) list types.
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
        items: list[str] = []

        for li in elem.findall("li"):
            result = recurse(li, context)
            if result.text:
                items.append(result.text)

        return ParseResult(text="\n".join(items))


@dataclass
class LiHandler:
    """Handler for <li> (list item) elements.

    Extracts text from list items, processing child <al> elements.
    Skips li.nr (handled separately for numbering).
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

        for child in elem:
            child_tag = get_tag_name(child)
            # Skip li.nr (handled separately for numbering)
            if child_tag == "li.nr":
                continue

            result = recurse(child, context)
            if result.text:
                parts.append(result.text)

        return ParseResult(text=" ".join(parts).strip())


@dataclass
class SkipHandler:
    """Handler that skips elements (returns empty text).

    Used for elements that should not contribute to text output.
    """

    @property
    def element_type(self) -> ElementType:
        return ElementType.SKIP

    def can_handle(self, elem: etree._Element, context: ParseContext) -> bool:
        return True

    def handle(
        self,
        elem: etree._Element,
        context: ParseContext,
        recurse: RecurseFn,
    ) -> ParseResult:
        return ParseResult(text="")


@dataclass
class PassthroughHandler:
    """Handler that extracts text from element and all children.

    Used for container elements that should contribute their text content.
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
        parts: list[str] = []

        if elem.text:
            parts.append(elem.text)

        for child in elem:
            result = recurse(child, context)
            if result.text:
                parts.append(result.text)

            if child.tail:
                parts.append(child.tail)

        return ParseResult(text="".join(parts).strip())


# Convenience aliases
KopHandler = SkipHandler
PlaatjeHandler = SkipHandler
IllustratieHandler = SkipHandler
