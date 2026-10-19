# -*- coding: utf-8 -*-
"""Knip de Kamerstukken in brokken toelichting en hang elk brok aan een artikel.

Waar de afspeler zegt hoe de tekst van een artikel tot stand kwam, zegt dit
script wat er in die tijd over werd gezegd: de memorie van toelichting, de
verslagen en de nota's naar aanleiding daarvan (van beide Kamers), en de
amendementen. De toelichting die een nota van wijziging bij haar onderdelen
geeft, hangt al aan de handeling zelf (afspelen.py) en wordt hier niet nog
eens gelezen.

HOE EEN BROK AAN EEN ARTIKEL HANGT, van sterk naar zwak:
- artikelsgewijs: het stuk heeft een tussenkop "Artikel 54 (...)",
  "Artikel 1.1 Begripsbepalingen" of "Artikel I"; alles tot de volgende zo'n
  kop of de volgende paragraaf gaat over dat artikel. Onder een
  wijzigingsartikel ("Artikel I") opent "Onderdeel A" een eigen brok, en dat
  hangt ook aan het artikel van de wet die dat onderdeel wijzigt;
- amendement: het adres van de opdracht ("In artikel 26, eerste lid, ...") of
  het artikel dat het amendement invoegt ("Artikel 13a. ...");
- vermelding: een alinea elders noemt "artikel 54, eerste lid". Wat erachter
  een andere regeling noemt telt niet mee: een naam ("artikel 22
  Provinciewet") of "van de/het ..." die niet deze wet is. Welke namen deze
  wet heeft, zegt dossier.yaml (deze_wet).

WAT HIER NIET WORDT GEHAAKT, en waarom. Het advies van de Raad van State
noemt artikelen in de nummering van de versie die de Afdeling zag, en die is
niet die van het voorstel; een vermelding daaruit zou op het verkeerde
artikel landen. Zijn alinea's tellen als niet gehaakt, met die reden. Een
nummer dat in geen enkele stand van de wet bestaat telt ook als niet gehaakt.

VERNUMMERING. Een stuk telt in de nummering van zijn eigen datum. Een nummer
wordt daarom gelezen in de stand van de wet op dat moment (de tijdlijn van
afspelen.py), en het brok hangt aan het artikel dat toen dat nummer droeg,
ook als het later een ander nummer kreeg. Dat artikel is zijn identiteit:
de sleutel waarmee het in de tijdlijn begon.

Voorstel M526.
"""
import json
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from handelingen import ADRES, RANG, RANGTELWOORDEN, schoon  # noqa: E402

# Stukken die zelf de tekst wijzigen, of de tekst zelf zijn: geen toelichting.
GEEN_TOELICHTING = ("Nota van wijziging", "Amendement", "Voorstel van wet", "Gewijzigd voorstel van wet",
                    "Motie", "Koninklijke boodschap")
# De nummering van de versie die de Afdeling advisering zag, is niet die van
# het voorstel; vermeldingen daaruit worden niet gehaakt.
EIGEN_NUMMERING = ("Advies Afdeling advisering Raad van State",)

# De soort brok, naar de soort van het stuk (zoals de Kamer haar noemt); de
# pagina kleurt erop. De eerste die past, telt.
SOORTEN = (
    ("Memorie van toelichting", "memorie"),
    ("Advies Afdeling advisering", "advies"),
    ("Voorlichting Afdeling advisering", "advies"),
    ("Nota n.a.v.", "antwoord"),
    ("Nota naar aanleiding", "antwoord"),
    ("Memorie van antwoord", "antwoord"),
    ("Verslag", "vraag"),
    ("Voorlopig verslag", "vraag"),
    ("Tweede verslag", "vraag"),
    ("Nader verslag", "vraag"),
    ("Amendement", "amendement"),
    ("Brief", "brief"),
)

