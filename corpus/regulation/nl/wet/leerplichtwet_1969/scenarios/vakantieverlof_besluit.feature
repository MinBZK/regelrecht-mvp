Feature: Het besluit op de aanvraag om vakantieverlof
  Het hoofd verleent verlof op aanvraag (artikel 13a, eerste lid), voor de
  vakantie bedoeld in artikel 11, onder f, voor ten hoogste tien dagen en
  niet in de eerste twee lesweken van het schooljaar (tweede lid). De eerste
  lesdag komt uit het (fictieve) beleid van de school, het oordeel over
  artikel 11, onder f, van het hoofd. Het besluit is een beschikking: de Awb
  vraagt een motivering (3:46) en opent een bezwaartermijn van zes weken (6:7).

  Background:
    Given the calculation date is "2026-10-20"
    Given law "algemene_wet_bestuursrecht" is loaded

  Scenario: Vijf schooldagen buiten de eerste twee lesweken worden verleend
    Given the following parameters:
      | bsn_jongere       | 999990019  |
      | eerste_verlofdag  | 2027-02-01 |
      | laatste_verlofdag | 2027-02-05 |
      | alleen_buiten_schoolvakanties_op_vakantie_door_beroep_ouder | true |
    When I evaluate "verlof_verleend" of "leerplichtwet_1969"
    Then the execution succeeds
    Then output "verlof_verleend" is true
    Then output "verlofdagen" equals 5
    Then output "verlof_raakt_eerste_twee_lesweken" is false
    Then output "motivering_vereist" is true
    Then output "motivering_vereist" has reactive provenance
    Then output "bezwaartermijn_weken" equals 6
    Then output "bezwaartermijn_weken" has reactive provenance

  Scenario: Twee weken met het weekend ertussen zijn tien schooldagen
    Given the following parameters:
      | bsn_jongere       | 999990019  |
      | eerste_verlofdag  | 2027-02-01 |
      | laatste_verlofdag | 2027-02-12 |
      | alleen_buiten_schoolvakanties_op_vakantie_door_beroep_ouder | true |
    When I evaluate "verlof_verleend" of "leerplichtwet_1969"
    Then the execution succeeds
    Then output "verlofdagen" equals 10
    Then output "verlof_verleend" is true

  Scenario: Elf schooldagen worden geweigerd
    Given the following parameters:
      | bsn_jongere       | 999990019  |
      | eerste_verlofdag  | 2027-02-01 |
      | laatste_verlofdag | 2027-02-15 |
      | alleen_buiten_schoolvakanties_op_vakantie_door_beroep_ouder | true |
    When I evaluate "verlof_verleend" of "leerplichtwet_1969"
    Then the execution succeeds
    Then output "verlofdagen" equals 11
    Then output "verlof_verleend" is false

  Scenario: Verlof in de tweede lesweek wordt geweigerd
    Given the following parameters:
      | bsn_jongere       | 999990019  |
      | eerste_verlofdag  | 2026-09-10 |
      | laatste_verlofdag | 2026-09-11 |
      | alleen_buiten_schoolvakanties_op_vakantie_door_beroep_ouder | true |
    When I evaluate "verlof_verleend" of "leerplichtwet_1969"
    Then the execution succeeds
    Then output "verlof_raakt_eerste_twee_lesweken" is true
    Then output "verlof_verleend" is false

  Scenario: Zonder de grond van artikel 11, onder f, wordt verlof geweigerd
    Given the following parameters:
      | bsn_jongere       | 999990019  |
      | eerste_verlofdag  | 2027-02-01 |
      | laatste_verlofdag | 2027-02-05 |
      | alleen_buiten_schoolvakanties_op_vakantie_door_beroep_ouder | false |
    When I evaluate "verlof_verleend" of "leerplichtwet_1969"
    Then the execution succeeds
    Then output "verlof_verleend" is false
