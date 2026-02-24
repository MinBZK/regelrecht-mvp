Feature: Bijstandsaanvraag Rotterdam via Participatiewet
  Als burger in Rotterdam zonder voldoende middelen
  Wil ik bijstand kunnen aanvragen bij gemeente Rotterdam
  Zodat ik in mijn levensonderhoud kan voorzien

  # Keten: Participatiewet (Rijkswet) + Verordening maatregelen en handhaving (Rotterdam)
  #
  # Art. 11: Rechthebbenden - Nederlanders zonder middelen
  # Art. 21: Normbedragen 2026 - €1.401,50 (alleenstaand) / €2.002,13 (gehuwd)
  # Art. 8:  Delegatie - gemeente stelt verordening vast
  # Art. 18: Verlaging - bij niet nakomen verplichtingen
  #
  # Verordening maatregelen en handhaving Rotterdam (GM0599):
  #   Categorie 1: 30%  - niet naar vermogen deeltijd, plan van aanpak, tegenprestatie
  #   Categorie 2: 100% - niet naar vermogen algemeen geaccepteerde arbeid verkrijgen
  #
  # Formule: uitkering = normbedrag - (normbedrag × verlaging%)

  Background:
    Given the calculation date is "2026-01-15"

  # === Toekenningsscenario's voor burger uit Rotterdam (GM0599) ===

  Scenario: Alleenstaande Rotterdammer krijgt volledige bijstand
    Given a citizen with the following data:
      | gemeente_code                          | GM0599       |
      | leeftijd                               | 35           |
      | is_nederlander                         | true         |
      | woont_in_nederland                     | true         |
      | is_alleenstaande                       | true         |
      | heeft_kostendelende_medebewoners       | false        |
      | heeft_pensioengerechtigde_leeftijd_bereikt | false    |
      | is_geregistreerd_als_werkzoekende      | true         |
      | heeft_voldoende_middelen               | false        |
      | gedragscategorie                       | 0            |
    When the bijstandsaanvraag is executed for participatiewet article 43
    Then the citizen has the right to bijstand
    And the uitkering_bedrag is "140150" eurocent

  Scenario: Gehuwde Rotterdammers krijgen volledige bijstand
    Given a citizen with the following data:
      | gemeente_code                          | GM0599       |
      | leeftijd                               | 42           |
      | is_nederlander                         | true         |
      | woont_in_nederland                     | true         |
      | is_alleenstaande                       | false        |
      | heeft_kostendelende_medebewoners       | false        |
      | heeft_pensioengerechtigde_leeftijd_bereikt | false    |
      | is_geregistreerd_als_werkzoekende      | true         |
      | heeft_voldoende_middelen               | false        |
      | gedragscategorie                       | 0            |
    When the bijstandsaanvraag is executed for participatiewet article 43
    Then the citizen has the right to bijstand
    And the uitkering_bedrag is "200213" eurocent

  Scenario: Rotterdammer met gedragscategorie 1 krijgt 30% verlaging
    # Categorie 1: niet naar vermogen deeltijd arbeid / plan van aanpak / tegenprestatie
    Given a citizen with the following data:
      | gemeente_code                          | GM0599       |
      | leeftijd                               | 28           |
      | is_nederlander                         | true         |
      | woont_in_nederland                     | true         |
      | is_alleenstaande                       | true         |
      | heeft_kostendelende_medebewoners       | false        |
      | heeft_pensioengerechtigde_leeftijd_bereikt | false    |
      | is_geregistreerd_als_werkzoekende      | true         |
      | heeft_voldoende_middelen               | false        |
      | gedragscategorie                       | 1            |
    When the bijstandsaanvraag is executed for participatiewet article 43
    Then the citizen has the right to bijstand
    And the uitkering_bedrag is "98105" eurocent

  Scenario: Rotterdammer met gedragscategorie 2 krijgt 100% verlaging
    # Categorie 2: niet naar vermogen algemeen geaccepteerde arbeid verkrijgen
    Given a citizen with the following data:
      | gemeente_code                          | GM0599       |
      | leeftijd                               | 30           |
      | is_nederlander                         | true         |
      | woont_in_nederland                     | true         |
      | is_alleenstaande                       | true         |
      | heeft_kostendelende_medebewoners       | false        |
      | heeft_pensioengerechtigde_leeftijd_bereikt | false    |
      | is_geregistreerd_als_werkzoekende      | true         |
      | heeft_voldoende_middelen               | false        |
      | gedragscategorie                       | 2            |
    When the bijstandsaanvraag is executed for participatiewet article 43
    Then the citizen has the right to bijstand
    And the uitkering_bedrag is "0" eurocent

  # === Afwijzingsscenario's Rotterdam ===

  Scenario: Niet-Nederlander in Rotterdam zonder gelijkstelling krijgt geen bijstand
    Given a citizen with the following data:
      | gemeente_code                          | GM0599       |
      | leeftijd                               | 35           |
      | is_nederlander                         | false        |
      | is_gelijkgestelde_vreemdeling          | false        |
      | woont_in_nederland                     | true         |
      | is_alleenstaande                       | true         |
      | heeft_kostendelende_medebewoners       | false        |
      | heeft_pensioengerechtigde_leeftijd_bereikt | false    |
      | is_geregistreerd_als_werkzoekende      | true         |
      | heeft_voldoende_middelen               | false        |
      | gedragscategorie                       | 0            |
    When the bijstandsaanvraag is executed for participatiewet article 43
    Then the citizen does not have the right to bijstand
    And the reden_afwijzing contains "Nederlander"

  Scenario: Rotterdammer met voldoende middelen krijgt geen bijstand
    Given a citizen with the following data:
      | gemeente_code                          | GM0599       |
      | leeftijd                               | 35           |
      | is_nederlander                         | true         |
      | woont_in_nederland                     | true         |
      | is_alleenstaande                       | true         |
      | heeft_kostendelende_medebewoners       | false        |
      | heeft_pensioengerechtigde_leeftijd_bereikt | false    |
      | is_geregistreerd_als_werkzoekende      | true         |
      | heeft_voldoende_middelen               | true         |
      | gedragscategorie                       | 0            |
    When the bijstandsaanvraag is executed for participatiewet article 43
    Then the citizen does not have the right to bijstand
    And the reden_afwijzing contains "voldoende middelen"
