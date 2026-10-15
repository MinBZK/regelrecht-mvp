Feature: Het bezwaar en de beslissing erop
  Wie bezwaar maakt, dient een bezwaarschrift in bij het bestuursorgaan dat
  het besluit nam (artikel 6:4). Het bezwaarschrift bevat ten minste wat
  artikel 6:5 noemt en is tijdig als het voor het einde van de termijn is
  ontvangen (artikel 6:9). Op een ontvankelijk bezwaar volgt een
  heroverweging: het artikel van het bestreden besluit wordt opnieuw
  toegepast, met de gegevens van dat besluit en wat het bestuursorgaan
  opnieuw oordeelt. Geeft dat een andere uitkomst, dan herroept het
  bestuursorgaan het bestreden besluit en neemt het in zijn plaats het besluit
  dat de heroverweging gaf (artikel 7:11). Het beslist binnen zes weken na het
  einde van de bezwaartermijn (artikel 7:10).

  Het bestreden besluit is hier een weigering van vakantieverlof (artikel 13a
  van de Leerplichtwet 1969), als besluitwaarde: het artikel waarop het
  berust, de gegevens waarop het is genomen en zijn uitkomst. Het hoofd
  oordeelde dat het beroep van de ouder niet van de aard is die artikel 11,
  onder f, vraagt. Buiten een procedure geeft de uitvoering artikel 7:11 de
  tijdigheid als parameter mee. De keten van bezwaarschrift naar beslissing,
  met het nieuwe besluit in de kroniek, staat in de tests van de cel
  (packages/cel).

  Background:
    Given the calculation date is "2026-11-15"
    Given law "leerplichtwet_1969" is loaded

  Scenario: Een tijdig bezwaar waarop de heroverweging aanleiding geeft, herroept het besluit
    # Bij de heroverweging oordeelt het hoofd opnieuw over het beroep van de
    # ouder; artikel 13a verleent dan het verlof.
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
      | bestreden_besluit | {"grondslag": "leerplichtwet_1969#13a", "fields": {"bsn_jongere": "999990019", "eerste_verlofdag": "2027-02-01", "laatste_verlofdag": "2027-02-05", "specifieke_aard_beroep_ouder": false, "verlof_valt_buiten_schoolvakanties": true, "verlof_al_verleend_in_schooljaar": false}, "outputs": {"verlofdagen": 5, "verlof_raakt_eerste_twee_lesweken": false, "verlof_verleend": false}} |
      | heroverweging_oordelen | {"specifieke_aard_beroep_ouder": true} |
    When I evaluate outputs "bestreden_besluit_herroepen, nieuw_besluit" of "algemene_wet_bestuursrecht"
    Then the execution succeeds
    Then output "bezwaar_ontvankelijk" is true
    Then output "heroverweging_geeft_aanleiding" is true
    Then output "bestreden_besluit_herroepen" is true
    Then output "bezwaarschrift_bevat_wat_artikel_6_5_eist" is true
    Then output "beslistermijn_bezwaar_einddatum" equals "2027-01-12"

  Scenario: Geeft de heroverweging geen aanleiding, dan blijft het besluit in stand
    # Het hoofd oordeelt niets opnieuw: artikel 13a weigert opnieuw.
    Given the following parameters:
      | datum_ontvangst_bezwaarschrift | 2026-11-01 |
      | bezwaartermijn_einddatum       | 2026-12-01 |
      | bezwaarschrift_tijdig_ingediend | true      |
      | bestreden_besluit | {"grondslag": "leerplichtwet_1969#13a", "fields": {"bsn_jongere": "999990019", "eerste_verlofdag": "2027-02-01", "laatste_verlofdag": "2027-02-05", "specifieke_aard_beroep_ouder": false, "verlof_valt_buiten_schoolvakanties": true, "verlof_al_verleend_in_schooljaar": false}, "outputs": {"verlofdagen": 5, "verlof_raakt_eerste_twee_lesweken": false, "verlof_verleend": false}} |
      | heroverweging_oordelen | null |
    When I evaluate outputs "bestreden_besluit_herroepen, nieuw_besluit" of "algemene_wet_bestuursrecht"
    Then the execution succeeds
    Then output "bezwaar_ontvankelijk" is true
    Then output "heroverweging_geeft_aanleiding" is false
    Then output "bestreden_besluit_herroepen" is false
    Then output "nieuw_besluit" is absent

  Scenario: Een bezwaarschrift na het einde van de termijn is niet ontvankelijk
    # Ontvangen op de dag na de laatste dag van de termijn. Artikel 6:11
    # (verschoonbare termijnoverschrijding) is niet gemodelleerd. Er vindt
    # geen heroverweging plaats (lid 1), dus er is geen oordeel over.
    Given the following parameters:
      | datum_ontvangst_bezwaarschrift | 2026-12-02 |
      | bezwaartermijn_einddatum       | 2026-12-01 |
      | bezwaarschrift_tijdig_ingediend | false     |
      | bestreden_besluit | {"grondslag": "leerplichtwet_1969#13a", "fields": {"bsn_jongere": "999990019", "eerste_verlofdag": "2027-02-01", "laatste_verlofdag": "2027-02-05", "specifieke_aard_beroep_ouder": false, "verlof_valt_buiten_schoolvakanties": true, "verlof_al_verleend_in_schooljaar": false}, "outputs": {"verlofdagen": 5, "verlof_raakt_eerste_twee_lesweken": false, "verlof_verleend": false}} |
      | heroverweging_oordelen | {"specifieke_aard_beroep_ouder": true} |
    When I evaluate outputs "bestreden_besluit_herroepen, nieuw_besluit" of "algemene_wet_bestuursrecht"
    Then the execution succeeds
    Then output "bezwaarschrift_tijdig_ingediend" is false
    Then output "bezwaar_ontvankelijk" is false
    Then output "heroverweging_geeft_aanleiding" is absent
    Then output "bestreden_besluit_herroepen" is false
    Then output "nieuw_besluit" is absent

  Scenario: Tegen een beslissing op bezwaar staat geen bezwaar open
    # Daartegen staat beroep open (artikel 7:1, eerste lid, onder a).
    Given the following parameters:
      | bezwaarschrift_tijdig_ingediend | true |
      | bestreden_besluit | {"grondslag": "algemene_wet_bestuursrecht#7:11", "fields": {"bezwaarschrift_tijdig_ingediend": true}, "outputs": {"bezwaar_ontvankelijk": true, "heroverweging_geeft_aanleiding": false, "bestreden_besluit_herroepen": false, "nieuw_besluit": null}} |
      | heroverweging_oordelen | null |
    When I evaluate outputs "bestreden_besluit_herroepen, nieuw_besluit" of "algemene_wet_bestuursrecht"
    Then the execution succeeds
    Then output "bezwaar_ontvankelijk" is false
    Then output "heroverweging_geeft_aanleiding" is absent
    Then output "bestreden_besluit_herroepen" is false
