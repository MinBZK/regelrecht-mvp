Feature: Eerder verleend vakantieverlof bij OBS De Klimop
  Het fictieve beleid van de school leest uit de eerdere besluiten van het
  hoofd of het de jongere in het schooljaar van het gevraagde verlof al
  verlof verleende (artikel 13a, tweede lid, van de Leerplichtwet 1969:
  slechts eenmaal per schooljaar). Een schooljaar loopt van 1 augustus tot
  en met 31 juli. Elk besluit is een rij met de jongere, de eerste verlofdag
  en of verlof is verleend.

  Background:
    Given the calculation date is "2027-03-01"
    Given the following parameters:
      | bsn_jongere      | 999990019  |
      | eerste_verlofdag | 2027-05-10 |

  Scenario: Zonder eerdere besluiten is er nog geen verlof verleend
    Given parameter "eerdere_besluiten" is the collection:
      | bsn_jongere | eerste_verlofdag | verlof_verleend |
    When I evaluate "verlof_al_verleend_in_schooljaar" of "fictief_beleid_verleend_verlof_obs_de_klimop"
    Then output "verlof_al_verleend_in_schooljaar" is false
    Then output "schooljaar_van_het_gevraagde_verlof" equals 2026

  Scenario: Verlof verleend in hetzelfde schooljaar telt
    Given parameter "eerdere_besluiten" is the collection:
      | bsn_jongere | eerste_verlofdag | verlof_verleend |
      | 999990019   | 2026-10-05       | true            |
    When I evaluate "verlof_al_verleend_in_schooljaar" of "fictief_beleid_verleend_verlof_obs_de_klimop"
    Then output "verlof_al_verleend_in_schooljaar" is true

  Scenario: Een weigering telt niet
    Given parameter "eerdere_besluiten" is the collection:
      | bsn_jongere | eerste_verlofdag | verlof_verleend |
      | 999990019   | 2026-10-05       | false           |
    When I evaluate "verlof_al_verleend_in_schooljaar" of "fictief_beleid_verleend_verlof_obs_de_klimop"
    Then output "verlof_al_verleend_in_schooljaar" is false

  Scenario: Verlof voor een andere jongere telt niet
    Given parameter "eerdere_besluiten" is the collection:
      | bsn_jongere | eerste_verlofdag | verlof_verleend |
      | 999990020   | 2026-10-05       | true            |
    When I evaluate "verlof_al_verleend_in_schooljaar" of "fictief_beleid_verleend_verlof_obs_de_klimop"
    Then output "verlof_al_verleend_in_schooljaar" is false

  Scenario: Verlof in juli hoort bij het schooljaar dat in augustus ervoor begon
    Given parameter "eerdere_besluiten" is the collection:
      | bsn_jongere | eerste_verlofdag | verlof_verleend |
      | 999990019   | 2026-07-27       | true            |
      | 999990019   | 2027-08-02       | true            |
    When I evaluate "verlof_al_verleend_in_schooljaar" of "fictief_beleid_verleend_verlof_obs_de_klimop"
    Then output "verlof_al_verleend_in_schooljaar" is false

  Scenario: Verlof op 1 augustus hoort bij het nieuwe schooljaar
    Given parameter "eerdere_besluiten" is the collection:
      | bsn_jongere | eerste_verlofdag | verlof_verleend |
      | 999990019   | 2026-08-01       | true            |
    When I evaluate "verlof_al_verleend_in_schooljaar" of "fictief_beleid_verleend_verlof_obs_de_klimop"
    Then output "verlof_al_verleend_in_schooljaar" is true
