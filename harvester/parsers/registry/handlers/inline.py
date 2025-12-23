"""Inline element handlers for text-level elements.

These handlers process elements that appear inline within text,
such as emphasis (nadruk), external references (extref), and
internal references (intref).
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import TYPE_CHECKING

from harvester.parsers.registry.protocols import (
    ElementType,
    ParseContext,
    ParseResult,
    RecurseFn,
)

if TYPE_CHECKING:
    from lxml import etree


def convert_jci_to_url(jci_ref: str) -> str:
    """Convert JCI reference to wetten.overheid.nl URL.

    Args:
        jci_ref: JCI reference like "jci1.3:c:BWBR0018450&artikel=1"

    Returns:
        URL like "https://wetten.overheid.nl/BWBR0018450#Artikel1"
    """
    bwb_match = re.search(r"BWBR\d+", jci_ref)
    artikel_match = re.search(r"artikel=(\d+\w*)", jci_ref)

    if bwb_match:
        bwb_id = bwb_match.group(0)
        if artikel_match:
            artikel = artikel_match.group(1)
            return f"https://wetten.overheid.nl/{bwb_id}#Artikel{artikel}"
        return f"https://wetten.overheid.nl/{bwb_id}"

    return jci_ref


@dataclass
class NadrukHandler:
    """Handler for <nadruk> (emphasis) elements.

    Converts emphasis to markdown: **text** for bold (type="vet"),
    *text* for italic (default).
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
        text = elem.text or ""
        nadruk_type = elem.get("type", "")

        if nadruk_type == "vet":
            formatted = f"**{text}**"
        else:
            formatted = f"*{text}*"

        return ParseResult(text=formatted)


@dataclass
class ExtrefHandler:
    """Handler for <extref> (external reference) elements.

    Converts external references to markdown links using reference-style
    formatting when a collector is available, or inline links otherwise.
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
        ref_text = elem.text or ""
        collector = context.collector

        if collector:
            ref_id = collector.add_reference(elem, is_internal=False)
            if ref_id:
                return ParseResult(text=f"[{ref_text}][{ref_id}]")
            return ParseResult(text=ref_text)

        # Fallback to inline link when no collector
        ref_url = elem.get("doc", "")
        if ref_url:
            converted_url = convert_jci_to_url(ref_url)
            return ParseResult(text=f"[{ref_text}]({converted_url})")

        return ParseResult(text=ref_text)


@dataclass
class IntrefHandler:
    """Handler for <intref> (internal reference) elements.

    Converts internal references to markdown links using reference-style
    formatting when a collector is available, or inline links otherwise.
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
        ref_text = elem.text or ""
        collector = context.collector

        if collector:
            ref_id = collector.add_reference(elem, is_internal=True)
            if ref_id:
                return ParseResult(text=f"[{ref_text}][{ref_id}]")
            return ParseResult(text=ref_text)

        # Fallback to inline link when no collector
        ref_url = elem.get("doc", "")
        if ref_url:
            converted_url = convert_jci_to_url(ref_url)
            return ParseResult(text=f"[{ref_text}]({converted_url})")

        return ParseResult(text=ref_text)


@dataclass
class AlHandler:
    """Handler for <al> (paragraph/alinea) elements.

    Extracts inline text including child elements like extref, intref,
    and nadruk. This is the main text container element.
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


@dataclass
class RedactieHandler:
    """Handler for <redactie> (editorial note) elements.

    Editorial notes indicate text that has been modified or replaced.
    The type attribute indicates the kind of editorial change.
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
        # Extract text content, recursing into children
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
