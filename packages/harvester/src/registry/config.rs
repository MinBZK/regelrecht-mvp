//! Registry configuration for Dutch law content parsing.

use super::handlers::{
    AanhefHandler, AfkondigingHandler, AlHandler, ConsideransAlHandler, ConsideransHandler,
    ExtrefHandler, IntrefHandler, LiHandler, LiNrHandler, LidHandler, LidnrHandler, LijstHandler,
    NadrukHandler, PassthroughHandler, RedactieHandler, SkipHandler, WijHandler,
};
use super::registry::ElementRegistry;

/// Create a content registry configured for Dutch law XML.
///
/// This registry includes handlers for all known element types
/// in Dutch legal documents.
#[must_use]
pub fn create_content_registry() -> ElementRegistry {
    let mut registry = ElementRegistry::new();

    // Inline handlers
    registry.register("nadruk", NadrukHandler);
    registry.register("extref", ExtrefHandler);
    registry.register("intref", IntrefHandler);
    registry.register("al", AlHandler);
    registry.register("redactie", RedactieHandler);

    // Structural handlers
    registry.register("lidnr", LidnrHandler);
    registry.register("li.nr", LiNrHandler);
    registry.register("lid", LidHandler);
    registry.register("lijst", LijstHandler);
    registry.register("li", LiHandler);

    // Preamble handlers
    registry.register("wij", WijHandler);
    registry.register("considerans", ConsideransHandler);
    registry.register("considerans.al", ConsideransAlHandler);
    registry.register("afkondiging", AfkondigingHandler);
    registry.register("aanhef", AanhefHandler);

    // Passthrough handlers (extract text but no special processing)
    registry.register("sup", PassthroughHandler);
    registry.register("sub", PassthroughHandler);
    registry.register("noot", SkipHandler); // Notes are skipped
    registry.register("nootref", PassthroughHandler);

    // Skip tags (metadata, structure, images)
    registry.skip([
        "meta-data",
        "kop",
        "jcis",
        "jci",
        "brondata",
        "plaatje",
        "illustratie",
        "formule",
        "formule-klein",
        "tussenkop",
        "adres",
        "wat",
        "giosduurbwb",
        "informatieproduct",
        "slotondertekening",
        "slotformulering",
    ]);

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_content_registry() {
        let registry = create_content_registry();

        // Check inline handlers
        assert!(registry.has_handler("nadruk"));
        assert!(registry.has_handler("extref"));
        assert!(registry.has_handler("intref"));
        assert!(registry.has_handler("al"));

        // Check structural handlers
        assert!(registry.has_handler("lid"));
        assert!(registry.has_handler("lijst"));
        assert!(registry.has_handler("li"));

        // Check skip tags
        assert!(registry.should_skip("meta-data"));
        assert!(registry.should_skip("kop"));
        assert!(registry.should_skip("plaatje"));
    }
}
