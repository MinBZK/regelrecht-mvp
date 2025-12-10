# Todo list

## Enum type voor parameters

### Context

In `regulation/nl/wet/burgerlijk_wetboek_boek_5/2024-01-01.yaml` regel 39-42:

```yaml
- name: type_beplanting
  type: string
  required: true
  description: Type beplanting - "boom" of "heg_of_heester"
```

Dit is eigenlijk een enum met twee mogelijke waarden, niet een vrije string.

### Onderzoek

Het oude schema (regelrecht-laws v0.1.6) ondersteunt dit ook niet. Beschikbare types zijn:
- string, number, boolean, amount, object, array, date

Er is geen `enum` type of `allowed_values` property.

### Voorstel

Optie A - enum als constraint op string:
```yaml
- name: type_beplanting
  type: string
  enum:
    - boom
    - heg_of_heester
  description: Type beplanting
```

Optie B - enum als apart type:
```yaml
- name: type_beplanting
  type: enum
  values:
    - boom
    - heg_of_heester
  description: Type beplanting
```

### Acties

- [ ] Kies gewenste syntax
- [ ] Update schema (indien extern beheerd)
- [ ] Implementeer validatie in engine (optioneel)
- [ ] Update bestaande YAML bestanden
