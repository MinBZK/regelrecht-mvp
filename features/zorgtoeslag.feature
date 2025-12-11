Feature: Healthcare allowance calculation
  As a citizen with health insurance
  I want to know if I am entitled to healthcare allowance
  So that I can reduce my healthcare costs

  Scenario: Get standard premium from Article 4 for 2025
    When I request the standard premium for year 2025
    Then the standard premium is "211200" eurocent

  Scenario: No regeling found for year 2024
    When I request the standard premium for year 2024
    Then the standard premium calculation should fail with "No matching regeling found"

  Scenario: Person over 18 is entitled to healthcare allowance
    Given the following RVIG "personal_data" data:
      | bsn       | geboortedatum | verblijfsadres | land_verblijf |
      | 999993653 | 2005-01-01    | Amsterdam      | NEDERLAND     |
    And the following RVIG "relationship_data" data:
      | bsn       | partnerschap_type | partner_bsn |
      | 999993653 | GEEN              | null        |
    And the following RVZ "insurance" data:
      | bsn       | polis_status |
      | 999993653 | ACTIEF       |
    And the following BELASTINGDIENST "box1" data:
      | bsn       | loon_uit_dienstbetrekking | uitkeringen_en_pensioenen | winst_uit_onderneming | resultaat_overige_werkzaamheden | eigen_woning |
      | 999993653 | 79547                     | 0                         | 0                     | 0                               | 0            |
    And the following BELASTINGDIENST "box2" data:
      | bsn       | reguliere_voordelen | vervreemdingsvoordelen |
      | 999993653 | 0                   | 0                      |
    And the following BELASTINGDIENST "box3" data:
      | bsn       | spaargeld | beleggingen | onroerend_goed | schulden |
      | 999993653 | 0         | 0           | 0              | 0        |
    When the healthcare allowance law is executed
    Then the allowance amount is "1358.93" euro
