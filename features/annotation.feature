Feature: TextQuoteSelector annotation resolution
  As a legal annotation system
  I want to resolve text annotations across law versions
  So that annotations remain valid when laws change

  # Alle scenarios gebruiken ECHTE voorbeelden uit Nederlandse wetgeving
  # Bronnen zijn gedocumenteerd in RFC-004 en de officiële Staatsbladen

  Scenario: Exact match - Zorgtoeslagwet 2025 Artikel 4a
    # Bron: wetten.overheid.nl/BWBR0018451/2025-01-01
    Given law version "2025-01-01":
      """
      $id: zorgtoeslagwet
      bwb_id: BWBR0018451
      articles:
        - number: '4a'
          text: >-
            De standaardpremie voor een persoon als bedoeld in artikel 69 van
            de Zorgverzekeringswet is, in afwijking van artikel 4, gelijk aan
            het met toepassing van dat artikel bepaalde bedrag
      """
    And annotation:
      """
      target:
        selector:
          type: TextQuoteSelector
          exact: standaardpremie voor een persoon als bedoeld in artikel 69
          prefix: "De "
          suffix: " van de Zorgverzekeringswet"
      """
    When I resolve the annotation
    Then the result is FOUND with confidence 1.0
    And the match is in article "4a"

  Scenario: Article renumbered - Staatsblad 2024, 291 (Art 3 -> Art 4a)
    # Bron: zoek.officielebekendmakingen.nl/stb-2024-291.html
    # "Artikel 3 wordt vernummerd tot artikel 4a"
    Given law version "2024-01-01":
      """
      $id: zorgtoeslagwet
      bwb_id: BWBR0018451
      articles:
        - number: '3'
          text: >-
            De standaardpremie voor een persoon als bedoeld in artikel 69 van
            de Zorgverzekeringswet is, in afwijking van artikel 4, gelijk aan
            het met toepassing van dat artikel bepaalde bedrag
      """
    And annotation created on version "2025-01-01" targeting article "4a":
      """
      target:
        selector:
          type: TextQuoteSelector
          exact: standaardpremie voor een persoon als bedoeld in artikel 69
          prefix: "De "
          suffix: " van de Zorgverzekeringswet"
      """
    When I resolve the annotation against version "2024-01-01"
    Then the result is FOUND with confidence 1.0
    And the match is in article "3"

  Scenario: Text change - Staatsblad 2008, 516 (percentage 3,5 -> 2,7)
    # Bron: zoek.officielebekendmakingen.nl/stb-2008-516.html
    # "Het percentage van 3,5 wordt gewijzigd in: 2,7"
    Given law version "2008":
      """
      $id: zorgtoeslagwet
      articles:
        - number: '2'
          text: >-
            bedragen voor een verzekerde zonder toeslagpartner 3,5 procent
            van het drempelinkomen, vermeerderd met 13,7 procent van het
            toetsingsinkomen
      """
    And law version "2009":
      """
      $id: zorgtoeslagwet
      articles:
        - number: '2'
          text: >-
            bedragen voor een verzekerde zonder toeslagpartner 2,7 procent
            van het drempelinkomen, vermeerderd met 13,7 procent van het
            toetsingsinkomen
      """
    And annotation created on version "2008":
      """
      target:
        selector:
          type: TextQuoteSelector
          exact: "3,5 procent van het drempelinkomen"
          prefix: "toeslagpartner "
          suffix: ", vermeerderd"
      """
    When I resolve the annotation against version "2009"
    Then the result is FOUND with confidence above 0.7
    And the matched text contains "2,7 procent"

  Scenario: Text removed - Staatsblad 2023, 490 (Participatiewet Art 37)
    # Bron: zoek.officielebekendmakingen.nl/stb-2023-490.html
    # "In artikel 37, vierde lid, vervalt de eerste zin"
    Given law version "2023":
      """
      $id: participatiewet
      articles:
        - number: '37 lid 4'
          text: >-
            De bijstandsnorm voor gehuwden wordt met ingang van 1 januari 2023
            verhoogd. Met ingang van 1 januari 2025 wordt de bijstandsnorm
            vastgesteld.
      """
    And law version "2024":
      """
      $id: participatiewet
      articles:
        - number: '37 lid 4'
          text: >-
            Met ingang van 1 januari 2025 wordt de bijstandsnorm vastgesteld.
      """
    And annotation created on version "2023":
      """
      target:
        selector:
          type: TextQuoteSelector
          exact: De bijstandsnorm voor gehuwden wordt met ingang van 1 januari 2023 verhoogd
          prefix: ""
          suffix: ". Met ingang"
      """
    When I resolve the annotation against version "2024"
    Then the result is ORPHANED
    And no match is found

  Scenario: Ambiguous match - "verzekerde" 79x in Zorgtoeslagwet
    # Bron: wetten.overheid.nl/BWBR0018451 - woord "verzekerde" komt 79x voor
    Given law version "2025-01-01" with "verzekerde" appearing 79 times
    And annotation with insufficient context:
      """
      target:
        selector:
          type: TextQuoteSelector
          exact: verzekerde
          prefix: ""
          suffix: ""
      """
    When I resolve the annotation
    Then the result is AMBIGUOUS
    And multiple matches are found

  Scenario: Unique match with context - "verzekerde" met prefix/suffix
    # Dezelfde wet, maar nu met voldoende context
    Given law version "2025-01-01":
      """
      $id: zorgtoeslagwet
      articles:
        - number: '2'
          text: >-
            1. Indien de normpremie voor een verzekerde in het berekeningsjaar
            minder bedraagt dan de standaardpremie in dat jaar, heeft de
            verzekerde aanspraak op een zorgtoeslag ter grootte van dat verschil.
      """
    And annotation:
      """
      target:
        selector:
          type: TextQuoteSelector
          exact: verzekerde
          prefix: "heeft de "
          suffix: " aanspraak op een zorgtoeslag"
      """
    When I resolve the annotation
    Then the result is FOUND with confidence 1.0
    And exactly 1 match is found

  Scenario: Hint optimization - correct hint finds match immediately
    # Performance hint points to correct article
    Given law version "2025-01-01":
      """
      $id: zorgtoeslagwet
      articles:
        - number: '1'
          text: Definities voor de wet.
        - number: '2'
          text: >-
            De verzekerde heeft aanspraak op een zorgtoeslag ter grootte
            van het verschil tussen normpremie en standaardpremie.
        - number: '3'
          text: Slotbepalingen.
      """
    And annotation:
      """
      target:
        selector:
          type: TextQuoteSelector
          exact: zorgtoeslag
          prefix: "aanspraak op een "
          suffix: " ter grootte"
          regelrecht:hint:
            type: CssSelector
            value: "article[number='2']"
      """
    When I resolve the annotation
    Then the result is FOUND with confidence 1.0
    And the match is in article "2"

  Scenario: Hint fallback - outdated hint still finds match
    # Hint points to article 3 but text is actually in article 2
    Given law version "2025-01-01":
      """
      $id: zorgtoeslagwet
      articles:
        - number: '1'
          text: Definities voor de wet.
        - number: '2'
          text: >-
            De verzekerde heeft aanspraak op een zorgtoeslag ter grootte
            van het verschil tussen normpremie en standaardpremie.
        - number: '3'
          text: Slotbepalingen.
      """
    And annotation:
      """
      target:
        selector:
          type: TextQuoteSelector
          exact: zorgtoeslag
          prefix: "aanspraak op een "
          suffix: " ter grootte"
          regelrecht:hint:
            type: CssSelector
            value: "article[number='3']"
      """
    When I resolve the annotation
    Then the result is FOUND with confidence 1.0
    And the match is in article "2"
