# Regulation Conversion Status

Dit document houdt de conversiestatus bij van wetten naar het v0.3.0 schema.

## Legenda

| Status | Betekenis |
|--------|-----------|
| :white_check_mark: | Compleet en gevalideerd |
| :warning: | Gedeeltelijk / met kanttekeningen |
| :x: | Niet gelukt / ontbreekt |
| :construction: | In progress |

---

## Wetten met machine_readable logica

### wet_op_de_zorgtoeslag
- **Status:** :white_check_mark: Compleet
- **Bron:** Bestaande conversie + oude YAML (TOESLAGEN-2025-01-01.yaml)
- **Machine_readable:** Ja, uitgebreide logica voor zorgtoeslag berekening
- **Referenties naar andere wetten:**
  - wet_minimumloon_en_minimumvakantiebijslag (voor drempelinkomen)
  - wet_basisregistratie_personen (leeftijd, partner)
  - zorgverzekeringswet (verzekeringsstatus)
  - wet_inkomstenbelasting_2001 (toetsingsinkomen, vermogen)
  - regeling_standaardpremie (standaardpremie)

### wet_basisregistratie_personen
- **Status:** :white_check_mark: Compleet
- **Bron:** Oude YAML (RvIG-2020-01-01.yaml)
- **Machine_readable:**
  - Artikel 2.7: `leeftijd` via SUBTRACT_DATE
  - Artikel 2.8: `heeft_partner` via IN operation
- **Referenties:** Geen (base data source)
- **Opmerkingen:** Base data source - data komt uit BRP database, niet uit andere wetten

### zorgverzekeringswet
- **Status:** :white_check_mark: Compleet
- **Bron:** Oude YAML (RVZ-2024-01-01.yaml)
- **Machine_readable:**
  - Artikel 2: `heeft_verzekering` via AND/NOT_NULL/IN
  - Artikel 69: `heeft_verdragsverzekering` via AND/NOT_NULL/IN/EQUALS
- **Referenties:** Geen (base data source)
- **Opmerkingen:** Base data source - data komt uit verzekeringsregistratie

### wet_inkomstenbelasting_2001
- **Status:** :white_check_mark: Compleet
- **Bron:** Oude YAML (UWV-2020-01-01.yaml)
- **Machine_readable:**
  - Artikel 2.18: `inkomen` en `partner_inkomen` via ADD
- **Referenties:** Geen (base data source)
- **Opmerkingen:**
  - Base data source - data komt uit Belastingdienst
  - Oude YAML had ook BUITENLANDS_INKOMEN in de ADD, dit is overgenomen

### wet_minimumloon_en_minimumvakantiebijslag
- **Status:** :white_check_mark: Compleet
- **Bron:** Geen oude YAML - zelf gemaakt op basis van wetstekst
- **BWB-ID:** BWBR0002638
- **Machine_readable:**
  - Artikel 8.1.b: `minimumloon_per_maand` = €2.191,80 (hardcoded waarde per 1 jan 2025)
- **Referenties:** Geen (base data source)
- **Opmerkingen:**
  - Geen oude YAML beschikbaar, dus zelf gemaakt
  - Waarde €2.191,80 komt uit artikel 8.1.b tekst
  - Dit bedrag wordt periodiek aangepast via ministeriële regeling (art. 14)
  - Voor toekomstige versies moet de waarde handmatig worden bijgewerkt

---

## Ministeriele regelingen

### regeling_standaardpremie
- **Status:** :warning: Aanwezig maar met validatie-issue
- **Validatie-fout:** `valid_from` gebruikt `#datum_inwerkingtreding` referentie ipv datum
- **Referenties:** Geen

---

## Wetten zonder machine_readable (alleen tekst)

Deze wetten zijn geharvest maar hebben (nog) geen machine_readable logica:

- algemene_wet_inkomensafhankelijke_regelingen
- burgerlijk_wetboek_boek_5
- kieswet
- participatiewet
- wet_langdurige_zorg

---

## Openstaande issues

1. ~~**wet_minimumloon_en_minimumvakantiebijslag ontbreekt**~~ - Opgelost
2. **regeling_standaardpremie valid_from** - Gebruikt interne referentie ipv datum

---

## Consistentie-opmerkingen

Alle machine_readable actions gebruiken nu de consistente structuur:
```yaml
actions:
  - output: result
    value:
      operation: OPERATION_NAME
      values/conditions/subject: ...
```

Dit is consistent met hoe comparison/logical operations werken en voorkomt verwarring over wat `values` betekent.
