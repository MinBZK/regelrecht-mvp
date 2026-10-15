Feature: Het bezwaar en de beslissing erop
  Wie bezwaar maakt, dient een bezwaarschrift in bij het bestuursorgaan dat
  het besluit nam (artikel 6:4). Het bezwaarschrift bevat ten minste wat
  artikel 6:5 noemt en is tijdig als het voor het einde van de termijn is
  ontvangen (artikel 6:9). Op een ontvankelijk bezwaar volgt een
  heroverweging; geeft die aanleiding, dan herroept het bestuursorgaan het
  bestreden besluit (artikel 7:11). Het beslist binnen zes weken na het
  einde van de bezwaartermijn (artikel 7:10).

  Buiten een procedure geeft de uitvoering artikel 7:11 de tijdigheid als
  parameter mee; dat artikel 6:9 die uit de data afleidt, toont de laatste
  scenario. De keten van bezwaarschrift naar beslissing staat in de tests
  van de cel (packages/cel).

  Background:
    Given the calculation date is "2026-11-15"

  Scenario: Een tijdig bezwaar waarop de heroverweging aanleiding geeft, herroept het besluit
    Given the following parameters:
      | bezwaarschrift_ondertekend     | true                     |
      | naam_indiener                  | J. Jansen                |
      | adres_indiener                 | Voorbeeldstraat 1        |
      | dagtekening_bezwaarschrift     | 2026-11-01               |
      | omschrijving_bestreden_besluit | weigering vakantieverlof |
      | gronden_bezwaar                | verklaring werkgever     |
      | datum_ontvangst_bezwaarschrift | 2026-11-01               |
      | bezwaartermijn_einddatum       | 2026-12-01               |
      | bezwaarschrift_tijdig_ingediend | true                    |
      | heroverweging_geeft_aanleiding_tot_herroeping | true      |
    When I evaluate "bestreden_besluit_herroepen" of "algemene_wet_bestuursrecht"
    Then the execution succeeds
    Then output "bezwaar_ontvankelijk" is true
    Then output "bestreden_besluit_herroepen" is true
    Then output "bezwaarschrift_bevat_wat_artikel_6_5_eist" is true
    Then output "beslistermijn_bezwaar_einddatum" equals "2027-01-12"

  Scenario: Geeft de heroverweging geen aanleiding, dan blijft het besluit in stand
    Given the following parameters:
      | datum_ontvangst_bezwaarschrift | 2026-11-01 |
      | bezwaartermijn_einddatum       | 2026-12-01 |
      | bezwaarschrift_tijdig_ingediend | true      |
      | heroverweging_geeft_aanleiding_tot_herroeping | false |
    When I evaluate "bestreden_besluit_herroepen" of "algemene_wet_bestuursrecht"
    Then the execution succeeds
    Then output "bezwaar_ontvankelijk" is true
    Then output "bestreden_besluit_herroepen" is false

  Scenario: Een bezwaarschrift na het einde van de termijn is niet ontvankelijk
    # Ontvangen op de dag na de laatste dag van de termijn. Artikel 6:11
    # (verschoonbare termijnoverschrijding) is niet gemodelleerd. Er vindt
    # geen heroverweging plaats (lid 1), dus er is geen oordeel over.
    Given the following parameters:
      | datum_ontvangst_bezwaarschrift | 2026-12-02 |
      | bezwaartermijn_einddatum       | 2026-12-01 |
      | bezwaarschrift_tijdig_ingediend | false     |
    When I evaluate "bestreden_besluit_herroepen" of "algemene_wet_bestuursrecht"
    Then the execution succeeds
    Then output "bezwaarschrift_tijdig_ingediend" is false
    Then output "bezwaar_ontvankelijk" is false
    Then output "bestreden_besluit_herroepen" is false
