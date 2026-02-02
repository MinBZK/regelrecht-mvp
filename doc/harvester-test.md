# Harvester Test Document

## Testaanpak

Dit document beschrijft de systematische testaanpak voor de Rust harvester. Het doel is om de harvester te valideren tegen 10 representatieve Nederlandse wetten voordat we deze uitrollen voor alle wetten.

### Prompt voor Claude Sessie

```
Je gaat de Rust harvester testen tegen Nederlandse wetten. Volg deze stappen:

## Instructies

1. **Selecteer een wet** uit de testlijst hieronder die nog status "pending" heeft
2. **Update de status** naar "in_progress" in dit document
3. **Harvest de wet** met de harvester:
   ```bash
   cd packages
   cargo run --release -p regelrecht-harvester -- download <BWB_ID> --date 2025-01-01 --output ../regulation/
   ```
4. **Review het resultaat** kritisch. Let op:
   - **Volledigheid**: Zijn alle artikelen aanwezig? Geen ontbrekende content?
   - **Artikelnummering**: Kloppen de nummers (1, 1.1, 1.1.a, etc.)?
   - **Tekst correctheid**: Is de tekst volledig en correct geformatteerd?
   - **Definities**: Zijn definities met sub-items compleet?
   - **Referenties**: Zijn cross-references correct geconverteerd naar markdown links?
   - **Formatting**: Geen rare spaties, broken markdown, of encoding issues?

5. **Rapporteer bevindingen** in de "Bevindingen" sectie van de betreffende wet
6. **Fix eventuele issues** in de harvester code
7. **Documenteer de oplossing** in de "Acties" sectie
8. **Harvest opnieuw** en herhaal review

**Herhaal stap 4-8 maximaal 3 keer** of tot er geen problemen meer zijn.

## Kwaliteitsstandaard

- **Minor issues zijn ook issues** die gefixt moeten worden
- De output moet **perfect** zijn
- We bouwen een basis voor alle Nederlandse wetgeving - kwaliteit is kritiek
```

---

## Worktree

```bash
cd C:\Users\timde\Documents\Code\regelrecht-mvp\.worktrees\feature\rust-harvester
```

---

## Testlijst

| # | Status | BWB ID | Wet | Reden voor selectie |
|---|--------|--------|-----|---------------------|
| 1 | issues_found | BWBR0005537 | Algemene wet bestuursrecht (Awb) | Zeer groot, veel hoofdstukken en bijlagen |
| 2 | issues_found | BWBR0011823 | Vreemdelingenwet 2000 | Complexe cross-references |
| 3 | passed | BWBR0035362 | Wet maatschappelijke ondersteuning 2015 | Delegatie naar gemeenten |
| 4 | issues_found | BWBR0015703 | Participatiewet | Complexe formules en berekeningen |
| 5 | passed | BWBR0002656 | Burgerlijk Wetboek Boek 1 | Klassiek burgerlijk recht |
| 6 | issues_found | BWBR0001854 | Wetboek van Strafrecht | Historisch, veel wijzigingen |
| 7 | issues_found | BWBR0025028 | Mediawet 2008 | Technische definities |
| 8 | blocked | BWBR0020368 | Wet op het financieel toezicht (Wft) | Zeer groot, complexe definities |
| 9 | issues_found | BWBR0002629 | Wet op de omzetbelasting 1968 | EU references, tarieftabellen |
| 10 | issues_found | BWBR0002221 | Algemene Ouderdomswet (AOW) | Datum-gebaseerde berekeningen |

**Status legenda:**
- `pending` - Nog niet getest
- `in_progress` - Test loopt
- `issues_found` - Issues gevonden, bezig met fixen
- `passed` - Test geslaagd, geen issues
- `blocked` - Geblokkeerd door ander issue

---

## Test Resultaten

### 1. Algemene wet bestuursrecht (Awb) - BWBR0005537

**Status:** issues_found

**Test iteraties:**

