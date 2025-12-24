from ruamel.yaml import YAML
from pathlib import Path

yaml = YAML()
yaml.preserve_quotes = True
yaml.default_flow_style = False
yaml.width = 4096

# Read the YAML file
yaml_path = Path("regulation/nl/wet/algemene_ouderdomswet/2024-01-01.yaml")
with open(yaml_path, "r", encoding="utf-8") as f:
    data = yaml.load(f)

# Find article 7.1
for i, article in enumerate(data["articles"]):
    if article.get("number") == "7.1":
        print(f"Found article 7.1 at index {i}")

        # Add machine_readable section
        machine_readable = {
            "endpoint": "aow_uitkering",
            "competent_authority": {
                "name": "Sociale Verzekeringsbank",
                "abbreviation": "SVB",
                "type": "INSTANCE",
            },
            "execution": {
                "parameters": [
                    {
                        "name": "BSN",
                        "type": "string",
                        "required": True,
                        "description": "BSN van de persoon",
                    }
                ],
                "definitions": {
                    "JAREN_VOOR_VOLLEDIG_PENSIOEN": {"value": 50},
                    "OPBOUW_PER_JAAR": {"value": 0.02},
                    "BASISBEDRAG_ALLEENSTAAND": {"value": 138000},
                    "BASISBEDRAG_GEDEELD": {"value": 95200},
                    "PARTNER_TOESLAG_MAXIMUM": {"value": 25800},
                    "INKOMENSGRENS_PARTNER": {"value": 280000},
                    "KORTINGSPERCENTAGE": {"value": 0.02},
                },
                "input": [
                    {
                        "name": "WOONACHTIGE_VERZEKERDE_JAREN",
                        "type": "number",
                        "description": "Aantal verzekerde jaren voor AOW-opbouw op basis van woonperiodes",
                        "source": {
                            "regulation": "verzekerde_tijdvakken_bron",
                            "output": "woonperiodes",
                            "parameters": {"bsn": "$BSN"},
                            "description": "Verzekerde jaren obv woonperiodes (externe bron)",
                        },
                    },
                    {
                        "name": "WERKZAME_VERZEKERDE_JAREN",
                        "type": "number",
                        "description": "Aantal verzekerde jaren voor AOW-opbouw op basis van werk en uitkeringen",
                        "source": {
                            "regulation": "wet_structuur_uitvoeringsorganisatie_werk_en_inkomen",
                            "output": "verzekerde_jaren",
                            "parameters": {"bsn": "$BSN"},
                            "description": "Verzekerde jaren obv werk (UWV)",
                        },
                    },
                    {
                        "name": "GEBOORTEDATUM",
                        "type": "date",
                        "description": "Geboortedatum van de aanvrager",
                        "source": {
                            "regulation": "wet_basisregistratie_personen",
                            "output": "geboortedatum",
                            "parameters": {"bsn": "$BSN"},
                            "description": "Geboortedatum uit BRP",
                        },
                    },
                    {
                        "name": "LEEFTIJD",
                        "type": "number",
                        "description": "Leeftijd van de aanvrager",
                        "source": {
                            "regulation": "wet_basisregistratie_personen",
                            "output": "leeftijd",
                            "parameters": {"bsn": "$BSN"},
                            "description": "Leeftijd uit BRP",
                        },
                    },
                    {
                        "name": "HEEFT_PARTNER",
                        "type": "boolean",
                        "description": "Heeft de persoon een partner volgens BRP",
                        "source": {
                            "regulation": "wet_basisregistratie_personen",
                            "output": "heeft_partner",
                            "parameters": {"bsn": "$BSN"},
                            "description": "Partnerstatus uit BRP",
                        },
                    },
                    {
                        "name": "PARTNER_BSN",
                        "type": "string",
                        "description": "BSN van de partner",
                        "source": {
                            "regulation": "wet_basisregistratie_personen",
                            "output": "partner_bsn",
                            "parameters": {"bsn": "$BSN"},
                            "description": "Partner BSN uit BRP",
                        },
                    },
                    {
                        "name": "PARTNER_GEBOORTEDATUM",
                        "type": "date",
                        "description": "Geboortedatum van de partner",
                        "source": {
                            "regulation": "wet_basisregistratie_personen",
                            "output": "partner_geboortedatum",
                            "parameters": {"bsn": "$BSN"},
                            "description": "Partner geboortedatum uit BRP",
                        },
                    },
                    {
                        "name": "PARTNER_LEEFTIJD",
                        "type": "number",
                        "description": "Leeftijd van de partner",
                        "source": {
                            "regulation": "wet_basisregistratie_personen",
                            "output": "leeftijd",
                            "parameters": {"bsn": "$PARTNER_BSN"},
                            "description": "Partner leeftijd uit BRP",
                        },
                    },
                    {
                        "name": "INKOMEN",
                        "type": "amount",
                        "description": "Toetsingsinkomen",
                        "source": {
                            "regulation": "wet_inkomstenbelasting_2001",
                            "output": "inkomen",
                            "parameters": {"bsn": "$BSN"},
                            "description": "Toetsingsinkomen uit Belastingdienst",
                        },
                    },
                    {
                        "name": "PARTNER_INKOMEN",
                        "type": "amount",
                        "description": "Toetsingsinkomen partner",
                        "source": {
                            "regulation": "wet_inkomstenbelasting_2001",
                            "output": "inkomen",
                            "parameters": {"bsn": "$PARTNER_BSN"},
                            "description": "Partner toetsingsinkomen uit Belastingdienst",
                        },
                    },
                    {
                        "name": "PENSIOENLEEFTIJD",
                        "type": "number",
                        "description": "AOW-leeftijd voor deze persoon",
                        "source": {
                            "regulation": "algemene_ouderdomswet",
                            "output": "pensioenleeftijd",
                            "parameters": {"geboortedatum": "$GEBOORTEDATUM"},
                            "description": "Pensioenleeftijd obv geboortedatum (artikel 7a)",
                        },
                    },
                    {
                        "name": "PARTNER_PENSIOENLEEFTIJD",
                        "type": "number",
                        "description": "AOW-leeftijd voor de partner",
                        "source": {
                            "regulation": "algemene_ouderdomswet",
                            "output": "pensioenleeftijd",
                            "parameters": {"geboortedatum": "$PARTNER_GEBOORTEDATUM"},
                            "description": "Partner pensioenleeftijd obv geboortedatum (artikel 7a)",
                        },
                    },
                ],
                "output": [
                    {
                        "name": "is_gerechtigd",
                        "type": "boolean",
                        "description": "Heeft de persoon recht op AOW",
                    },
                    {
                        "name": "basisbedrag",
                        "type": "amount",
                        "description": "Basis AOW-bedrag voor opbouw",
                    },
                    {
                        "name": "opbouwpercentage",
                        "type": "number",
                        "description": "Opbouwpercentage AOW",
                    },
                    {
                        "name": "totaal_verzekerde_jaren",
                        "type": "number",
                        "description": "Som van woonachtige en werkzame verzekerde jaren",
                    },
                    {
                        "name": "partner_toeslag_factor",
                        "type": "number",
                        "description": "Factor voor partnertoeslag berekening",
                    },
                    {
                        "name": "pensioenbedrag",
                        "type": "amount",
                        "description": "Uiteindelijke AOW-uitkering",
                    },
                ],
                "actions": [
                    {
                        "output": "totaal_verzekerde_jaren",
                        "value": {
                            "operation": "ADD",
                            "values": [
                                "$WOONACHTIGE_VERZEKERDE_JAREN",
                                "$WERKZAME_VERZEKERDE_JAREN",
                            ],
                        },
                    },
                    {
                        "output": "is_gerechtigd",
                        "value": {
                            "operation": "IF",
                            "when": {
                                "operation": "AND",
                                "conditions": [
                                    {
                                        "operation": "GREATER_THAN_OR_EQUAL",
                                        "subject": "$LEEFTIJD",
                                        "value": "$PENSIOENLEEFTIJD",
                                    },
                                    {
                                        "operation": "GREATER_THAN",
                                        "subject": "$totaal_verzekerde_jaren",
                                        "value": 0,
                                    },
                                ],
                            },
                            "then": True,
                            "else": False,
                        },
                    },
                    {
                        "output": "basisbedrag",
                        "value": {
                            "operation": "IF",
                            "when": {
                                "operation": "EQUALS",
                                "subject": "$HEEFT_PARTNER",
                                "value": True,
                            },
                            "then": "$BASISBEDRAG_GEDEELD",
                            "else": "$BASISBEDRAG_ALLEENSTAAND",
                        },
                    },
                    {
                        "output": "opbouwpercentage",
                        "value": {
                            "operation": "DIVIDE",
                            "values": [
                                {
                                    "operation": "MIN",
                                    "values": [
                                        "$totaal_verzekerde_jaren",
                                        "$JAREN_VOOR_VOLLEDIG_PENSIOEN",
                                    ],
                                },
                                "$JAREN_VOOR_VOLLEDIG_PENSIOEN",
                            ],
                        },
                    },
                    {
                        "output": "partner_toeslag_factor",
                        "value": {
                            "operation": "IF",
                            "when": {
                                "operation": "EQUALS",
                                "subject": "$HEEFT_PARTNER",
                                "value": True,
                            },
                            "then": {
                                "operation": "ADD",
                                "values": [
                                    1,
                                    {
                                        "operation": "IF",
                                        "when": {
                                            "operation": "AND",
                                            "conditions": [
                                                {
                                                    "operation": "LESS_THAN",
                                                    "subject": "$PARTNER_LEEFTIJD",
                                                    "value": "$PARTNER_PENSIOENLEEFTIJD",
                                                },
                                                {
                                                    "operation": "LESS_THAN",
                                                    "subject": "$PARTNER_INKOMEN",
                                                    "value": "$INKOMENSGRENS_PARTNER",
                                                },
                                            ],
                                        },
                                        "then": {
                                            "operation": "MIN",
                                            "values": [
                                                {
                                                    "operation": "DIVIDE",
                                                    "values": [
                                                        "$PARTNER_TOESLAG_MAXIMUM",
                                                        "$BASISBEDRAG_GEDEELD",
                                                    ],
                                                },
                                                {
                                                    "operation": "MULTIPLY",
                                                    "values": [
                                                        {
                                                            "operation": "SUBTRACT",
                                                            "values": [
                                                                "$INKOMENSGRENS_PARTNER",
                                                                "$PARTNER_INKOMEN",
                                                            ],
                                                        },
                                                        "$KORTINGSPERCENTAGE",
                                                    ],
                                                },
                                            ],
                                        },
                                        "else": 0,
                                    },
                                ],
                            },
                            "else": 1,
                        },
                    },
                    {
                        "output": "pensioenbedrag",
                        "value": {
                            "operation": "MULTIPLY",
                            "values": [
                                "$basisbedrag",
                                "$opbouwpercentage",
                                "$partner_toeslag_factor",
                            ],
                        },
                    },
                ],
            },
        }

        article["machine_readable"] = machine_readable
        print("Added machine_readable section to article 7.1")
        break

# Write back to file
with open(yaml_path, "w", encoding="utf-8") as f:
    yaml.dump(data, f)

print(f"Updated {yaml_path}")
