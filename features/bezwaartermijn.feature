Feature: Bezwaartermijn chain
  Als burger die een beschikking ontvangt
  Wil ik weten wanneer de bezwaartermijn afloopt
  Zodat ik tijdig bezwaar kan maken

  # This feature tests four RFCs working together:
  # - RFC-007 (IoC): KB gelijkgestelde dagen implements Termijnenwet art 3
  # - RFC-008 (Hooks): AWB articles fire on BESCHIKKING
  # - RFC-009 (Overrides): Vreemdelingenwet overrides AWB 6:7
  # - RFC-011 (Temporal): Date arithmetic, feestdagen calendar

  Background:
    Given the calculation date is "2026-01-01"

  # === Hooks: AWB fires on any BESCHIKKING ===

  Scenario: Vreemdelingenwet beschikking includes bezwaartermijn via hook
    Given a bezwaartermijn query with the following data:
      | heeft_geldige_mvv     | true |
      | heeft_geldig_document | true |
    When the vreemdelingenwet beschikking is executed
    Then the execution succeeds
    And the output "verblijfsvergunning_verleend" is "true"
    And the output "motivering_vereist" is "true"
    And the output "bezwaartermijn_weken" is "4"

  # === Override: Vreemdelingenwet overrides bezwaartermijn ===

  Scenario: Vreemdelingenwet override changes bezwaartermijn from 6 to 4 weeks
    Given a bezwaartermijn query with the following data:
      | heeft_geldige_mvv     | true |
      | heeft_geldig_document | true |
    When the vreemdelingenwet beschikking is executed
    Then the execution succeeds
    And the output "bezwaartermijn_weken" is "4"

  # === Date arithmetic: full chain with bekendmaking_datum ===

  Scenario: Full bezwaartermijn date chain for vreemdelingenwet
    Given a bezwaartermijn query with the following data:
      | heeft_geldige_mvv     | true       |
      | heeft_geldig_document | true       |
      | bekendmaking_datum    | 2026-03-12 |
      | jaar                  | 2026       |
      | pasen_datum           | 2026-04-05 |
    When the vreemdelingenwet beschikking is executed
    Then the execution succeeds
    And the output "verblijfsvergunning_verleend" is "true"
    And the output "bezwaartermijn_weken" is "4"
    And the output "bezwaartermijn_startdatum" is "2026-03-13"
    And the output "bezwaartermijn_einddatum" is "2026-04-09"

  # === Feestdagen calendar: Termijnenwet art 3 with IoC ===

  Scenario: Feestdagen list includes fixed, Easter-dependent, and KB dates
    Given a bezwaartermijn query with the following data:
      | jaar        | 2026       |
      | pasen_datum | 2026-04-05 |
    When the feestdagen calendar is requested
    Then the execution succeeds
    And the output "feestdagen" contains "2026-01-01"
    And the output "feestdagen" contains "2026-01-02"
    And the output "feestdagen" contains "2026-04-03"
    And the output "feestdagen" contains "2026-04-06"
    And the output "feestdagen" contains "2026-04-27"
    And the output "feestdagen" contains "2026-05-05"
    And the output "feestdagen" contains "2026-05-14"
    And the output "feestdagen" contains "2026-05-15"
    And the output "feestdagen" contains "2026-05-25"
    And the output "feestdagen" contains "2026-12-25"
    And the output "feestdagen" contains "2026-12-26"

  # === Weekend/feestdag extension ===

  Scenario: Termijn ending on Saturday is extended to Monday
    Given a bezwaartermijn query with the following data:
      | termijn_einddatum | 2026-03-14 |
      | jaar              | 2026       |
      | pasen_datum       | 2026-04-05 |
    When the termijn extension is requested
    Then the execution succeeds
    And the output "verlengde_einddatum" is "2026-03-16"

  Scenario: Termijn ending on feestdag is extended past weekend
    # Goede Vrijdag 3 apr → za 4 + zo 5 (Pasen) + ma 6 (2e Paasdag) → di 7 apr
    Given a bezwaartermijn query with the following data:
      | termijn_einddatum | 2026-04-03 |
      | jaar              | 2026       |
      | pasen_datum       | 2026-04-05 |
    When the termijn extension is requested
    Then the execution succeeds
    And the output "verlengde_einddatum" is "2026-04-07"

  Scenario: Nieuwjaar + KB brugdag chain (4 days)
    # do 1 jan (Nieuwjaar) + vr 2 jan (KB brugdag) + za 3 + zo 4 → ma 5 jan
    Given a bezwaartermijn query with the following data:
      | termijn_einddatum | 2026-01-01 |
      | jaar              | 2026       |
      | pasen_datum       | 2026-04-05 |
    When the termijn extension is requested
    Then the execution succeeds
    And the output "verlengde_einddatum" is "2026-01-05"

  Scenario: Hemelvaart + KB brugdag chain (4 days)
    # do 14 mei (Hemelvaart) + vr 15 mei (KB brugdag) + za 16 + zo 17 → ma 18 mei
    Given a bezwaartermijn query with the following data:
      | termijn_einddatum | 2026-05-14 |
      | jaar              | 2026       |
      | pasen_datum       | 2026-04-05 |
    When the termijn extension is requested
    Then the execution succeeds
    And the output "verlengde_einddatum" is "2026-05-18"

  Scenario: Kerst chain (3 days)
    # vr 25 dec (1e Kerstdag) + za 26 (2e Kerstdag) + zo 27 → ma 28 dec
    Given a bezwaartermijn query with the following data:
      | termijn_einddatum | 2026-12-25 |
      | jaar              | 2026       |
      | pasen_datum       | 2026-04-05 |
    When the termijn extension is requested
    Then the execution succeeds
    And the output "verlengde_einddatum" is "2026-12-28"

  Scenario: Weekday that is not a feestdag stays unchanged
    # wo 11 mrt — gewone werkdag, geen verlenging
    Given a bezwaartermijn query with the following data:
      | termijn_einddatum | 2026-03-11 |
      | jaar              | 2026       |
      | pasen_datum       | 2026-04-05 |
    When the termijn extension is requested
    Then the execution succeeds
    And the output "verlengde_einddatum" is "2026-03-11"

  # === Edge cases: no-override context ===

  Scenario: AWB 6:7 without contextual law returns 6 weeks (no override)
    # When AWB is executed directly (not via Vreemdelingenwet), no lex specialis
    # override applies — bezwaartermijn stays at the default 6 weeks.
    When the AWB bezwaartermijn is executed directly
    Then the execution succeeds
    And the output "bezwaartermijn_weken" is "6"

  # === Edge cases: hook skip on missing parameter ===

  Scenario: Hook skips gracefully when optional parameter is missing
    # AWB 6:8 needs bekendmaking_datum for date calculation. When not
    # provided, 6:8 should skip gracefully. AWB 3:46 and 6:7 (which
    # don't need that parameter) should still fire.
    Given a bezwaartermijn query with the following data:
      | heeft_geldige_mvv     | true |
      | heeft_geldig_document | true |
    When the vreemdelingenwet beschikking is executed
    Then the execution succeeds
    And the output "motivering_vereist" is "true"
    And the output "bezwaartermijn_weken" is "4"
    And the output "bezwaartermijn_startdatum" is not present
    And the output "bezwaartermijn_einddatum" is not present