NR = r"(?:[A-Z][a-z]? )?\d+[a-z]*(?:[.:]\d+[a-z]*)*|[IVXL]+(?![\w])"
ARTKOP = re.compile(rf"^Artikel[\s ]+(?P<nr>{NR})\.?(?:[\s ]*\((?P<titel>[^)]*)\)?|[\s ]+(?P<rest>.*)|$)$")
ONDKOP = re.compile(r"^Onderde(?:el|len)[\s ]+(?P<letters>[A-Z]{1,7}\b.*)$")
TOKEN = r"\d+[a-z]?(?:[.:]\d+[a-z]?)*"
VERMELDING = re.compile(
    rf"\b[Aa]rtikel(?:en)?[\s ]+(?P<lijst>{TOKEN}(?:(?:,[\s ]+|[\s ]+(?:en|of|tot en met|t/m)[\s ]+){TOKEN})*)"
    rf"(?P<lid>,?[\s ]+(?:{RANG})[\s ]+lid)?"
)
NUMMER = re.compile(TOKEN)
VOOR_DE_NAAM = re.compile(
    r"^[\s ,]*(?:(?:eerste|tweede|derde|vierde|vijfde|zesde|zevende|achtste|negende|tiende) lid,?\s*)?"
    r"(?:onder(?:deel)? [a-z0-9]+,?\s*)?")
DEZE = ("wetsvoorstel", "voorstel", "voorstel van wet", "wet", "deze wet", "dit wetsvoorstel")


def soort_van(stuksoort):
    return next((s for k, s in SOORTEN if stuksoort.startswith(k)), "brief")


def tekst(e):
    """De tekst van een element zonder voetnoten."""
    delen = []

    def loop(x):
        if x.tag == "noot":
            if x.tail:
                delen.append(x.tail)
            return
        if x.text:
            delen.append(x.text)
        for c in x:
            loop(c)
        if x is not e and x.tail:
            delen.append(x.tail)

    loop(e)
    return schoon("".join(delen))


def is_naam(t, naam):
    """Begint t met deze naam, en niet met een langere ('Archiefwet 1995' is
    een andere wet dan 'Archiefwet'; 'wetgever' is geen 'wet')?"""
    if not t.startswith(naam):
        return False
    return not re.match(r"\w|\s\d{4}\b|\s20\.\.", t[len(naam):len(naam) + 6])


def andere_wet(staart, deze_wet):
    """Wijst wat na een vermelding staat een andere regeling aan?"""
    rest = staart[VOOR_DE_NAAM.match(staart).end():]
    m = re.match(r"van (?:de|het|boek|Boek|titel|Titel) ", rest)
    if m:
        return not any(is_naam(rest[m.end():], n) for n in DEZE + tuple(deze_wet))
    if re.match(r"[A-Z]", rest):
        # Een hoofdletter direct na het nummer is de naam van een regeling
        # ("artikel 22 Provinciewet", "artikel 17 EVRM"); alleen deze wet telt.
        return not any(is_naam(rest, n) for n in deze_wet)
    return False


def vermeldingen(t, deze_wet=("Wpp", "Wet op de politieke partijen")):
    """De artikelen van deze wet die een alinea noemt, met het lid als dat erbij staat."""
    uit = []
    for m in VERMELDING.finditer(t):
        if andere_wet(t[m.end():m.end() + 80], deze_wet):
            continue
        nrs = NUMMER.findall(m["lijst"])
        reeks = re.search(r"(\d+)[\s ]+(?:tot en met|t/m)[\s ]+(\d+)$", m["lijst"])
        if reeks and int(reeks[2]) - int(reeks[1]) < 30:
            nrs = [n for n in nrs if n not in reeks.groups()] + [str(i) for i in range(int(reeks[1]), int(reeks[2]) + 1)]
        lid = None
        if m["lid"] and len(nrs) == 1:
            lid = RANGTELWOORDEN[m["lid"].replace(",", "").split()[0]]
        uit += [(n, lid if n == nrs[-1] else None) for n in nrs]
    return uit


# ---------------------------------------------------------------- de tijd

