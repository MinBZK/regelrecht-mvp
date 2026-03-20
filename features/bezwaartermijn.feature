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
    Given a bezwaartermijn query with the following data:
      | termijn_einddatum | 2026-04-03 |
      | jaar              | 2026       |
      | pasen_datum       | 2026-04-05 |
    When the termijn extension is requested
    Then the execution succeeds
    And the output "verlengde_einddatum" is "2026-04-07"
