Feature: De aanvraag om vakantieverlof, met de Awb erop
  Artikel 13a vestigt een aanvraag (RFC-045). De Awb haakt op elke aanvraag:
  artikel 4:2 zegt wat de aanvraag ten minste bevat, artikel 4:13 wanneer
  de redelijke beslistermijn in ieder geval is verstreken.

  Background:
    Given the calculation date is "2026-10-13"
    Given law "algemene_wet_bestuursrecht" is loaded

  Scenario: Een volledige aanvraag geeft de velden van 4:2 en de beslistermijn van 4:13
    Given the following parameters:
      | bsn_jongere           | 999990019             |
      | eerste_verlofdag      | 2027-02-01            |
      | laatste_verlofdag     | 2027-02-05            |
      | aanvraag_ondertekend  | true                  |
      | naam_aanvrager        | J. Jansen             |
      | adres_aanvrager       | Voorbeeldstraat 1     |
      | dagtekening           | 2026-10-12            |
      | gevraagde_beschikking | vakantieverlof        |
      | datum_ontvangst       | 2026-10-13            |
    When I evaluate "verlof_aangevraagd" of "leerplichtwet_1969"
    Then the execution succeeds
    Then output "verlof_aangevraagd" is true
    Then output "aanvraag_bevat_wat_artikel_4_2_eist" is true
    Then output "aanvraag_bevat_wat_artikel_4_2_eist" has reactive provenance
    Then output "redelijke_beslistermijn_einddatum" equals "2026-12-08"
    Then output "redelijke_beslistermijn_einddatum" has reactive provenance

  Scenario: Zonder adres bevat de aanvraag niet wat 4:2 eist
    Given the following parameters:
      | bsn_jongere           | 999990019             |
      | eerste_verlofdag      | 2027-02-01            |
      | laatste_verlofdag     | 2027-02-05            |
      | aanvraag_ondertekend  | true                  |
      | naam_aanvrager        | J. Jansen             |
      | adres_aanvrager       | null                  |
      | dagtekening           | 2026-10-12            |
      | gevraagde_beschikking | vakantieverlof        |
      | datum_ontvangst       | 2026-10-13            |
    When I evaluate "verlof_aangevraagd" of "leerplichtwet_1969"
    Then the execution succeeds
    Then output "aanvraag_bevat_wat_artikel_4_2_eist" is false
