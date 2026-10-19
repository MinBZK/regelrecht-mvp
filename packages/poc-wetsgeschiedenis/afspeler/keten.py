# -*- coding: utf-8 -*-
"""Een wet door de tijd: elke versie, en elke overgang getoetst.

kamerdossier.py --keten legt in bron/regelingen/<BWB>/ elke versie van een
wet vast zoals wetten.overheid.nl haar kent, en per overgang het Staatsblad
van de wet die haar maakte. Van elke overgang staan dus beide uiteinden
officieel vast. Dit script doet per overgang twee dingen:

- het neemt de tekst van de nieuwe versie over (die is de waarheid), en houdt
  per regel bij sinds welke overgang zij zo luidt;
- het speelt de wijzigingsartikelen uit het Staatsblad af op de oude versie
  (afspelen.py, dezelfde motor als voor de nota's) en legt de uitkomst naast
  de nieuwe versie. Komt het uit, dan sluit de keten op die schakel; zo niet,
  dan zegt het welke artikelen afwijken: een opdracht die deze grammatica niet
  leest, een wijziging die niet uit een wet komt (een besluit, een
  rectificatie), of een fout in een van beide.

De uitkomst heeft de vorm van een tijdlijn van afspelen.py, zodat de
wetsgeschiedenis haar toont zoals een Kamerdossier: elke overgang een moment,
de wet op elk moment, per artikel zijn levensloop, en in Doorwerking per
overgang de vergelijking met het Staatsblad.

    python3 generatoren/keten.py BWBR0007376

Voorstel M526.
"""
import copy
import json
import os
import re
import sys
from pathlib import Path

HIER = Path(__file__).resolve().parent
sys.path.insert(0, str(HIER))
import afspelen as A  # noqa: E402
from handelingen import splits_onderdeel  # noqa: E402

REGELINGEN = Path(os.environ.get("WETSGESCHIEDENIS_BRON") or HIER.parent / "bron") / "regelingen"
STATUS_GRENS = 0.5  # boven dit deel van de geraakte artikelen gelijk: 'met aanname', anders 'niet'

SOORT = {"Nieuwe-regeling": "Nieuwe wet", "Wijziging": "Wijzigingswet", "Intrekking-regeling": "Intrekking",
         "Aanhangig": "Aanhangig voorstel"}


def regels(k, diepte=0):
    """De regels van een knoop: [tekst, diepte, soort], zoals Opname ze maakt."""
    if k["soort"] == "begrip":
        t = f"{k['nr']}: {k['tekst']}".strip()
    elif k["soort"] in ("lid", "onderdeel"):
        t = f"{k['nr']} {k['tekst']}".strip()
    else:
        t = k["tekst"]
    uit = [[t, diepte, k["soort"]]] if t else []
    for c in k["kinderen"]:
        uit += regels(c, diepte + (k["soort"] in ("lid", "onderdeel")))
    return uit


def blokken(wet):
    """{sleutel: (soort, nr, [[tekst, diepte, soort]])} en de volgorde."""
    uit, volgorde = {}, []
    for k in wet:
        s = f"{k['soort']}:{k['nr']}"
        if k["soort"] == "artikel":
            r = [[f"Artikel {k['nr']}. {k['titel']}".strip(), 0, "kop"]] + [x for c in k["kinderen"] for x in regels(c)]
        else:
            r = [[f"{k['soort'].capitalize()} {k['nr']}. {k['titel']}".strip(), 0, k["soort"]]]
        uit[s] = (k["soort"], k["nr"], r)
        volgorde.append(s)
    return uit, volgorde


