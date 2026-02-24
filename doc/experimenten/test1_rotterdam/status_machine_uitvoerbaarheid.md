# Status Machine-Uitvoerbaarheid Rotterdam Participatiewet - Test 1

*Datum: 2026-02-24*
*Experiment: test1_rotterdam*
*Voortbouwend op: test0_rotterdam*

---

## 1. Doelstelling

Omzetting van alle YAML-bestanden naar de MVP schema versie en uitbreiding van de machine-uitvoerbare logica conform de aanbevelingen uit test0_rotterdam.

## 2. Uitgevoerde wijzigingen

### 2.1 Schema migratie

**Schema identifier gewijzigd naar MVP:**
- Van: `https://raw.githubusercontent.com/MinBZK/poc-machine-law/refs/heads/main/schema/v0.3.0/schema.json`
- Naar: `https://regelrecht.nl/schema/v0.3.0/schema.json`

**Bijgewerkte bestanden:**
- `schema/v0.2.0/schema.json` - `$id` bijgewerkt
- `schema/v0.3.0/schema.json` - `$id` bijgewerkt
- 21 YAML-bestanden in `regulation/nl/**/*.yaml` - `$schema` referentie bijgewerkt

### 2.2 PiB Fase 1 - Inkomensvrijlating bij werk (art. 31 Pw)

**Bestand:** `regulation/nl/wet/participatiewet/2025-01-01.yaml`

Uitgebreid met Participatiewet in Balans fase 1 wijzigingen:

| Parameter | Oud (pre-PiB) | Nieuw (PiB fase 1) |
|-----------|---------------|---------------------|
| MAX_MAANDEN_ARBEIDSVRIJLATING | 6 | 30 |
| VRIJLATING_ARBEID_PERCENTAGE (standaard) | 25% | 25% |
| VRIJLATING_ARBEID_PERCENTAGE_ALLEENSTAANDE_OUDER | - | 37.5% (+12.5% extra) |
| VRIJLATING_ARBEID_PERCENTAGE_MEDISCH_URENBEPERKT | - | 15% |
| MAX_VRIJLATING_ARBEID_MAAND | €285 | €253 |

**Nieuwe inputs toegevoegd:**
- `huishoudtype` - Type huishouden voor bepaling percentage
- `is_medisch_urenbeperkt` - Indicatie art. 6b Pw (UWV-bepaling)

**Nieuwe acties:**
- `vrijlating_percentage` - Dynamische berekening percentage op basis van type

### 2.3 Daklozenkorting (art. 27 Pw)

**Bestand:** `regulation/nl/wet/participatiewet/2025-01-01.yaml`

Toegevoegd: machine-uitvoerbare logica voor standaard 20% normverlaging bij dakloosheid (gangbare praktijk Rotterdam).

| Element | Waarde |
|---------|--------|
| DAKLOZENKORTING_PERCENTAGE | 20% |
| endpoint | `daklozenkorting` |
| Input | `is_dakloos` (human_input) |
| Output | `verlaging_woonsituatie`, `aangepaste_norm` |

**Noot:** Art. 27 blijft `requires_human_assessment: true` voor andere woonsituaties dan dakloosheid.

### 2.4 Terugwerkende kracht (art. 44 Pw)

**Bestand:** `regulation/nl/wet/participatiewet/2025-01-01.yaml`

Toegevoegd: PiB fase 1 mogelijkheid voor terugwerkende kracht tot 3 maanden voor meldingsdatum.

| Element | Waarde |
|---------|--------|
| MAX_TERUGWERKENDE_KRACHT_MAANDEN | 3 |
| Nieuwe input | `was_bijstandsbehoeftig_voor_melding` (human_input) |
| Nieuwe input | `datum_bijstandsbehoefte_ontstaan` (human_input) |
| Nieuwe output | `heeft_terugwerkende_kracht` |
| Nieuwe output | `vroegste_ingangsdatum` |

**Logica:** Indien aanvrager aannemelijk maakt dat hij al bijstandsbehoeftig was voor de meldingsdatum, kan de ingangsdatum worden bepaald op de datum van bijstandsbehoefte, met een maximum van 3 maanden voor de meldingsdatum.