class Standen:
    """Uit de tijdlijn van afspelen.py: welk artikel op een datum welk nummer
    droeg, en de identiteit van elk artikel door de tijd."""

    def __init__(self, tijdlijn):
        self.t = tijdlijn
        self.ident = tijdlijn.get("identiteit") or {}
        # Per moment: het aantal handelingen dat ervoor heeft gespeeld.
        self.grenzen, gespeeld = [], 0
        for m in tijdlijn.get("momenten") or []:
            self.grenzen.append(((m.get("datum") or ""), gespeeld, m.get("stuk")))
            if m.get("stappen"):
                gespeeld = m["stappen"][1]
        self._cache = {}
        self.alle = {}
        b = dict(tijdlijn["begin"]["blokken"])
        for k, x in b.items():
            if x["soort"] == "artikel":
                self.alle.setdefault(x["nr"], self.ident.get(k, k))
        for s in tijdlijn["stappen"]:
            for k, x in s["wijzig"].items():
                if x["soort"] == "artikel":
                    self.alle.setdefault(x["nr"], self.ident.get(k, k))

    def stand(self, i):
        if i not in self._cache:
            b = dict(self.t["begin"]["blokken"])
            for s in self.t["stappen"][:i]:
                b.update(s["wijzig"])
                for w in s["weg"]:
                    b.pop(w, None)
            self._cache[i] = {x["nr"]: self.ident.get(k, k) for k, x in b.items() if x["soort"] == "artikel"}
        return self._cache[i]

    def index(self, stuk, datum):
        """Het aantal gespeelde handelingen op het moment van dit stuk."""
        for d, gespeeld, s in self.grenzen:
            if s == stuk:
                return gespeeld
        n = 0
        for d, gespeeld, s in self.grenzen:
            if d and datum and d <= datum:
                n = gespeeld
        return n

    def artikel(self, nr, stuk, datum):
        """De identiteit van het artikel dat op de datum van het stuk dit
        nummer droeg; anders van het eerste dat het ooit droeg."""
        i = self.index(stuk, datum)
        return self.stand(i).get(nr) or self.alle.get(nr)


# ---------------------------------------------------------------- het lezen