def speel_overgang(basis, staatsbladen, titels, stuk):
    """De wijzigingsartikelen van de Staatsbladen, na elkaar op de oude versie."""
    wet = copy.deepcopy(basis)
    handelingen, tel = [], {"afgespeeld": 0, "aanname": 0, "niet": 0}
    for pad in staatsbladen:
        artnr, onderdelen = A.wijzigingsonderdelen(pad, titels)
        for o in onderdelen:
            for h in splits_onderdeel(o["paragrafen"]):
                A.AANNAMES.clear()
                try:
                    if h["soort"] is None:
                        raise A.Fout("de opdracht is niet te lezen")
                    if h["soort"] == "samengesteld":
                        raise A.Fout("een samengestelde opdracht zonder te lezen onderdelen")
                    A.speel(wet, h, {"stuk": stuk, "onderdeel": o["letter"], "artikel": artnr})
                    status, reden = ("aanname" if A.AANNAMES else "ok"), "; ".join(A.AANNAMES[:3])
                except A.Fout as e:
                    status, reden = "niet", str(e)
                tel[{"ok": "afgespeeld"}.get(status, status)] += 1
                a = h["adres"] or {}
                handelingen.append({"onderdeel": o["letter"], "sub": h.get("subnummer"), "opdracht": h["opdracht"],
                                    "status": status, "reden": reden, "stb": Path(pad).stem,
                                    "artikelen": sorted({d["artikel"] for d in a.get("doelen", [])} or {a.get("artikel")} - {None})})
    A.ontmerk(wet)
    return wet, handelingen, tel


def tekst_per_artikel(wet):
    # Een lidnummer telt zonder punt: het Staatsblad schrijft '1.', het BWB '1'.
    # Een spatie na een komma ook niet: het BWB schrijft in artikel 2b '5,6,'.
    return {k["nr"]: [re.sub(r"\s*,\s*", ", ", re.sub(r"^(\d+[a-z]?)\. ", r"\1 ", r)) for r in A.artikeltekst(k)]
            for k in wet if k["soort"] == "artikel"}