### 2.5 CVDR Downloads Rotterdam Uitvoeringsbeleid

Drie beleidsregelingen gedownload en geconverteerd naar YAML:

| CVDR ID | Titel | Artikelen | Geldig vanaf |
|---------|-------|-----------|--------------|
| CVDR719087 | Beleidsregels bijzondere bijstand Rotterdam 2024 | 54 | 2025-05-16 |
| CVDR432303 | Beleidsregels terugvordering Rotterdam 2017 | 11 | 2021-07-07 |
| CVDR701597 | Beleidsregels giften en kostenvoordelen Rotterdam 2023 | 4 | 2023-10-11 |

**Locatie:** `regulation/nl/gemeentelijke_verordening/rotterdam/uitvoeringsbeleid/`

### 2.6 Draagkrachtberekening Bijzondere Bijstand (Hoofdstuk 10)

**Bestand:** `regulation/nl/gemeentelijke_verordening/rotterdam/uitvoeringsbeleid/beleidsregels_bijzondere_bijstand_rotterdam_2024_2025-05-16.yaml`

Machine-uitvoerbare logica toegevoegd aan artikelen 10.1 t/m 10.4:

#### Art. 10.1 - Algemene beleidsuitgangspunten

| Output | Beschrijving |
|--------|-------------|
| `heeft_draagkracht` | False bij schuldhulptraject (lid 9) |
| `vrijstelling_individuele_inkomenstoeslag` | Inkomenstoeslag niet meegerekend (lid 2) |
| `vrijstelling_alo_kop` | ALO-kop niet meegerekend (lid 3) |
| `aftrek_eigen_bijdrage_wlz` | Eigen bijdrage WLZ aftrekbaar (lid 5) |

#### Art. 10.2 - Draagkrachtperiode

| Definitie | Waarde |
|-----------|--------|
| DRAAGKRACHTPERIODE_MAANDEN | 12 |

| Output | Beschrijving |
|--------|-------------|
| `nieuwe_periode_vastgesteld` | Geen nieuwe periode bij actieve periode (lid 4) |
| `bijzondere_bijstand_bedrag` | Kosten minus draagkracht (lid 5) |

#### Art. 10.3 - Draagkracht en inkomen (kernberekening)

**Endpoint:** `draagkracht_inkomen`

**Definities:**

| Constante | Waarde | Beschrijving |
|-----------|--------|--------------|
| DRAAGKRACHTVRIJ_PERCENTAGE | 100 | Onder 100% norm = geen draagkracht |
| DRAAGKRACHTVRIJ_PERCENTAGE_BIJLAGE2 | 110 | Bijlage 2: t/m 110% norm draagkrachtvrij |
| DRAAGKRACHT_PERCENTAGE_110_150 | 50 | 50% van inkomen tussen 110-150% norm |
| DRAAGKRACHT_PERCENTAGE_150_PLUS | 100 | 100% van inkomen boven 150% norm |
| GRENS_150_PERCENTAGE | 150 | Grens voor volledige draagkracht |

**Staffel Bijlage 2 kostensoorten:**

| Inkomen t.o.v. bijstandsnorm | Draagkracht% | Berekening |
|------------------------------|--------------|------------|
| ≤110% | 0% | Geen draagkracht |
| 110-150% | 50% | 50% × (inkomen - 110% norm) |
| >150% | 100% | 50% × (150%-110% norm) + 100% × (inkomen - 150% norm) |

**Staffel Bijlage 3 kostensoorten:**

| Inkomen t.o.v. bijstandsnorm | Draagkracht% | Berekening |
|------------------------------|--------------|------------|
| ≤100% | 0% | Geen draagkracht |
| >100% | 100% | 100% × (inkomen - norm) |

**Outputs:**
- `draagkracht_inkomen` - Berekende draagkracht uit inkomen
- `draagkrachtpercentage` - Toegepast percentage
- `inkomen_als_percentage_norm` - Inkomen als % van norm
- `heeft_draagkracht_inkomen` - Boolean

#### Art. 10.4 - Draagkracht en vermogen

**Endpoint:** `draagkracht_vermogen`