class Lezer:
    """Loopt één stuk door en levert de brokken."""

    def __init__(self, stuk, datum, stuksoort, bestaat, deze_wet, doel=()):
        self.stuk, self.datum, self.soort = stuk, datum, soort_van(stuksoort)
        self.eigen_nummering = stuksoort.startswith(EIGEN_NUMMERING)
        self.bestaat, self.deze_wet = bestaat, deze_wet
        # Bij een wijzigingswet: de artikelen van de wetten die zij wijzigt.
        # Een vermelding van 'artikel 17a' gaat daarover, niet over het voorstel.
        self.doel = set(doel)
        self.pad = []          # de koppen boven de plek, als (diepte, tekst)
        self.sub = None        # de laatste tussenkop buiten een artikelsgewijs brok
        self.brok = None       # het artikelsgewijze brok dat nu loopt
        self.artikel = None    # het artikel van dat brok, voor 'Onderdeel A'
        self.brokken, self.losse, self.niet = [], {}, []

    def kop(self):
        delen = [t for _, t in self.pad] + ([self.sub] if self.sub else [])
        return " › ".join(delen)

    def sluit(self):
        if self.brok and len(self.brok["tekst"]) > 0:
            self.brokken.append(self.brok)
        self.brok = None

    def open(self, nr, titel, onderdelen=None):
        self.sluit()
        self.brok = {"stuk": self.stuk, "datum": self.datum, "soort": self.soort,
                     "rol": "artikelsgewijs", "kop": self.kop(), "titel": titel,
                     "artikelen": [nr], "leden": {}, "tekst": []}
        if onderdelen:
            self.brok["onderdelen"] = onderdelen

    def alinea(self, t, soort="al"):
        if not t:
            return
        if self.brok:
            self.brok["tekst"].append([t, soort])
            for nr, lid in vermeldingen(t, self.deze_wet):
                if nr == self.brok["artikelen"][0] and lid:
                    self.brok["leden"].setdefault(nr, set()).add(lid)
            return
        gevonden = vermeldingen(t, self.deze_wet)
        if not gevonden:
            return
        if self.eigen_nummering:
            self.niet.append({"stuk": self.stuk, "reden": "nummering van de adviesversie", "tekst": t[:160]})
            return
        for nr, lid in gevonden:
            if nr not in self.bestaat and nr in self.doel:
                sleutel = (self.kop(), "doel", nr)
                b = self.losse.get(sleutel)
                if not b:
                    b = self.losse[sleutel] = {"stuk": self.stuk, "datum": self.datum, "soort": self.soort,
                                               "rol": "vermelding", "kop": self.kop(), "artikelen": [],
                                               "doelartikelen": [nr], "leden": {}, "tekst": []}
                if not b["tekst"] or b["tekst"][-1][0] != t:
                    b["tekst"].append([t, "al"])
                continue
            if nr not in self.bestaat:
                self.niet.append({"stuk": self.stuk, "reden": f"artikel {nr} bestaat in geen stand", "tekst": t[:160]})
                continue
            sleutel = (self.kop(), nr)
            b = self.losse.get(sleutel)
            if not b:
                b = self.losse[sleutel] = {"stuk": self.stuk, "datum": self.datum, "soort": self.soort,
                                           "rol": "vermelding", "kop": self.kop(), "artikelen": [nr],
                                           "leden": {}, "tekst": []}
            if not b["tekst"] or b["tekst"][-1][0] != t:
                b["tekst"].append([t, "al"])
            if lid:
                b["leden"].setdefault(nr, set()).add(lid)

    def loop(self, e, diepte=0):
        for c in e:
            if c.tag in ("table", "noot", "metadata", "kamerstukkop", "ondertekening", "meta"):
                continue
            if c.tag == "divisie":
                self.loop(c, diepte + 1)
                continue
            if c.tag == "kop":
                titel = tekst(c.find("titel")) if c.find("titel") is not None else tekst(c)
                nr = c.find("nr")
                t = (f"{tekst(nr)} {titel}" if nr is not None and tekst(nr) else titel).strip()
                m = ARTKOP.match(t)
                if m and m["nr"] in self.bestaat:
                    # Een artikelkop als paragraafkop (Archiefwet 2026: 'Artikel 1.1
                    # Begripsbepalingen' in een divisie).
                    self.artikel = m["nr"]
                    self.open(m["nr"], t)
                    continue
                self.sluit()
                self.artikel = None
                self.pad = [(d, x) for d, x in self.pad if d < diepte] + ([(diepte, t)] if t else [])
                self.sub = None
                continue
            if c.tag == "tussenkop":
                t = tekst(c)
                m = ARTKOP.match(t)
                if m and m["nr"] in self.bestaat:
                    self.artikel = m["nr"]
                    self.open(m["nr"], t)
                    continue
                o = ONDKOP.match(t)
                if o and self.artikel and re.fullmatch(r"[IVXL]+", self.artikel):
                    # 'Artikel I' › 'Onderdeel A': een eigen brok per onderdeel
                    # van het wijzigingsartikel.
                    letters = re.findall(r"\b[A-Z]{1,7}\b", o["letters"])
                    self.open(self.artikel, f"Artikel {self.artikel}, {t[0].lower() + t[1:]}", letters)
                    continue
                if self.brok:
                    self.brok["tekst"].append([t, "kop"])
                else:
                    self.sub = t
                continue
            if c.tag == "al":
                self.alinea(tekst(c))
                continue
            if c.tag == "li":
                nr = c.find("li.nr")
                inhoud = " ".join(tekst(x) for x in c if x.tag != "li.nr")
                self.alinea((f"{tekst(nr)} " if nr is not None else "") + inhoud, "li")
                continue
            self.loop(c, diepte)

    def lees(self, pad):
        self.loop(ET.parse(pad).getroot())
        self.sluit()
        return self.brokken + list(self.losse.values())


def amendementen(handelingen, lijst, bestaat):
    """Een amendement als één brok: wat het wil, aan de artikelen die het raakt."""
    uit = []
    for s in handelingen:
        if not s["soort"].startswith("Amendement"):
            continue
        n = s["stuk"].rsplit("-", 1)[1]
        meta = (lijst.get(n) or [{}])[0]
        arts, opdrachten, toel = [], [], []
        for o in s["onderdelen"]:
            for h in o["handelingen"]:
                opdrachten.append(h["opdracht"])
                a = h.get("adres") or {}
                for d in a.get("doelen") or [a]:
                    # "I 7" is een artikel van de Kieswet, niet van deze wet.
                    if d.get("artikel") and d["artikel"] in bestaat:
                        arts.append(d["artikel"])
                for m in re.finditer(rf"Artikel ({NR})\.? [A-Z]", " ".join(h.get("tekst") or [])):
                    arts.append(m[1])
            for t in o.get("toelichtingstekst", []):
                for p in t["tekst"]:
                    if p not in toel:
                        toel.append(p)
        arts = list(dict.fromkeys(arts))
        uit.append({"stuk": s["stuk"], "datum": (meta.get("datum") or "")[:10] or None, "soort": "amendement",
                    "rol": "amendement", "kop": meta.get("onderwerp") or s["soort"],
                    "artikelen": [a for a in arts if a in bestaat], "leden": {},
                    "nieuw": [a for a in arts if a not in bestaat],
                    "opdracht": opdrachten, "tekst": [[p, "al"] for p in toel]})
    return uit


