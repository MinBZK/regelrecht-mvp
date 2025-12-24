"""Configuration for Dutch law hierarchy."""

from harvester.parsers.splitting.protocols import ElementSpec
from harvester.parsers.splitting.registry import HierarchyRegistry


def create_dutch_law_hierarchy() -> HierarchyRegistry:
    """Create hierarchy registry for Dutch law structure.

    The hierarchy represents the structural nesting of Dutch legal documents:

    ```
    artikel
    ├── lid (numbered paragraph, e.g. "1.")
    │   ├── al (text paragraph)
    │   └── lijst (list)
    │       └── li (list item, e.g. "a.")
    │           ├── al
    │           └── lijst (nested)
    │               └── li (e.g. "1°")
    │                   └── al
    ```

    Returns:
        Configured HierarchyRegistry for Dutch laws
    """
    registry = HierarchyRegistry()

    # Artikel: top-level article element
    registry.register(
        ElementSpec(
            tag="artikel",
            children=["lid", "lijst"],  # Structural children only
            number_source="kop/nr",
            content_tags=["al"],
            is_split_point=True,
            skip_for_number=["kop", "meta-data"],
        )
    )

    # Lid: numbered paragraph within artikel
    registry.register(
        ElementSpec(
            tag="lid",
            children=["lijst"],  # Structural children only
            number_source="lidnr",
            content_tags=["al"],
            is_split_point=True,
            skip_for_number=["lidnr", "meta-data"],
        )
    )

    # Lijst: list container (not a split point itself)
    registry.register(
        ElementSpec(
            tag="lijst",
            children=["li"],
            content_tags=[],
            is_split_point=False,
        )
    )

    # Li: list item
    registry.register(
        ElementSpec(
            tag="li",
            children=["lijst"],  # Structural children only
            number_source="li.nr",
            content_tags=["al"],
            is_split_point=True,
            skip_for_number=["li.nr"],
        )
    )

    return registry
