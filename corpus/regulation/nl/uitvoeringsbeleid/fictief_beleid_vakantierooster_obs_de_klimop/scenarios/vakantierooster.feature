Feature: Het vakantierooster van OBS De Klimop
  Het bevoegd gezag stelt voor elk schooljaar een herfst- en een
  voorjaarsvakantie vast, elk een aaneengesloten periode binnen dat
  schooljaar (fictief beleid). De cel legt die vaststelling ambtshalve vast
  in de kroniek van de school (RFC-051).

  Background:
    Given the calculation date is "2026-08-01"

  Scenario: Een rooster binnen het schooljaar volgt het beleid
    Given the following parameters:
      | eerste_jaar_schooljaar        | 2026       |
      | herfstvakantie_eerste_dag     | 2026-10-17 |
      | herfstvakantie_laatste_dag    | 2026-10-25 |
      | voorjaarsvakantie_eerste_dag  | 2027-02-20 |
      | voorjaarsvakantie_laatste_dag | 2027-02-28 |
    When I evaluate "rooster_volgens_beleid" of "fictief_beleid_vakantierooster_obs_de_klimop"
    Then the execution succeeds
    Then output "rooster_volgens_beleid" is true

  Scenario: Een voorjaarsvakantie in het volgende schooljaar volgt het beleid niet
    Given the following parameters:
      | eerste_jaar_schooljaar        | 2026       |
      | herfstvakantie_eerste_dag     | 2026-10-17 |
      | herfstvakantie_laatste_dag    | 2026-10-25 |
      | voorjaarsvakantie_eerste_dag  | 2027-07-31 |
      | voorjaarsvakantie_laatste_dag | 2027-08-08 |
    When I evaluate "rooster_volgens_beleid" of "fictief_beleid_vakantierooster_obs_de_klimop"
    Then output "rooster_volgens_beleid" is false

  Scenario: Een herfstvakantie die eindigt voor zij begint volgt het beleid niet
    Given the following parameters:
      | eerste_jaar_schooljaar        | 2026       |
      | herfstvakantie_eerste_dag     | 2026-10-25 |
      | herfstvakantie_laatste_dag    | 2026-10-17 |
      | voorjaarsvakantie_eerste_dag  | 2027-02-20 |
      | voorjaarsvakantie_laatste_dag | 2027-02-28 |
    When I evaluate "rooster_volgens_beleid" of "fictief_beleid_vakantierooster_obs_de_klimop"
    Then output "rooster_volgens_beleid" is false
