Feature: Eerder verleend vakantieverlof bij OBS De Klimop
  Het fictieve beleid van de school leest uit de verloven die het hoofd
  verleende of het de jongere in het schooljaar van het gevraagde verlof al
  verlof verleende (artikel 13a, tweede lid, van de Leerplichtwet 1969:
  slechts eenmaal per schooljaar). Een schooljaar loopt van 1 augustus tot
  en met 31 juli. Elk verleend verlof is een rij met de jongere en de eerste
  verlofdag. Het beleid zegt niet wie die lijst aanlevert; hier doet het
  scenario dat.

  Background:
    Given the calculation date is "2027-03-01"
    Given the following parameters:
      | bsn_jongere      | 999990019  |
      | eerste_verlofdag | 2027-05-10 |

  Scenario: Zonder verleende verloven is er nog geen verlof verleend
    Given parameter "verleende_verloven" is the collection:
      | bsn_jongere | eerste_verlofdag |
    When I evaluate "verlof_al_verleend_in_schooljaar" of "fictief_beleid_verleend_verlof_obs_de_klimop"
    Then output "verlof_al_verleend_in_schooljaar" is false
    Then output "begin_schooljaar" equals "2026-08-01"
    Then output "einde_schooljaar" equals "2027-07-31"

  Scenario: Verlof verleend in hetzelfde schooljaar telt
    Given parameter "verleende_verloven" is the collection:
      | bsn_jongere | eerste_verlofdag |
      | 999990019   | 2026-10-05       |
    When I evaluate "verlof_al_verleend_in_schooljaar" of "fictief_beleid_verleend_verlof_obs_de_klimop"
    Then output "verlof_al_verleend_in_schooljaar" is true

  Scenario: Verlof voor een andere jongere telt niet
    Given parameter "verleende_verloven" is the collection:
      | bsn_jongere | eerste_verlofdag |
      | 999990020   | 2026-10-05       |
    When I evaluate "verlof_al_verleend_in_schooljaar" of "fictief_beleid_verleend_verlof_obs_de_klimop"
    Then output "verlof_al_verleend_in_schooljaar" is false

  Scenario: Verlof in juli hoort bij het schooljaar dat in augustus ervoor begon
    Given parameter "verleende_verloven" is the collection:
      | bsn_jongere | eerste_verlofdag |
      | 999990019   | 2026-07-27       |
      | 999990019   | 2027-08-02       |
    When I evaluate "verlof_al_verleend_in_schooljaar" of "fictief_beleid_verleend_verlof_obs_de_klimop"
    Then output "verlof_al_verleend_in_schooljaar" is false

  Scenario: Verlof op 1 augustus hoort bij het nieuwe schooljaar
    Given parameter "verleende_verloven" is the collection:
      | bsn_jongere | eerste_verlofdag |
      | 999990019   | 2026-08-01       |
    When I evaluate "verlof_al_verleend_in_schooljaar" of "fictief_beleid_verleend_verlof_obs_de_klimop"
    Then output "verlof_al_verleend_in_schooljaar" is true

  Scenario: Verlof op 31 juli hoort nog bij het schooljaar dat eindigt
    Given parameter "verleende_verloven" is the collection:
      | bsn_jongere | eerste_verlofdag |
      | 999990019   | 2027-07-31       |
    When I evaluate "verlof_al_verleend_in_schooljaar" of "fictief_beleid_verleend_verlof_obs_de_klimop"
    Then output "verlof_al_verleend_in_schooljaar" is true