| Definitie | Waarde |
|-----------|--------|
| DRAAGKRACHT_PERCENTAGE_VERMOGEN | 100 |

**Berekening:**
1. Vermogen boven vermogensgrens (art. 34 lid 3 Pw) = 100% draagkracht
2. Vrijstelling: max 1x bijstandsnorm van spaartegoed (kosten levensonderhoud)

**Outputs:**
- `draagkracht_vermogen` - Berekende draagkracht uit vermogen
- `vrijstelling_levensonderhoud` - Vrijgesteld bedrag (max 1x norm)
- `heeft_draagkracht_vermogen` - Boolean

---

## 3. Validatie

### 3.1 Schema validatie resultaten

```
Rotterdam participatie YAML: 3/3 valid
Rotterdam uitvoeringsbeleid YAML: 3/3 valid
Participatiewet: 1/1 valid
```

Alle gewijzigde bestanden valideren succesvol tegen `schema/v0.3.0/schema.json`.

### 3.2 Bekende validatieproblemen (pre-existing)

| Bestand | Probleem | Oorzaak |
|---------|----------|---------|
| regeling_standaardpremie | `valid_from` niet conform patroon | Gebruikt `#datum_inwerkingtreding` referentie |
| vreemdelingenwet_2000 | IN operatie met array | Schema ondersteunt geen arrays in `operationValue` |
| wet_basisregistratie_personen | IN operatie met array | Schema ondersteunt geen arrays in `operationValue` |

Deze problemen zijn pre-existing en vallen buiten scope van test1_rotterdam.

---

## 4. Vergelijking test0 vs test1

| Aspect | test0_rotterdam | test1_rotterdam |
|--------|----------------|-----------------|
| Schema | PoC (MinBZK) | MVP (regelrecht.nl) |
| Inkomensvrijlating max maanden | 6 | 30 (PiB) |
| Alleenstaande ouder extra | Niet | +12.5% |
| Medisch urenbeperkt | Niet | 15% |
| Daklozenkorting | Alleen human_assessment | Machine-uitvoerbaar (20%) |
| Terugwerkende kracht | Geen | Max 3 maanden (PiB) |
| Beleidsregels bijzondere bijstand | Niet gedownload | Gedownload + machine_readable |
| Draagkrachtberekening | Niet gemodelleerd | Volledig machine-uitvoerbaar |
| Beleidsregels terugvordering | Niet gedownload | Gedownload (tekst) |
| Beleidsregels giften | Niet gedownload | Gedownload (tekst) |

---

## 5. Dekkingsgraad

### 5.1 Opgeloste gaps uit test0

| Gap | Status |
|-----|--------|
| Inkomensvrijlating bij werk niet gemodelleerd | **Opgelost** - Volledig PiB-conform |
| Normafwijkingen niet gemodelleerd | **Gedeeltelijk opgelost** - Art. 27 daklozenkorting |
| Terugwerkende kracht niet gemodelleerd | **Opgelost** - Art. 44 met 3 maanden |
| Draagkrachtpercentage bijzondere bijstand | **Opgelost** - Art. 10.1-10.4 machine_readable |
| Beleidsregels terugvordering niet opgehaald | **Opgelost** - CVDR432303 gedownload |
| Beleidsregels giften niet opgehaald | **Opgelost** - CVDR701597 gedownload |

### 5.2 Openstaande gaps

| Gap | Status | Actie |
|-----|--------|-------|
| Geindexeerd uurtarief jobcoaching 2024-2026 | Open | UWV Besluit Normbedragen ophalen |
| Compensatietabel beschut werk na 01-07-2026 | Open | Wacht op Rotterdam |
| Beleidsregels terugvordering machine_readable | Open | Beslisregels toevoegen aan CVDR432303 |
| Beleidsregels giften PiB-conformiteit | Open | Toetsing €1.200 grens aan art. 39 Pw |

---

## 6. Aanbevelingen vervolgstappen

