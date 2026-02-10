Feature: Rijbewijscategorieën minimumleeftijd (Reglement rijbewijzen Art. 5)
  Als rijbewijsaanvrager
  Wil ik weten of ik aan de minimumleeftijd voldoe
  Zodat ik weet of ik een rijbewijs kan aanvragen

  Background:
    Given the calculation date is "2025-01-01"

  # === AM categorie (sub a): 16 jaar ===

  Scenario: 16-jarige voldoet aan minimumleeftijd voor AM
    Given a query with the following data:
      | leeftijd           | 16    |
      | categorie          | AM    |
      | heeft_rijbewijs_A2 | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "16"
    And the persoon voldoet aan de leeftijdseis

  Scenario: 15-jarige voldoet niet aan minimumleeftijd voor AM
    Given a query with the following data:
      | leeftijd           | 15    |
      | categorie          | AM    |
      | heeft_rijbewijs_A2 | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "16"
    And the persoon voldoet niet aan de leeftijdseis

  # === A1 categorie (sub b): 18 jaar ===

  Scenario: 18-jarige voldoet aan minimumleeftijd voor A1
    Given a query with the following data:
      | leeftijd           | 18    |
      | categorie          | A1    |
      | heeft_rijbewijs_A2 | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "18"
    And the persoon voldoet aan de leeftijdseis

  Scenario: 17-jarige voldoet niet aan minimumleeftijd voor A1
    Given a query with the following data:
      | leeftijd           | 17    |
      | categorie          | A1    |
      | heeft_rijbewijs_A2 | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "18"
    And the persoon voldoet niet aan de leeftijdseis

  # === A2 categorie (sub c): 20 jaar ===

  Scenario: 20-jarige voldoet aan minimumleeftijd voor A2
    Given a query with the following data:
      | leeftijd           | 20    |
      | categorie          | A2    |
      | heeft_rijbewijs_A2 | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "20"
    And the persoon voldoet aan de leeftijdseis

  Scenario: 19-jarige voldoet niet aan minimumleeftijd voor A2
    Given a query with the following data:
      | leeftijd           | 19    |
      | categorie          | A2    |
      | heeft_rijbewijs_A2 | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "20"
    And the persoon voldoet niet aan de leeftijdseis

  # === A categorie met A2-rijbewijs (sub d): 22 jaar ===

  Scenario: 22-jarige met A2-rijbewijs voldoet aan minimumleeftijd voor A
    Given a query with the following data:
      | leeftijd           | 22    |
      | categorie          | A     |
      | heeft_rijbewijs_A2 | true  |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "22"
    And the persoon voldoet aan de leeftijdseis

  Scenario: 21-jarige met A2-rijbewijs voldoet niet aan minimumleeftijd voor A
    Given a query with the following data:
      | leeftijd           | 21    |
      | categorie          | A     |
      | heeft_rijbewijs_A2 | true  |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "22"
    And the persoon voldoet niet aan de leeftijdseis

  # === A categorie zonder A2-rijbewijs (sub e): 24 jaar ===

  Scenario: 24-jarige zonder A2-rijbewijs voldoet aan minimumleeftijd voor A
    Given a query with the following data:
      | leeftijd           | 24    |
      | categorie          | A     |
      | heeft_rijbewijs_A2 | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "24"
    And the persoon voldoet aan de leeftijdseis

  Scenario: 23-jarige zonder A2-rijbewijs voldoet niet aan minimumleeftijd voor A
    Given a query with the following data:
      | leeftijd           | 23    |
      | categorie          | A     |
      | heeft_rijbewijs_A2 | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "24"
    And the persoon voldoet niet aan de leeftijdseis

  # === A categorie driewielig (sub e uitzondering): 21 jaar ===

  Scenario: 21-jarige voor driewielig A voldoet aan minimumleeftijd
    Given a query with the following data:
      | leeftijd           | 21    |
      | categorie          | A     |
      | heeft_rijbewijs_A2 | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | true  |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "21"
    And the persoon voldoet aan de leeftijdseis

  Scenario: 20-jarige voor driewielig A voldoet niet aan minimumleeftijd
    Given a query with the following data:
      | leeftijd           | 20    |
      | categorie          | A     |
      | heeft_rijbewijs_A2 | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | true  |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "21"
    And the persoon voldoet niet aan de leeftijdseis

  # === B categorie (sub f): 18 jaar ===

  Scenario: 18-jarige voldoet aan minimumleeftijd voor B
    Given a query with the following data:
      | leeftijd           | 18    |
      | categorie          | B     |
      | heeft_rijbewijs_A2 | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "18"
    And the persoon voldoet aan de leeftijdseis

  Scenario: 17-jarige voldoet niet aan minimumleeftijd voor B
    Given a query with the following data:
      | leeftijd           | 17    |
      | categorie          | B     |
      | heeft_rijbewijs_A2 | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "18"
    And the persoon voldoet niet aan de leeftijdseis

  # === C1 categorie (sub g): 18 jaar ===

  Scenario: 18-jarige voldoet aan minimumleeftijd voor C1
    Given a query with the following data:
      | leeftijd           | 18    |
      | categorie          | C1    |
      | heeft_rijbewijs_A2 | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig      | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "18"
    And the persoon voldoet aan de leeftijdseis

  # === C categorie zonder vakbekwaamheid (sub h): 21 jaar ===

  Scenario: 21-jarige zonder vakbekwaamheid voldoet aan minimumleeftijd voor C
    Given a query with the following data:
      | leeftijd             | 21    |
      | categorie            | C     |
      | heeft_rijbewijs_A2   | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig        | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "21"
    And the persoon voldoet aan de leeftijdseis

  Scenario: 20-jarige zonder vakbekwaamheid voldoet niet aan minimumleeftijd voor C
    Given a query with the following data:
      | leeftijd             | 20    |
      | categorie            | C     |
      | heeft_rijbewijs_A2   | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig        | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "21"
    And the persoon voldoet niet aan de leeftijdseis

  # === C categorie met vakbekwaamheid (sub i): 18 jaar ===

  Scenario: 18-jarige met vakbekwaamheid voldoet aan minimumleeftijd voor C
    Given a query with the following data:
      | leeftijd             | 18    |
      | categorie            | C     |
      | heeft_rijbewijs_A2   | false |
      | heeft_vakbekwaamheid | true  |
      | is_driewielig        | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "18"
    And the persoon voldoet aan de leeftijdseis

  # === D1 categorie zonder vakbekwaamheid (sub j): 21 jaar ===

  Scenario: 21-jarige zonder vakbekwaamheid voldoet aan minimumleeftijd voor D1
    Given a query with the following data:
      | leeftijd             | 21    |
      | categorie            | D1    |
      | heeft_rijbewijs_A2   | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig        | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "21"
    And the persoon voldoet aan de leeftijdseis

  Scenario: 20-jarige zonder vakbekwaamheid voldoet niet aan minimumleeftijd voor D1
    Given a query with the following data:
      | leeftijd             | 20    |
      | categorie            | D1    |
      | heeft_rijbewijs_A2   | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig        | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "21"
    And the persoon voldoet niet aan de leeftijdseis

  # === D1 categorie met vakbekwaamheid (sub k): 18 jaar ===

  Scenario: 18-jarige met vakbekwaamheid voldoet aan minimumleeftijd voor D1
    Given a query with the following data:
      | leeftijd             | 18    |
      | categorie            | D1    |
      | heeft_rijbewijs_A2   | false |
      | heeft_vakbekwaamheid | true  |
      | is_driewielig        | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "18"
    And the persoon voldoet aan de leeftijdseis

  # === D categorie zonder vakbekwaamheid (sub l): 24 jaar ===

  Scenario: 24-jarige zonder vakbekwaamheid voldoet aan minimumleeftijd voor D
    Given a query with the following data:
      | leeftijd             | 24    |
      | categorie            | D     |
      | heeft_rijbewijs_A2   | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig        | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "24"
    And the persoon voldoet aan de leeftijdseis

  Scenario: 23-jarige zonder vakbekwaamheid voldoet niet aan minimumleeftijd voor D
    Given a query with the following data:
      | leeftijd             | 23    |
      | categorie            | D     |
      | heeft_rijbewijs_A2   | false |
      | heeft_vakbekwaamheid | false |
      | is_driewielig        | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "24"
    And the persoon voldoet niet aan de leeftijdseis

  # === D categorie met vakbekwaamheid (sub m): 21 jaar ===

  Scenario: 21-jarige met vakbekwaamheid voldoet aan minimumleeftijd voor D
    Given a query with the following data:
      | leeftijd             | 21    |
      | categorie            | D     |
      | heeft_rijbewijs_A2   | false |
      | heeft_vakbekwaamheid | true  |
      | is_driewielig        | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "21"
    And the persoon voldoet aan de leeftijdseis

  Scenario: 20-jarige met vakbekwaamheid voldoet niet aan minimumleeftijd voor D
    Given a query with the following data:
      | leeftijd             | 20    |
      | categorie            | D     |
      | heeft_rijbewijs_A2   | false |
      | heeft_vakbekwaamheid | true  |
      | is_driewielig        | false |
    When the rijbewijs minimumleeftijd is requested for reglement_rijbewijzen article 5
    Then the minimum_leeftijd is "21"
    And the persoon voldoet niet aan de leeftijdseis