#### Iteratie 1
- **Datum:** 2026-02-02
- **Bevindingen:**
  1. **KRITIEK - Dubbele artikelnummers in Bijlagen**: De harvester produceert dubbele artikelnummers in Bijlage 2 en Bijlage 3. Dit komt doordat meerdere aparte opsommingen binnen hetzelfde artikel elk hun eigen a/b/c sub-items hebben.
     - Duplicaten gevonden: B2:1.a-h (2x), B2:2.a-f (2x), B2:4.a-c (2x), B2:7.a-b (2x), B2:9.a-c (2x), B2:11.a-b (2x), B3:2.a-c (2x)
     - **Root cause**: Binnen één artikel kunnen meerdere `<lijst>` elementen voorkomen die elk hun eigen enumeratie starten (a, b, c...). De harvester combineert de artikelnummer met de lijst-item nummer zonder te onderscheiden tussen verschillende lijsten.

  2. **KRITIEK - Dubbele leden in artikel 8:36c**: Dit artikel heeft twee versies (gefaseerde inwerkingtreding digitaal procederen), beide met leden 1-4.
     - Duplicaten: 8:36c.1, 8:36c.2, 8:36c.3, 8:36c.4 (elk 2x)
     - **Root cause**: Het artikel heeft een transitie-opmerking gevolgd door de nieuwe versie van de leden.

  3. **Positief - Cross-references correct**: De markdown links naar andere wetten zijn correct geconverteerd (bijv. `[artikel 9, eerste lid, van de Wet Nationale ombudsman][ref1]`)

  4. **Positief - Artikelnummering hoofdwet**: De reguliere artikelen (niet-bijlagen) hebben correcte hiërarchische nummering (bijv. 1:1.1.a, 3:10.4, 8:109.1.b)

- **Acties:**
  - Issue #1 en #2 vereisen aanpassingen in de splitting logic om:
    a. Meerdere lijsten binnen een artikel te onderscheiden (bijv. door een volgnummer toe te voegen)
    b. Gefaseerde inwerkingtreding/transitie-artikelen correct te verwerken

**Eindresultaat:** Wacht op fix voor duplicate number issues

---

### 2. Vreemdelingenwet 2000 - BWBR0011823

**Status:** issues_found

**Opmerking:** Datum 2025-01-01 niet beschikbaar in repository, getest met 2024-01-01.

**Test iteraties:**

#### Iteratie 1
- **Datum:** 2026-02-02
- **Bevindingen:**
  1. **KRITIEK - 42x dubbel artikelnummer '1'**: Artikel 1 bevat definities die elk als apart artikel worden geëxtraheerd, maar allemaal nummer '1' krijgen zonder sub-nummering.
     - Alle 42 definities krijgen nummer '1'
     - Elk item is een aparte definitie (bijv. "aanvullend document:", "ambtenaren belast met de grensbewaking:", etc.)
     - **Root cause**: De definities in artikel 1 gebruiken een ongemarkeerde lijst (`<lijst type="ongemarkeerd">`) zonder expliciete sub-markers (a, b, c). De harvester splitst deze naar aparte artikelen maar kan geen unieke nummering genereren.

  2. **Positief - Overige artikelen correct genummerd**: Artikelen 2 en verder hebben correcte hiërarchische nummering (2.1, 2.2, 20.1.a, etc.)

  3. **Positief - Cross-references correct**: Interne en externe verwijzingen zijn correct omgezet naar markdown links met references

- **Acties:**
  - Issue #3 vereist aanpassing: ongemarkeerde lijsten zonder markers moeten een volgnummer krijgen (bijv. 1.1, 1.2, 1.3... of 1-def1, 1-def2...)

**Eindresultaat:** Wacht op fix voor definitions numbering

---

### 3. Wet maatschappelijke ondersteuning 2015 - BWBR0035362

**Status:** passed

**Test iteraties:**

#### Iteratie 1
- **Datum:** 2026-02-02
- **Bevindingen:**
  1. **Geen duplicaten**: Alle artikelnummers zijn uniek
  2. **Correcte hiërarchische nummering**: Artikelen volgen het 1.1.1.1, 1.1.2.4.c patroon correct
  3. **Definities goed geformatteerd**: Artikel 1.1.1.1 bevat alle definities in één blok met markdown bullet points (- *term:* definitie)
  4. **Cross-references correct**: Verwijzingen naar andere wetten (Vreemdelingenwet, Richtlijn 2004/38/EG) zijn correct omgezet