def bouw(bwb):
    map_ = REGELINGEN / bwb
    k = json.loads((map_ / "keten.json").read_text(encoding="utf-8"))
    titel = k["titel"]
    titels = [titel] + ([k["afkorting"]] if k.get("afkorting") else [])
    versies = {v["datum"]: map_ / v["pad"] for v in k["versies"] if v["pad"]}
    data = sorted(versies)
    begin_d = data[0]
    stuk_van = lambda d: f"versie-{d}"  # noqa: E731

    refs, refnr = [], {}

    def ref(stuk):
        if stuk not in refnr:
            refnr[stuk] = len(refs)
            refs.append({"stuk": stuk, "onderdeel": None, "sub": None})
        return refnr[stuk]

    def met_herkomst(bl, vorig, stuk):
        """Regels met hun herkomst: een regel die er al zo stond, houdt de
        zijne; een nieuwe komt van deze overgang."""
        soort, nr, rr = bl
        oud = {}
        for r in (vorig or {}).get("regels", []):
            oud.setdefault(r[0], r[1])
        return {"soort": soort, "nr": nr,
                "regels": [[t, oud.get(t, [ref(stuk)]), d, s] for t, d, s in rr]}

    lees = lambda d: A.lees_wettekst(versies[d], stuk_van(d))  # noqa: E731
    wet0 = lees(begin_d)
    b0, v0 = blokken(wet0)
    huidig = {s: met_herkomst(b, None, stuk_van(begin_d)) for s, b in b0.items()}
    begin = {"volgorde": v0, "blokken": dict(huidig)}
    volgorde = v0

    stukken = [{"ondernummer": "–", "volgorde": 0, "stuk": stuk_van(begin_d), "soort": "Versie van de wet",
                "titel": f"{titel}, zoals de wet gold op {begin_d} (de oudste versie op wetten.overheid.nl)",
                "datum": begin_d, "kamer": "", "speelt_mee": True, "uitslag": None, "niet_mee": None}]
    momenten = [{"soort": "stuk", "stuk": stuk_van(begin_d), "datum": begin_d}]
    stappen, doorwerking, tel = [], [], {"afgespeeld": 0, "aanname": 0, "niet": 0}
    vorige_wet = wet0
    for o in k["overgangen"]:
        stuk = stuk_van(o["tot"])
        soort = SOORT.get(o["betreft"][0], o["betreft"][0]) if o["betreft"] else "Wijziging"
        st = {"ondernummer": ", ".join(o["dossiers"]) or "–", "volgorde": len(stukken), "stuk": stuk, "soort": soort,
              "titel": f"Wet van {o['ondertekening'] or '?'}, {', '.join(s.replace('stb-', 'Stb. ').replace('-', ' ', 1).replace('-', ', ') for s in o['staatsbladen']) or 'zonder Staatsblad'}; in werking {o['tot']}",
              "datum": o["tot"], "kamer": "", "speelt_mee": False, "uitslag": None, "niet_mee": None}
        stukken.append(st)
        if o["tot"] not in versies or o["tot"] == begin_d:
            st["niet_mee"] = ("de oudste versie op wetten.overheid.nl" if o["tot"] == begin_d else
                              "geen tekst van deze versie op wetten.overheid.nl")
            if o["tot"] != begin_d and o["tot"] < begin_d:
                momenten.insert(0, {"soort": "stuk", "stuk": stuk, "datum": o["tot"]})
            continue
        wet = lees(o["tot"])
        nb, nv = blokken(wet)
        nieuw = {s: met_herkomst(b, huidig.get(s), stuk) for s, b in nb.items()}
        wijzig = {s: b for s, b in nieuw.items() if huidig.get(s, {}).get("regels") is None
                  or [r[0] for r in huidig[s]["regels"]] != [r[0] for r in b["regels"]]}
        weg = [s for s in huidig if s not in nieuw]
        # De toets: speelt het Staatsblad deze overgang af?
        pads = [map_ / "stb" / f"{s}.xml" for s in o["staatsbladen"] if (map_ / "stb" / f"{s}.xml").exists()]
        gespeeld, handelingen, t = speel_overgang(vorige_wet, pads, titels, stuk)
        echt, gekregen, oud = tekst_per_artikel(wet), tekst_per_artikel(gespeeld), tekst_per_artikel(vorige_wet)
        geraakt = sorted(nr for nr in set(echt) | set(oud) if A.norm(" ".join(echt.get(nr, []))) != A.norm(" ".join(oud.get(nr, []))))
        anders = sorted(nr for nr in set(echt) | set(gekregen)
                        if A.norm(" ".join(echt.get(nr, []))) != A.norm(" ".join(gekregen.get(nr, []))))
        if not geraakt:
            status, reden = "ok", "de tekst van de artikelen veranderde niet (een kop, een opschrift of alleen de datum)"
        elif not handelingen:
            status, reden = "niet", ("geen wijzigingsartikel voor deze wet gevonden in " + ", ".join(o["staatsbladen"])
                                     if o["staatsbladen"] else "geen Staatsblad bij deze versie")
        elif not anders:
            status, reden = "ok", ""
        else:
            goed = len([nr for nr in geraakt if nr not in anders])
            status = "aanname" if goed / len(geraakt) > STATUS_GRENS else "niet"
            reden = f"{len(anders)} artikel(en) wijken af van de nieuwe versie: {', '.join(anders[:12])}"
        tel[{"ok": "afgespeeld"}.get(status, status)] += 1
        st["speelt_mee"] = True
        stappen.append({"stuk": stuk, "onderdeel": ", ".join(o["dossiers"]) or "–", "sub": None, "soort": "versie",
                        "opdracht": f"{soort} {', '.join(o['dossiers']) or ''}: {len(geraakt)} artikel(en) gewijzigd"
                                    f" ({', '.join(geraakt[:20])}{' …' if len(geraakt) > 20 else ''}); "
                                    f"het Staatsblad speelt {t['afgespeeld']} van {sum(t.values())} opdrachten af",
                        "tekst": [], "status": status, "reden": reden, "ronde": 1, "veeg": False, "toelichting": [],
                        "wijzig": wijzig, "weg": weg, **({"volgorde": nv} if nv != volgorde else {})})
        momenten.append({"soort": "stuk", "stuk": stuk, "datum": o["tot"], "stappen": [len(stappen), len(stappen)]})
        doorwerking.append({
            "titel": f"{o['tot']}: {', '.join(o['dossiers']) or 'zonder dossier'}", "bwb": bwb,
            "basis": f"de versie van {o['van']}", "nummers": geraakt,
            "versies": [{"versie": ", ".join(o["staatsbladen"]) or "geen Staatsblad", "stuk": stuk, "datum": o["tot"],
                         "artikel": "", "tel": t, "handelingen": handelingen,
                         "artikelen": {nr: {"voor": [A.norm(r) for r in oud.get(nr, [])],
                                            "na": [A.norm(r) for r in gekregen.get(nr, [])]}
                                       for nr in sorted(set(geraakt) | set(anders))},
                         "ijk": {"naam": f"de versie van {o['tot']}", "gelijk": len(set(echt) | set(gekregen)) - len(anders),
                                 "verschillend": anders,
                                 "artikelen": {nr: [A.norm(r) for r in echt.get(nr, [])] for nr in anders}}}]})
        huidig, volgorde, vorige_wet = nieuw, nv, wet
    for v in k.get("aanhangig") or []:
        for d in v["dossiers"]:
            stukken.append({"ondernummer": d, "volgorde": len(stukken), "stuk": f"aanhangig-{d}", "soort": "Aanhangig voorstel",
                            "titel": f"Kamerdossier {d}, aanhangig; raakt artikel {', '.join(v['artikelen'])}",
                            "datum": None, "kamer": "", "speelt_mee": False, "uitslag": None,
                            "niet_mee": "nog niet aangenomen; zie het dossier zelf"})
    for v in k.get("ingetrokken") or []:
        stuk = f"intrekking-{v['staatsblad']}"
        stukken.append({"ondernummer": ", ".join(v["dossiers"]), "volgorde": len(stukken), "stuk": stuk, "soort": "Intrekking",
                        "titel": f"De wet wordt ingetrokken: {v['staatsblad']}, ondertekend {v['ondertekening']}",
                        "datum": v["ondertekening"], "kamer": "", "speelt_mee": False, "uitslag": None, "niet_mee": None})
        if v["ondertekening"]:
            momenten.append({"soort": "stuk", "stuk": stuk, "datum": v["ondertekening"]})
    momenten.sort(key=lambda m: m["datum"] or "9999")

    # Het ijkpunt is de laatste versie zoals zij is, zonder de normalisering van de toets.
    eind = {k["nr"]: A.artikeltekst(k) for k in vorige_wet if k["soort"] == "artikel"}
    return {
        "dossier": bwb, "naam": f"{titel} door de tijd", "soort": "keten", "deze_wet": titels,
        "voorstel": stuk_van(begin_d), "stukken": stukken, "momenten": momenten, "refs": refs, "begin": begin,
        "stappen": stappen, "identiteit": {}, "toelichtingen": {}, "groepen": A.GROEPEN, "bron": {},
        "ijkpunten": [{"naam": f"De geldende tekst ({data[-1]})", "soort": "bwb",
                       "uitleg": "de laatste versie op wetten.overheid.nl", "datum": data[-1], "gelijk": len(eind),
                       "verschillend": 0, "artikelen": {nr: [A.norm(r) for r in rr] for nr, rr in eind.items()}}],
        "doorwerking": doorwerking, "gelijk": len(eind), "tel": tel,
        "commentaar": {"brokken": [], "niet": [], "tel": {"brokken": 0, "artikelsgewijs": 0, "vermelding": 0,
                                                          "amendement": 0, "artikelen_met_toelichting": 0,
                                                          "aan_de_doelwet": 0, "niet_gehaakt": 0, "gelezen": []}},
    }


def regelingen():
    return sorted(p.parent.name for p in REGELINGEN.glob("*/keten.json"))


if __name__ == "__main__":
    t = bouw(sys.argv[1])
    print(t["naam"], t["tel"])
    for d in t["doorwerking"]:
        v = d["versies"][0]
        print(f"  {d['titel']:22s} {v['versie']:28s} gespeeld {v['tel']}, afwijkend {len(v['ijk']['verschillend'])} "
              f"van {len(d['nummers'])} geraakt {v['ijk']['verschillend'][:8]}")
