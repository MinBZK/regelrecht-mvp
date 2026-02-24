Feature: Gewone Bijstand - End-to-End Scenarios
  Als burger die bijstand aanvraagt
  Wil ik weten of ik recht heb op bijstand en hoeveel
  Zodat ik kan voorzien in mijn levensonderhoud

  # Deze feature test de volledige keten van de Participatiewet:
  # - Artikel 3: Gezamenlijke huishouding
  # - Artikel 4: Huishouden type (alleenstaande/alleenstaande ouder/gehuwden)
  # - Artikel 11: Recht op bijstand
  # - Artikel 19: Voorwaarden (inkomen < norm, geen vermogen)
  # - Artikel 21: Normbedragen 21-65 jaar
  # - Artikel 22: Normbedragen 65+
  # - Artikel 22a: Kostendelersnorm
  # - Artikel 31-34: Middelen, inkomen, vermogen

  Background:
    Given the calculation date is "2026-01-15"

  # ===========================================================================
  # HUISHOUDEN TYPE SCENARIOS (Artikel 3 en 4)
  # ===========================================================================

  @huishouden @alleenstaande
  Scenario: Alleenstaande zonder kinderen wordt correct geclassificeerd
    Given a citizen with the following profile:
      | bsn                            | 123456781      |
      | is_gehuwd_of_geregistreerd     | false          |
      | heeft_medebewoner              | false          |
      | aantal_ten_laste_komende_kinderen | 0           |
      | leeftijd                       | 35             |
    When I determine the huishouden_type for participatiewet
    Then the huishouden_type is "alleenstaande"
    And is_alleenstaande is true

  @huishouden @alleenstaande_ouder
  Scenario: Alleenstaande met kinderen wordt alleenstaande ouder
    Given a citizen with the following profile:
      | bsn                            | 123456782      |
      | is_gehuwd_of_geregistreerd     | false          |
      | heeft_medebewoner              | false          |
      | aantal_ten_laste_komende_kinderen | 2           |
      | heeft_volledige_zorg_kinderen  | true           |
      | leeftijd                       | 32             |
    When I determine the huishouden_type for participatiewet
    Then the huishouden_type is "alleenstaande_ouder"
    And is_alleenstaande_ouder is true

  @huishouden @gehuwden
  Scenario: Gehuwd persoon wordt als gehuwden geclassificeerd
    Given a citizen with the following profile:
      | bsn                            | 123456783      |
      | is_gehuwd_of_geregistreerd     | true           |
      | heeft_medebewoner              | true           |
      | aantal_ten_laste_komende_kinderen | 0           |
      | leeftijd                       | 45             |
    When I determine the huishouden_type for participatiewet
    Then the huishouden_type is "gehuwden"
    And is_gehuwden is true

  @huishouden @samenwonend
  Scenario: Samenwonenden met gezamenlijke huishouding worden als gehuwden aangemerkt
    # Artikel 3 lid 2a: ongehuwd met gezamenlijke huishouding = als gehuwd
    Given a citizen with the following profile:
      | bsn                            | 123456784      |
      | is_gehuwd_of_geregistreerd     | false          |
      | heeft_medebewoner              | true           |
      | levert_bijdrage_huishouding    | true           |
      | is_bloedverwant_eerste_graad   | false          |
      | is_bloedverwant_tweede_graad_met_zorgbehoefte | false |
      | aantal_ten_laste_komende_kinderen | 0           |
      | leeftijd                       | 40             |
    When I determine the huishouden_type for participatiewet
    Then the huishouden_type is "gehuwden"
    And wordt_als_gehuwd_aangemerkt is true

  @huishouden @bloedverwant
  Scenario: Samenwonend met bloedverwant eerste graad blijft alleenstaande
    # Uitzondering artikel 3: bloedverwant eerste graad telt niet als gezamenlijke huishouding
    Given a citizen with the following profile:
      | bsn                            | 123456785      |
      | is_gehuwd_of_geregistreerd     | false          |
      | heeft_medebewoner              | true           |
      | levert_bijdrage_huishouding    | true           |
      | is_bloedverwant_eerste_graad   | true           |
      | aantal_ten_laste_komende_kinderen | 0           |
      | leeftijd                       | 28             |
    When I determine the huishouden_type for participatiewet
    Then the huishouden_type is "alleenstaande"
    And is_gezamenlijke_huishouding is false

  # ===========================================================================
  # NORMBEDRAGEN SCENARIOS (Artikel 21: 21-65 jaar)
  # ===========================================================================

  @norm @21_aow
  Scenario: Alleenstaande 21-AOW jaar krijgt juiste normbedrag
    # Artikel 21: €1.401,50 per maand voor alleenstaande (2026)
    Given a citizen with the following profile:
      | bsn                            | 123456786      |
      | leeftijd                       | 35             |
      | huishouden_type                | alleenstaande  |
    When I calculate the norm_21_65 for participatiewet
    Then the norm_21_65 is "140150" eurocent
    And valt_onder_artikel_21 is true

  @norm @21_aow
  Scenario: Alleenstaande ouder 21-AOW jaar krijgt norm
    # Artikel 21: €1.401,50 per maand voor alleenstaande ouder (2026, gelijk aan alleenstaande)
    Given a citizen with the following profile:
      | bsn                            | 123456787      |
      | leeftijd                       | 32             |
      | huishouden_type                | alleenstaande_ouder |
    When I calculate the norm_21_65 for participatiewet
    Then the norm_21_65 is "140150" eurocent

  @norm @21_aow
  Scenario: Gehuwden 21-AOW jaar krijgen gezamenlijke norm
    # Artikel 21: €2.002,13 per maand voor gehuwden (2026)
    Given a citizen with the following profile:
      | bsn                            | 123456788      |
      | leeftijd                       | 42             |
      | leeftijd_partner               | 40             |
      | huishouden_type                | gehuwden       |
    When I calculate the norm_21_65 for participatiewet
    Then the norm_21_65 is "200213" eurocent

  @norm @21_65
  Scenario: Persoon van 20 jaar valt niet onder artikel 21
    Given a citizen with the following profile:
      | bsn                            | 123456789      |
      | leeftijd                       | 20             |
      | huishouden_type                | alleenstaande  |
    When I calculate the norm_21_65 for participatiewet
    Then valt_onder_artikel_21 is false

  # ===========================================================================
  # KOSTENDELERSNORM SCENARIOS (Artikel 22a)
  # ===========================================================================

  @kostendeler
  Scenario: Twee kostendelers - norm wordt verlaagd
    # Formule: ((A - B) x (1 - (1/3 x ((M-1)/M))) + B) / M
    # Met M=2, A=norm gehuwden, B=65% van A
    Given a citizen with the following profile:
      | bsn                            | 123456790      |
      | leeftijd                       | 35             |
      | aantal_meerderjarige_medebewoners | 2           |
      | heeft_alleen_commerciele_relaties | false       |
      | verblijft_in_inrichting        | false          |
      | heeft_alleen_studerende_medebewoners | false    |
      | norm_gehuwden                  | 200213         |
    When I calculate the kostendelersnorm for participatiewet
    Then valt_onder_kostendelersnorm is true
    And aantal_kostendelers is 2

  @kostendeler
  Scenario: Drie kostendelers - norm wordt verder verlaagd
    Given a citizen with the following profile:
      | bsn                            | 123456791      |
      | leeftijd                       | 40             |
      | aantal_meerderjarige_medebewoners | 3           |
      | heeft_alleen_commerciele_relaties | false       |
      | verblijft_in_inrichting        | false          |
      | heeft_alleen_studerende_medebewoners | false    |
      | norm_gehuwden                  | 200213         |
    When I calculate the kostendelersnorm for participatiewet
    Then valt_onder_kostendelersnorm is true
    And aantal_kostendelers is 3

  @kostendeler @uitzondering
  Scenario: Commerciele relatie - kostendelersnorm niet van toepassing
    # Artikel 22a lid 2a: uitzondering voor commerciele relaties
    Given a citizen with the following profile:
      | bsn                            | 123456792      |
      | leeftijd                       | 35             |
      | aantal_meerderjarige_medebewoners | 2           |
      | heeft_alleen_commerciele_relaties | true        |
      | verblijft_in_inrichting        | false          |
      | heeft_alleen_studerende_medebewoners | false    |
      | norm_gehuwden                  | 200213         |
    When I calculate the kostendelersnorm for participatiewet
    Then valt_onder_kostendelersnorm is false

  @kostendeler @uitzondering
  Scenario: Persoon in inrichting - kostendelersnorm niet van toepassing
    # Artikel 22a lid 2b: uitzondering voor verblijf in inrichting
    Given a citizen with the following profile:
      | bsn                            | 123456793      |
      | leeftijd                       | 35             |
      | aantal_meerderjarige_medebewoners | 2           |
      | heeft_alleen_commerciele_relaties | false       |
      | verblijft_in_inrichting        | true           |
      | heeft_alleen_studerende_medebewoners | false    |
      | norm_gehuwden                  | 200213         |
    When I calculate the kostendelersnorm for participatiewet
    Then valt_onder_kostendelersnorm is false

  @kostendeler @uitzondering
  Scenario: Alleen studerende medebewoners en 21+ - kostendelersnorm niet van toepassing
    # Artikel 22a lid 2c: uitzondering voor 21+ met alleen studerende medebewoners
    Given a citizen with the following profile:
      | bsn                            | 123456794      |
      | leeftijd                       | 25             |
      | aantal_meerderjarige_medebewoners | 2           |
      | heeft_alleen_commerciele_relaties | false       |
      | verblijft_in_inrichting        | false          |
      | heeft_alleen_studerende_medebewoners | true     |
      | norm_gehuwden                  | 200213         |
    When I calculate the kostendelersnorm for participatiewet
    Then valt_onder_kostendelersnorm is false

  @kostendeler @uitzondering
  Scenario: Persoon jonger dan 21 - kostendelersnorm niet van toepassing
    # Artikel 22a lid 2d: uitzondering voor personen < 21 jaar
    Given a citizen with the following profile:
      | bsn                            | 123456795      |
      | leeftijd                       | 19             |
      | aantal_meerderjarige_medebewoners | 2           |
      | heeft_alleen_commerciele_relaties | false       |
      | verblijft_in_inrichting        | false          |
      | heeft_alleen_studerende_medebewoners | false    |
      | norm_gehuwden                  | 200213         |
    When I calculate the kostendelersnorm for participatiewet
    Then valt_onder_kostendelersnorm is false

  # ===========================================================================
  # RECHT OP BIJSTAND SCENARIOS (Artikel 11)
  # ===========================================================================

  @recht @toekenning
  Scenario: Nederlandse burger zonder middelen heeft recht op bijstand
    Given a citizen with the following profile:
      | bsn                            | 123456796      |
      | is_nederlander                 | true           |
      | woont_in_nederland             | true           |
      | heeft_onvoldoende_middelen     | true           |
    When I check heeft_recht_op_bijstand for participatiewet
    Then heeft_recht_op_bijstand is true

  @recht @toekenning
  Scenario: Vreemdeling met rechtmatig verblijf heeft recht op bijstand
    # Artikel 11 lid 2: gelijkstelling met Nederlander
    Given a citizen with the following profile:
      | bsn                            | 123456797      |
      | is_nederlander                 | false          |
      | heeft_rechtmatig_verblijf      | true           |
      | woont_in_nederland             | true           |
      | heeft_onvoldoende_middelen     | true           |
    When I check heeft_recht_op_bijstand for participatiewet
    Then heeft_recht_op_bijstand is true

  @recht @afwijzing
  Scenario: Burger met voldoende middelen heeft geen recht
    Given a citizen with the following profile:
      | bsn                            | 123456798      |
      | is_nederlander                 | true           |
      | woont_in_nederland             | true           |
      | heeft_onvoldoende_middelen     | false          |
    When I check heeft_recht_op_bijstand for participatiewet
    Then heeft_recht_op_bijstand is false

  @recht @afwijzing
  Scenario: Burger die niet in Nederland woont heeft geen recht
    Given a citizen with the following profile:
      | bsn                            | 123456799      |
      | is_nederlander                 | true           |
      | woont_in_nederland             | false          |
      | heeft_onvoldoende_middelen     | true           |
    When I check heeft_recht_op_bijstand for participatiewet
    Then heeft_recht_op_bijstand is false

  @recht @afwijzing
  Scenario: Niet-Nederlander zonder rechtmatig verblijf heeft geen recht
    Given a citizen with the following profile:
      | bsn                            | 123456800      |
      | is_nederlander                 | false          |
      | heeft_rechtmatig_verblijf      | false          |
      | woont_in_nederland             | true           |
      | heeft_onvoldoende_middelen     | true           |
    When I check heeft_recht_op_bijstand for participatiewet
    Then heeft_recht_op_bijstand is false

  # ===========================================================================
  # UITSLUITINGSGRONDEN SCENARIOS (Artikel 13)
  # ===========================================================================

  @uitsluiting
  Scenario: Gedetineerde is uitgesloten van bijstand
    # Artikel 13 lid 1a: rechtens vrijheid ontnomen
    Given a citizen with the following profile:
      | bsn                            | 123456801      |
      | leeftijd                       | 35             |
      | is_gedetineerd                 | true           |
      | vervult_dienstplicht           | false          |
      | is_stakend                     | false          |
      | weken_buitenland               | 0              |
    When I check is_uitgesloten_van_bijstand for participatiewet
    Then is_uitgesloten_van_bijstand is true

  @uitsluiting
  Scenario: Persoon jonger dan 18 is uitgesloten van bijstand
    # Artikel 13 lid 1e: jonger dan 18 jaar
    Given a citizen with the following profile:
      | bsn                            | 123456802      |
      | leeftijd                       | 17             |
      | is_gedetineerd                 | false          |
      | vervult_dienstplicht           | false          |
      | is_stakend                     | false          |
      | weken_buitenland               | 0              |
    When I check is_uitgesloten_van_bijstand for participatiewet
    Then is_uitgesloten_van_bijstand is true

  @uitsluiting
  Scenario: Burger langer dan 4 weken in buitenland is uitgesloten
    # Artikel 13 lid 1d: langer dan 4 weken buitenland
    Given a citizen with the following profile:
      | bsn                            | 123456803      |
      | leeftijd                       | 35             |
      | is_gedetineerd                 | false          |
      | vervult_dienstplicht           | false          |
      | is_stakend                     | false          |
      | weken_buitenland               | 6              |
    When I check is_uitgesloten_van_bijstand for participatiewet
    Then is_uitgesloten_van_bijstand is true

  @uitsluiting
  Scenario: 65-plusser mag 13 weken in buitenland
    # Artikel 13 lid 4: uitzondering voor 65+
    Given a citizen with the following profile:
      | bsn                            | 123456804      |
      | leeftijd                       | 67             |
      | is_gedetineerd                 | false          |
      | vervult_dienstplicht           | false          |
      | is_stakend                     | false          |
      | weken_buitenland               | 10             |
    When I check is_uitgesloten_van_bijstand for participatiewet
    Then is_uitgesloten_van_bijstand is false

  @uitsluiting
  Scenario: 65-plusser langer dan 13 weken in buitenland is wel uitgesloten
    Given a citizen with the following profile:
      | bsn                            | 123456805      |
      | leeftijd                       | 68             |
      | is_gedetineerd                 | false          |
      | vervult_dienstplicht           | false          |
      | is_stakend                     | false          |
      | weken_buitenland               | 15             |
    When I check is_uitgesloten_van_bijstand for participatiewet
    Then is_uitgesloten_van_bijstand is true

  # ===========================================================================
  # VOORWAARDEN BIJSTAND SCENARIOS (Artikel 19)
  # ===========================================================================

  @voorwaarden
  Scenario: Inkomen onder norm en geen vermogen geeft recht
    Given a citizen with the following profile:
      | bsn                            | 123456806      |
      | inkomen                        | 50000          |
      | bijstandsnorm                  | 140150         |
      | vermogen                       | 0              |
      | vermogensgrens                 | 800000         |
    When I check heeft_recht_op_algemene_bijstand for participatiewet
    Then heeft_recht_op_algemene_bijstand is true
    And heeft_onvoldoende_middelen is true

  @voorwaarden
  Scenario: Inkomen boven norm geeft geen recht
    Given a citizen with the following profile:
      | bsn                            | 123456807      |
      | inkomen                        | 150000         |
      | bijstandsnorm                  | 140150         |
      | vermogen                       | 0              |
      | vermogensgrens                 | 800000         |
    When I check heeft_recht_op_algemene_bijstand for participatiewet
    Then heeft_recht_op_algemene_bijstand is false

  @voorwaarden
  Scenario: Vermogen boven grens geeft geen recht
    Given a citizen with the following profile:
      | bsn                            | 123456808      |
      | inkomen                        | 50000          |
      | bijstandsnorm                  | 140150         |
      | vermogen                       | 850000         |
      | vermogensgrens                 | 800000         |
    When I check heeft_recht_op_algemene_bijstand for participatiewet
    Then heeft_recht_op_algemene_bijstand is false

  @voorwaarden
  Scenario: Bijstand bedrag is verschil tussen norm en inkomen
    # Artikel 19 lid 2: bijstand = bijstandsnorm - inkomen
    Given a citizen with the following profile:
      | bsn                            | 123456809      |
      | inkomen                        | 40000          |
      | bijstandsnorm                  | 140150         |
      | vermogen                       | 0              |
      | vermogensgrens                 | 800000         |
    When I check heeft_recht_op_algemene_bijstand for participatiewet
    Then the bijstand_bedrag is "100150" eurocent

  # ===========================================================================
  # NORMEN IN INRICHTING SCENARIOS (Artikel 23)
  # ===========================================================================

  @inrichting
  Scenario: Alleenstaande in inrichting krijgt lagere norm
    # Artikel 23: €443,76 per maand voor alleenstaande in inrichting (2026)
    Given a citizen with the following profile:
      | bsn                            | 123456810      |
      | verblijft_in_inrichting        | true           |
      | partner_verblijft_in_inrichting | false         |
      | huishouden_type                | alleenstaande  |
    When I calculate the norm_inrichting for participatiewet
    Then the norm_inrichting is "44376" eurocent
    And valt_onder_artikel_23 is true

  @inrichting
  Scenario: Gehuwden beiden in inrichting krijgen gezamenlijke norm
    # Artikel 23: €690,27 per maand voor gehuwden in inrichting (2026)
    Given a citizen with the following profile:
      | bsn                            | 123456811      |
      | verblijft_in_inrichting        | true           |
      | partner_verblijft_in_inrichting | true          |
      | huishouden_type                | gehuwden       |
    When I calculate the norm_inrichting for participatiewet
    Then the norm_inrichting is "69027" eurocent

  @inrichting
  Scenario: Gehuwden waarvan een in inrichting krijgen som van alleenstaande normen
    # Artikel 23 lid 2: som van normen als alleenstaande
    Given a citizen with the following profile:
      | bsn                            | 123456812      |
      | verblijft_in_inrichting        | true           |
      | partner_verblijft_in_inrichting | false         |
      | huishouden_type                | gehuwden       |
    When I calculate the norm_inrichting for participatiewet
    # 2 x €443,76 = €887,52
    Then the norm_inrichting is "88752" eurocent
