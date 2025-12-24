"""Parser for consolidated legal text (content) files."""

from __future__ import annotations

from dataclasses import dataclass, field

import requests
from lxml import etree

from harvester.config import BWB_REPOSITORY_URL, HTTP_TIMEOUT
from harvester.models import Article, Reference
from harvester.parsers.reference_parser import parse_extref, parse_intref


# Tags to skip when extracting text (contain metadata, not content)
SKIP_TAGS = {"meta-data", "kop", "jcis", "jci", "brondata"}


@dataclass
class ReferenceCollector:
    """Collects references during text extraction."""

    references: list[Reference] = field(default_factory=list)
    _counter: int = field(default=0, repr=False)

    def add_reference(self, elem: etree._Element, is_internal: bool = True) -> str:
        """Add a reference and return the markdown reference ID.

        Args:
            elem: The intref or extref XML element
            is_internal: True for intref, False for extref

        Returns:
            Reference ID like "ref1" for use in markdown
        """
        self._counter += 1
        ref_id = f"ref{self._counter}"

        parse_fn = parse_intref if is_internal else parse_extref
        ref = parse_fn(elem, ref_id)

        if ref:
            self.references.append(ref)
            return ref_id
        return ""

    def get_reference_definitions(self) -> str:
        """Generate markdown reference definitions.

        Returns:
            Markdown reference definitions like:
            [ref1]: https://wetten.overheid.nl/BWBR0018451#Artikel4
            [ref2]: https://wetten.overheid.nl/BWBR0018450#Artikel1
        """
        if not self.references:
            return ""

        lines = []
        for ref in self.references:
            url = ref.to_wetten_url()
            lines.append(f"[{ref.id}]: {url}")
        return "\n".join(lines)


def download_content(bwb_id: str, date: str) -> etree._Element:
    """Download consolidated legal text file for a specific date.

    Uses the consolidated version URL pattern with _0 suffix to get the
    most recent version of the law as of the specified date.

    Args:
        bwb_id: The BWB identifier (e.g., "BWBR0018451")
        date: The effective date in YYYY-MM-DD format

    Returns:
        Parsed XML element tree

    Raises:
        requests.HTTPError: If download fails
    """
    # Use _0 suffix for consolidated (geconsolideerde) version
    url = f"{BWB_REPOSITORY_URL}/{bwb_id}/{date}_0/xml/{bwb_id}_{date}_0.xml"
    response = requests.get(url, timeout=HTTP_TIMEOUT, allow_redirects=True)
    try:
        response.raise_for_status()
    except requests.HTTPError as e:
        raise requests.HTTPError(
            f"Failed to download content for {bwb_id} at date {date}: {e}"
        ) from e
    return etree.fromstring(response.content)


def parse_articles_split(
    content_tree: etree._Element,
    bwb_id: str,
    date: str,
) -> list[Article]:
    """Parse articles, splitting to lowest-level components.

    This function splits articles into their constituent parts (leden,
    onderdelen) using dot-notation numbering. For example, artikel 1,
    lid 1, onderdeel a becomes "1.1.a".

    Args:
        content_tree: Parsed content XML element
        bwb_id: The BWB identifier
        date: The effective date in YYYY-MM-DD format

    Returns:
        List of Article objects, one per lowest-level component
    """
    from harvester.parsers.article_splitter import build_articles_from_content

    return build_articles_from_content(content_tree, bwb_id, date)