1. **Schema uitbreiden** - Ondersteuning voor arrays in IN/NOT_IN operaties toevoegen
2. **Terugvordering machine_readable** - Beslisregels uit CVDR432303 extraheren
3. **Giften PiB-conformiteit** - Toetsen of €1.200 grensbedrag conform art. 39 Pw nieuw
4. **BDD tests uitbreiden** - Scenario's voor draagkrachtberekening
5. **Bijlage 2/3 kostensoorten** - Lijst van kostensoorten per regime extraheren

---

## 7. Bestanden

### 7.1 Gewijzigde bestanden

| Bestand | Wijziging |
|---------|----------|
| `schema/v0.2.0/schema.json` | $id naar regelrecht.nl |
| `schema/v0.3.0/schema.json` | $id naar regelrecht.nl |
| `regulation/nl/wet/participatiewet/2025-01-01.yaml` | Art. 27, 31, 44 machine_readable uitgebreid |
| `regulation/nl/gemeentelijke_verordening/rotterdam/participatie/*.yaml` | $schema naar MVP |
| 18 andere YAML bestanden | $schema naar MVP |

### 7.2 Nieuwe bestanden

| Bestand | Beschrijving |
|---------|-------------|
| `doc/experimenten/test1_rotterdam/status_machine_uitvoerbaarheid.md` | Dit document |
| `regulation/nl/gemeentelijke_verordening/rotterdam/uitvoeringsbeleid/beleidsregels_bijzondere_bijstand_rotterdam_2024_2025-05-16.yaml` | Bijzondere bijstand met draagkracht machine_readable |
| `regulation/nl/gemeentelijke_verordening/rotterdam/uitvoeringsbeleid/beleidsregel_..._terugvordering_..._2021-07-07.yaml` | Terugvordering (tekst) |
| `regulation/nl/gemeentelijke_verordening/rotterdam/uitvoeringsbeleid/beleidsregels_giften_en_kostenvoordelen_..._2023-10-11.yaml` | Giften (tekst) |
| `script/convert_cvdr.py` | CVDR XML naar YAML converter |

---

## 8. Parameters per datum

Alle bedragen in **eurocent** tenzij anders vermeld. Geldend per **01-01-2026**:

### Participatiewet (landelijk)

**Inkomensvrijlating (art. 31 Pw PiB):**
- Vrijlating percentage standaard: 25%
- Vrijlating percentage alleenstaande ouder: 37.5%
- Vrijlating percentage medisch urenbeperkt: 15%
- Max vrijlating per maand: 25300 (€253)
- Max maanden: 30

**Daklozenkorting (art. 27 Pw):**
- Verlaging percentage: 20%

**Terugwerkende kracht (art. 44 Pw PiB):**
- Max maanden voor melding: 3

### Beleidsregels Bijzondere Bijstand Rotterdam

**Draagkrachtperiode (art. 10.2):**
- Standaard periode: 12 maanden

**Draagkracht inkomen - Bijlage 2 (art. 10.3):**
- Draagkrachtvrij t/m: 110% bijstandsnorm
- Draagkracht% 110-150% norm: 50%
- Draagkracht% >150% norm: 100%

**Draagkracht inkomen - Bijlage 3 (art. 10.3):**
- Draagkrachtvrij t/m: 100% bijstandsnorm
- Draagkracht% >100% norm: 100%

**Draagkracht vermogen (art. 10.4):**
- Boven vermogensgrens: 100%
- Vrijstelling spaartegoed: 1x bijstandsnorm

### Beleidsregels Giften Rotterdam

**Grensbedragen (art. 2):**
- Standaard grensbedrag: €1.200/jaar
- Tijdelijk verhoogd (2022-2023): €2.250/jaar

---

## 9. Endpoints overzicht

| Endpoint | Artikel | Bestand | Beschrijving |
|----------|---------|---------|--------------|
| `daklozenkorting` | Art. 27 Pw | participatiewet | 20% normverlaging daklozen |
| `draagkracht_inkomen` | Art. 10.3 | bijzondere_bijstand_rotterdam | Draagkracht uit inkomen |
| `draagkracht_vermogen` | Art. 10.4 | bijzondere_bijstand_rotterdam | Draagkracht uit vermogen |

---

*Validatie uitgevoerd: 2026-02-24*
*Schema: https://regelrecht.nl/schema/v0.3.0/schema.json*
