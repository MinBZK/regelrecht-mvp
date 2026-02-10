Feature: Leerplichtwet 1969
  Als burger
  Wil ik weten of mijn kind leerplichtig of kwalificatieplichtig is
  Zodat ik aan mijn wettelijke verplichtingen kan voldoen

  Background:
    Given the calculation date is "2024-06-01"

  # === Artikel 3: Leerplicht (5-16 jaar / 12 schooljaren) ===

  Scenario: Kind 4 jaar, nog niet leerplichtig
    Given a query with the following data:
      | leeftijd         | 4 |
      | aantal_schooljaren | 0 |
    When the law output is_leerplichtig is requested for leerplichtwet_1969
    Then the result "is_leerplichtig" is "false"

  Scenario: Kind 5 jaar, leerplichtig
    Given a query with the following data:
      | leeftijd         | 5 |
      | aantal_schooljaren | 0 |
    When the law output is_leerplichtig is requested for leerplichtwet_1969
    Then the result "is_leerplichtig" is "true"

  Scenario: Kind 10 jaar, 5 schooljaren
    Given a query with the following data:
      | leeftijd         | 10 |
      | aantal_schooljaren | 5  |
    When the law output is_leerplichtig is requested for leerplichtwet_1969
    Then the result "is_leerplichtig" is "true"

  Scenario: Kind 15 jaar, 11 schooljaren, nog leerplichtig
    Given a query with the following data:
      | leeftijd         | 15 |
      | aantal_schooljaren | 11 |
    When the law output is_leerplichtig is requested for leerplichtwet_1969
    Then the result "is_leerplichtig" is "true"

  Scenario: Kind 15 jaar, 12 schooljaren, leerplicht afgelopen
    Given a query with the following data:
      | leeftijd         | 15 |
      | aantal_schooljaren | 12 |
    When the law output is_leerplichtig is requested for leerplichtwet_1969
    Then the result "is_leerplichtig" is "false"

  Scenario: Jongere 16 jaar, 10 schooljaren, leerplicht afgelopen
    Given a query with the following data:
      | leeftijd         | 16 |
      | aantal_schooljaren | 10 |
    When the law output is_leerplichtig is requested for leerplichtwet_1969
    Then the result "is_leerplichtig" is "false"

  Scenario: Jongere 16 jaar, 12 schooljaren
    Given a query with the following data:
      | leeftijd         | 16 |
      | aantal_schooljaren | 12 |
    When the law output is_leerplichtig is requested for leerplichtwet_1969
    Then the result "is_leerplichtig" is "false"

  # === Artikel 2: Eigen schoolplicht (vanaf 12 jaar) ===

  Scenario: Kind 11 jaar, geen eigen schoolplicht
    Given a query with the following data:
      | leeftijd | 11 |
    When the law output heeft_eigen_schoolplicht is requested for leerplichtwet_1969
    Then the result "heeft_eigen_schoolplicht" is "false"

  Scenario: Kind 12 jaar, eigen schoolplicht
    Given a query with the following data:
      | leeftijd | 12 |
    When the law output heeft_eigen_schoolplicht is requested for leerplichtwet_1969
    Then the result "heeft_eigen_schoolplicht" is "true"

  Scenario: Jongere 15 jaar, eigen schoolplicht
    Given a query with the following data:
      | leeftijd | 15 |
    When the law output heeft_eigen_schoolplicht is requested for leerplichtwet_1969
    Then the result "heeft_eigen_schoolplicht" is "true"

  # === Artikel 4b: Kwalificatieplicht (16-18, geen startkwalificatie) ===

  Scenario: 14 jaar, 9 schooljaren, nog leerplichtig, niet kwalificatieplichtig
    Given a query with the following data:
      | leeftijd              | 14    |
      | aantal_schooljaren    | 9     |
      | heeft_startkwalificatie | false |
    When the law output is_kwalificatieplichtig is requested for leerplichtwet_1969
    Then the result "is_kwalificatieplichtig" is "false"

  Scenario: 16 jaar, 10 schooljaren, geen startkwalificatie, kwalificatieplichtig
    Given a query with the following data:
      | leeftijd              | 16    |
      | aantal_schooljaren    | 10    |
      | heeft_startkwalificatie | false |
    When the law output is_kwalificatieplichtig is requested for leerplichtwet_1969
    Then the result "is_kwalificatieplichtig" is "true"

  Scenario: 17 jaar, 11 schooljaren, geen startkwalificatie, kwalificatieplichtig
    Given a query with the following data:
      | leeftijd              | 17    |
      | aantal_schooljaren    | 11    |
      | heeft_startkwalificatie | false |
    When the law output is_kwalificatieplichtig is requested for leerplichtwet_1969
    Then the result "is_kwalificatieplichtig" is "true"

  Scenario: 17 jaar, 11 schooljaren, wel startkwalificatie, niet kwalificatieplichtig
    Given a query with the following data:
      | leeftijd              | 17   |
      | aantal_schooljaren    | 11   |
      | heeft_startkwalificatie | true |
    When the law output is_kwalificatieplichtig is requested for leerplichtwet_1969
    Then the result "is_kwalificatieplichtig" is "false"

  Scenario: 18 jaar, 12 schooljaren, geen startkwalificatie, te oud
    Given a query with the following data:
      | leeftijd              | 18    |
      | aantal_schooljaren    | 12    |
      | heeft_startkwalificatie | false |
    When the law output is_kwalificatieplichtig is requested for leerplichtwet_1969
    Then the result "is_kwalificatieplichtig" is "false"

  Scenario: 15 jaar, 12 schooljaren, leerplicht afgelopen door schooljaren, kwalificatieplichtig
    Given a query with the following data:
      | leeftijd              | 15    |
      | aantal_schooljaren    | 12    |
      | heeft_startkwalificatie | false |
    When the law output is_kwalificatieplichtig is requested for leerplichtwet_1969
    Then the result "is_kwalificatieplichtig" is "true"

  # === Artikel 11a: Vrijstelling jonge kinderen (< 6 jaar) ===

  Scenario: 5 jaar, standaard vrijstelling
    Given a query with the following data:
      | leeftijd                     | 5     |
      | heeft_uitbreiding_vrijstelling | false |
    When the law output vrijstelling_uren_per_week is requested for leerplichtwet_1969
    Then the result "vrijstelling_uren_per_week" is "5"

  Scenario: 5 jaar, met uitbreiding vrijstelling
    Given a query with the following data:
      | leeftijd                     | 5    |
      | heeft_uitbreiding_vrijstelling | true |
    When the law output vrijstelling_uren_per_week is requested for leerplichtwet_1969
    Then the result "vrijstelling_uren_per_week" is "10"

  Scenario: 6 jaar, geen vrijstelling
    Given a query with the following data:
      | leeftijd                     | 6     |
      | heeft_uitbreiding_vrijstelling | false |
    When the law output vrijstelling_uren_per_week is requested for leerplichtwet_1969
    Then the result "vrijstelling_uren_per_week" is "0"

  Scenario: 10 jaar, geen vrijstelling
    Given a query with the following data:
      | leeftijd                     | 10    |
      | heeft_uitbreiding_vrijstelling | false |
    When the law output vrijstelling_uren_per_week is requested for leerplichtwet_1969
    Then the result "vrijstelling_uren_per_week" is "0"