def doelen_van(tijdlijn, wetsartikel, letters):
    """De artikelen van de doelwet die de onderdelen van een wijzigingsartikel
    noemen ('B. Artikel 2a wordt als volgt gewijzigd'), uit de begintekst."""
    uit = []
    for b in tijdlijn["begin"]["blokken"].values():
        if b["soort"] != "artikel" or b["nr"] != wetsartikel:
            continue
        for r in b["regels"]:
            m = re.match(r"^([A-Z]{1,7}) (.*)$", r[0])
            if m and m[1] in letters:
                a = ADRES.search(m[2])
                if a:
                    uit.append(a["artikel"])
    return list(dict.fromkeys(uit))


def main(map_, handelingen, tijdlijn, deze_wet=None):
    """Alle brokken van het dossier in `map_`, gehaakt aan de artikelen van
    `tijdlijn` (afspelen.py)."""
    map_ = Path(map_)
    lijst = json.loads((map_ / "stukken.json").read_text(encoding="utf-8"))
    dossier = tijdlijn["dossier"]
    standen = Standen(tijdlijn)
    bestaat = set(standen.alle)
    if deze_wet is None:
        deze_wet = tuple(tijdlijn.get("deze_wet") or ())
    voorstel = tijdlijn.get("voorstel", f"kst-{dossier}-2")
    doel = {nr for d in tijdlijn.get("doorwerking") or [] for nr in d.get("nummers") or []}
    brokken, niet, gelezen = [], [], []
    for n, xs in lijst.items():
        x = xs[0]
        pad = map_ / f"kst-{dossier}-{n}.xml"
        stuk = f"kst-{dossier}-{n}"
        if stuk == voorstel or (x["soort"] or "").startswith(GEEN_TOELICHTING) or not pad.exists():
            continue
        lezer = Lezer(stuk, (x["datum"] or "")[:10] or None, x["soort"] or "", bestaat, deze_wet, doel)
        brokken += lezer.lees(pad)
        niet += lezer.niet
        gelezen.append(n)
    brokken += amendementen(handelingen, lijst, bestaat)
    volgorde = {f"kst-{dossier}-{n}": i for i, n in enumerate(lijst)}
    brokken.sort(key=lambda b: (b["datum"] or "", volgorde.get(b["stuk"], 0), b["rol"] != "artikelsgewijs"))
    for i, b in enumerate(brokken):
        b["id"] = i
        b["leden"] = {k: sorted(v) for k, v in b["leden"].items()}
        # Het artikel dat het nummer op de datum van het stuk droeg.
        b["ids"] = [standen.artikel(nr, b["stuk"], b["datum"]) for nr in b["artikelen"]]
        if b.get("onderdelen"):
            b["doelartikelen"] = doelen_van(tijdlijn, b["artikelen"][0], b["onderdelen"])
        b.setdefault("doelartikelen", [])
    tel = {
        "brokken": len(brokken),
        "artikelsgewijs": sum(b["rol"] == "artikelsgewijs" for b in brokken),
        "vermelding": sum(b["rol"] == "vermelding" for b in brokken),
        "amendement": sum(b["rol"] == "amendement" for b in brokken),
        "artikelen_met_toelichting": len({a for b in brokken if b["rol"] == "artikelsgewijs" for a in b["artikelen"]}),
        "aan_de_doelwet": sum(bool(b["doelartikelen"]) for b in brokken),
        "niet_gehaakt": len(niet),
        "gelezen": gelezen,
    }
    return {"brokken": brokken, "niet": niet, "tel": tel}


if __name__ == "__main__":
    m, h = sys.argv[1], json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
    t = json.loads(Path(sys.argv[3]).read_text(encoding="utf-8"))
    print(json.dumps(main(m, h, t)["tel"], ensure_ascii=False))