- **Acties:** Geen actie nodig

**Eindresultaat:** PASSED - Geen issues gevonden

---

### 4. Participatiewet - BWBR0015703

**Status:** issues_found

**Test iteraties:**

#### Iteratie 1
- **Datum:** 2026-02-02
- **Bevindingen:**
  1. **KRITIEK - Dubbel artikelnummer 22a.1**: Artikel 22a bevat een formule met variabele-definities (A, B) die elk als apart artikel worden geëxtraheerd met nummer "22a.1".
     - 3x "22a.1" in output
     - Dit is dezelfde issue als #3 (Vreemdelingenwet) - ongemarkeerde lijst items krijgen geen unieke sub-nummering

  2. **Positief - Definities in artikel 1 correct genummerd**: Anders dan de Vreemdelingenwet, heeft de Participatiewet expliciet geletterde definities (1.a, 1.b, 1.c, etc.)

  3. **Positief - Cross-references correct**: Verwijzingen naar andere artikelen en wetten zijn correct omgezet

- **Acties:** Issue #3 fix zal ook dit probleem oplossen

**Eindresultaat:** Wacht op fix voor unmarked list numbering (issue #3)

---

### 5. Burgerlijk Wetboek Boek 1 - BWBR0002656

**Status:** passed

**Opmerking:** BWB ID gecorrigeerd van BWBR0001840 (Grondwet) naar BWBR0002656. Datum 2025-01-01 niet beschikbaar, getest met 2024-01-01.

**Test iteraties:**

#### Iteratie 1
- **Datum:** 2026-02-02
- **Bevindingen:**
  1. **Geen duplicaten**: Alle 1737 artikelen hebben unieke nummers
  2. **Correcte hiërarchische nummering**: Artikelen volgen 1.1, 1.2, 3.1, 4.4 patroon correct
  3. **Cross-references correct**: Verwijzingen naar andere artikelen binnen dezelfde wet zijn correct

- **Acties:** Geen actie nodig

**Eindresultaat:** PASSED - Geen issues gevonden

---

### 6. Wetboek van Strafrecht - BWBR0001854

**Status:** issues_found

**Test iteraties:**

#### Iteratie 1
- **Datum:** 2026-02-02
- **Bevindingen:**
  1. **KRITIEK - Artikel vs lid naamconflict**: 420bis.1 en 420quater.1 komen dubbel voor. Dit is een legitiem probleem in de wet-structuur: er bestaan zowel "artikel 420bis lid 1" als een apart "artikel 420bis.1".
     - 420bis.1 (2x): lid 1 van 420bis VS apart artikel 420bis.1 (eenvoudig witwassen)
     - 420quater.1 (2x): lid 1 van 420quater VS apart artikel 420quater.1 (eenvoudig schuldwitwassen)
     - **Root cause**: De wet heeft artikelen die eindigen op ".1" als volledige artikelnaam, wat conflicteert met de lid-nummering

  2. **Issue - Dash markers niet genummerd**: Artikel 421.1.b heeft sub-items met dash (–) als marker. Alle 4 krijgen dezelfde nummer "421.1.b.–"
     - Dit is gerelateerd aan issue #3 (ongemarkeerde lijsten)

  3. **Positief - Overige artikelen correct**: De meeste artikelen (1927 totaal) zijn correct genummerd

- **Acties:**
  - Issue 1 vereist mogelijk een andere naamgevingsconventie om leden te onderscheiden van artikel-subnummers (bijv. artikel_lid ipv artikel.lid)
  - Issue 2 valt onder issue #3 oplossing

#### Iteratie 2
- **Datum:** 2026-02-02
- **Bevindingen:**
  1. **FIXED - Dash markers nu inline**: Issue #3 is opgelost. Lijsten waar ALLE items non-addressable markers hebben (dash, bullet, leeg) worden nu inline gehouden in het parent artikel ipv gesplitst.
     - Artikel 421.1.b bevat nu alle dash-items inline in de tekst
     - Geen duplicate artikelnummers meer voor dash-marked lists

  2. **OPEN - Artikel vs lid naamconflict**: Issue #4 blijft open (420bis.1, 420quater.1)

- **Acties:** Issue #3 is opgelost, Issue #4 blijft open

**Eindresultaat:** Wacht op fix voor artikel/lid conflict (issue #4)

---

### 7. Mediawet 2008 - BWBR0025028

**Status:** issues_found

**Opmerking:** BWB ID gecorrigeerd van BWBR0006004 naar BWBR0025028. Datum 2025-01-01 niet beschikbaar, getest met 2024-01-01.

**Test iteraties:**

#### Iteratie 1
- **Datum:** 2026-02-02
- **Bevindingen:**
  1. **Issue - Dubbel artikelnummer 2.88.5**: Artikel 2.88 lid 5 komt 2x voor met verschillende teksten
     - Eerste versie: over voorkomen van aanzetten tot geweld/haat
     - Tweede versie: over reflectie in jaarverslag over journalistieke deontologie
     - **Root cause**: Waarschijnlijk gefaseerde inwerkingtreding of transitie-artikel (vergelijkbaar met Awb issue #2)

  2. **Positief - 1627 artikelen geëxtraheerd**: Meeste artikelen correct genummerd

- **Acties:** Issue valt onder issue #2 (gefaseerde inwerkingtreding)

**Eindresultaat:** Wacht op fix voor phased implementation articles

---

### 8. Wet op het financieel toezicht (Wft) - BWBR0020368

**Status:** blocked

**Test iteraties:**

#### Iteratie 1
- **Datum:** 2026-02-02
- **Bevindingen:**
  1. **BLOCKED - XML te groot**: De Wft XML is 52.6 MB, wat de 50 MB limiet overschrijdt
     - Error: "HTTP response too large: 52626772 bytes exceeds limit of 52428800 bytes"
     - De Wft is een van de grootste Nederlandse wetten

- **Acties:**
  - Verhoog HTTP response limiet in harvester, of
  - Onderzoek streaming/chunked download mogelijkheden

**Eindresultaat:** BLOCKED - Kan niet getest worden door size limit

---

### 9. Wet op de omzetbelasting 1968 - BWBR0002629

**Status:** issues_found

**Opmerking:** BWB ID gecorrigeerd van BWBR0003245 (Wet milieubeheer) naar BWBR0002629.

**Test iteraties:**

#### Iteratie 1
- **Datum:** 2026-02-02
- **Bevindingen:**
  1. **Issue - Dash markers in meerdere artikelen**: Artikelen 21b en 2a hebben sub-items met dash (–) markers die duplicaten veroorzaken
     - 21b.1.b.– (meerdere keren)
     - 21b.1.c.– (meerdere keren)
     - 21b.4.a.– (meerdere keren)
     - 21b.4.b.– (meerdere keren)
     - 2a.1.s.2°.– (meerdere keren)
     - **Root cause**: Zelfde als issue #3 - dash-gemarkeerde lijst items krijgen geen unieke volgnummer

  2. **Positief - 1038 artikelen geëxtraheerd**: Meeste artikelen correct

- **Acties:** Issue valt onder issue #3

#### Iteratie 2
- **Datum:** 2026-02-02
- **Bevindingen:**
  1. **FIXED - Dash markers nu inline**: Issue #3 is opgelost. Lijsten met alleen dash-markers worden nu inline gehouden.
     - Artikelen 21b en 2a bevatten nu dash-items inline in de tekst
     - Geen duplicate artikelnummers meer voor dash-marked lists

- **Acties:** Issue #3 is opgelost

**Eindresultaat:** Hertest nodig om te verifiëren dat alle issues opgelost zijn

---

### 10. Algemene Ouderdomswet (AOW) - BWBR0002221

**Status:** issues_found

**Opmerking:** BWB ID gecorrigeerd van BWBR0002629 (Wet OB) naar BWBR0002221.

**Test iteraties:**

#### Iteratie 1
- **Datum:** 2026-02-02
- **Bevindingen:**
  1. **Issue - Lege artikelnummers voor vervallen artikelen**: Vervallen artikelen (tekst: "Vervallen") krijgen een leeg nummer ''
     - Artikel 4: number: '' (zou '4' moeten zijn)
     - Artikel 16a: number: '' (zou '16a' moeten zijn)
     - **Root cause**: Vervallen artikelen missen hun nummer-element in de XML maar behouden hun URL referentie

  2. **Issue - Dash markers**: Artikel 11.2 heeft sub-items met dash markers
     - 11.2.– (meerdere keren)
     - Zelfde issue als #3

  3. **Positief - 385 artikelen geëxtraheerd**: Meeste artikelen correct genummerd

- **Acties:**
  - Issue 1 vereist fallback naar artikel-nummer uit URL/structuur wanneer nummer-element ontbreekt
  - Issue 2 valt onder issue #3

#### Iteratie 2
- **Datum:** 2026-02-02
- **Bevindingen:**
  1. **OPEN - Lege artikelnummers**: Issue #6 blijft open (vervallen artikelen)

  2. **FIXED - Dash markers nu inline**: Issue #3 is opgelost. Artikel 11.2 bevat nu alle dash-items inline.

- **Acties:** Issue #6 blijft open

**Eindresultaat:** Wacht op fix voor empty numbers (issue #6)

---

## Samenvatting

| Wet | Status | Iteraties | Issues gevonden | Issues opgelost |
|-----|--------|-----------|-----------------|-----------------|
| Awb | issues_found | 1 | 2 | 0 |
| Vreemdelingenwet 2000 | issues_found | 1 | 1 | 0 |
| Wmo 2015 | passed | 1 | 0 | 0 |
| Participatiewet | issues_found | 1 | 1 | 0 |
| BW Boek 1 | passed | 1 | 0 | 0 |
| Wetboek van Strafrecht | issues_found | 2 | 2 | 1 |
| Mediawet 2008 | issues_found | 1 | 1 | 0 |
| Wft | blocked | 1 | 1 | 0 |
| Wet OB 1968 | issues_found | 2 | 1 | 1 |
| AOW | issues_found | 2 | 2 | 1 |

**Totaal:** 10/10 wetten getest, 2 passed, 7 issues_found, 1 blocked. Issue #3 (dash-marker lijsten) opgelost.

---

## Bekende Issues Log

Hier worden alle gevonden issues bijgehouden met hun status.

| Issue # | Wet(ten) | Beschrijving | Status | Commit |
|---------|----------|--------------|--------|--------|
| 1 | Awb | Dubbele artikelnummers bij meerdere lijsten in bijlagen | fixed | - |
| 2 | Awb, Mediawet | Dubbele leden bij gefaseerde inwerkingtreding | fixed | - |
| 3 | WvS, Wet OB, AOW | Dash-marker lijsten worden gesplitst ipv inline gehouden | fixed | - |
| 4 | WvS | Artikel vs lid naamconflict (420bis.1 vs 420bis lid 1) | open | - |
| 5 | Wft | XML response te groot (>50MB) | open | - |
| 6 | AOW | Vervallen artikelen krijgen leeg nummer | fixed | - |

**Issue status:** `open`, `in_progress`, `fixed`, `wont_fix`

---

## Aanbevolen Fix Prioriteit

1. ~~**Issue #3** (Ongemarkeerde lijsten) - Hoogste impact, raakt 5 wetten~~ **FIXED** - Dash-marker lijsten worden nu inline gehouden
2. ~~**Issue #2** (Gefaseerde inwerkingtreding) - Raakt 2 wetten, structureel probleem~~ **FIXED** - Multi-versie artikelen worden als één component gehouden, redactioneel commentaar geëxcludeerd
3. ~~**Issue #6** (Lege nummers vervallen artikelen) - Eenvoudige fix~~ **FIXED** - Nummer uit label attribuut gehaald als kop/nr ontbreekt
4. ~~**Issue #1** (Meerdere lijsten in artikel) - Complex, raakt bijlagen~~ **FIXED** - Meerdere sibling `<lijst>` elementen worden nu inline gehouden ipv gesplitst
5. **Issue #4** (Artikel/lid conflict) - Edge case in WvS
6. **Issue #5** (HTTP size limit) - Configuratie aanpassing
