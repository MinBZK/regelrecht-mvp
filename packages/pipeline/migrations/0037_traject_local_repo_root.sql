-- De repository-root van een lokale bron (RFC-018: de notities staan naast
-- de regelgeving, niet erin). Voor de monorepo is dat `corpus` bij
-- `corpus/regulation/nl`. NULL: de map is zijn eigen root, zoals tot nu toe.
ALTER TABLE traject_corpus_sources ADD COLUMN local_repo_root TEXT;
