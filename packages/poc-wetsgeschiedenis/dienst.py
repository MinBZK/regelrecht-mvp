# -*- coding: utf-8 -*-
"""De wetsgeschiedenis als dienst: de pagina, en een wet ophalen vanaf haar BWB-id.

Achter het poc-portaal staat de pagina onder /wetsgeschiedenis/. Wat zij laat
zien, komt uit twee bronnen:

- de dossiers die al in het image zitten (corpus-poc/wetsgeschiedenis/gegevens),
  dezelfde als in het losse bestand;
- wat een bezoeker zelf ophaalt. Hij geeft een BWB-id, de dienst leest op
  wetten.overheid.nl welke Kamerdossiers die wet maakten en wijzigden, en hij
  kiest er een (of de hele wet door de tijd). De dienst haalt de stukken op,
  speelt ze af met dezelfde keten als het dossier waarin zij is ontwikkeld (afspeler/, een kopie
  van werkbank/generatoren daar) en zet de tijdlijn naast de andere.

Wat is opgehaald staat in WG_DATA en is van iedereen die de poc opent; er
draait één opdracht tegelijk. De dienst haalt alleen op van wetten.overheid.nl,
zoek.officielebekendmakingen.nl, repository.overheid.nl en het Gegevensmagazijn
van de Tweede Kamer; het component heeft daar uitgaand verkeer naartoe nodig.

Alleen de standaardbibliotheek, pyyaml en curl.

  WG_PORT        luisterpoort (8420; het image zet 8000)
  WG_BASE_PATH   het voorvoegsel, gelijk aan het pad in het portaal (/wetsgeschiedenis/)
  WG_STATIC_DIR  de gebouwde frontend (dist)
  WG_VAST_DIR    de gegevens die in het image zitten
  WG_DATA        waar opgehaalde stukken en tijdlijnen landen
  WG_MAX         hoeveel opgehaalde dossiers en wetten er ten hoogste blijven staan (40)
"""
import contextlib
import importlib.util
import io
import json
import mimetypes
import os
import re
import sys
import threading
import time
import traceback
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

HIER = Path(__file__).resolve().parent
POORT = int(os.environ.get("WG_PORT") or 8420)
BASIS = "/" + (os.environ.get("WG_BASE_PATH") or "/wetsgeschiedenis/").strip("/") + "/"
STATISCH = Path(os.environ.get("WG_STATIC_DIR") or HIER.parent.parent / "frontend-poc-wetsgeschiedenis" / "dist")
VAST = Path(os.environ.get("WG_VAST_DIR") or HIER.parent.parent / "corpus-poc" / "wetsgeschiedenis" / "gegevens")
DATA = Path(os.environ.get("WG_DATA") or HIER / "data")
MAX = int(os.environ.get("WG_MAX") or 40)
OPGEHAALD = DATA / "gegevens"

# De afspeelketen leest haar mappen bij het laden; dus eerst de omgeving.
os.environ.setdefault("WETSGESCHIEDENIS_BRON", str(DATA / "bron"))
os.environ.setdefault("WETSGESCHIEDENIS_WERK", str(DATA / "werk"))
AFSPELER = HIER / "afspeler"
sys.path.insert(0, str(AFSPELER))


def _laad(naam, bestand):
    spec = importlib.util.spec_from_file_location(naam, AFSPELER / bestand)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


kamerdossier = _laad("kamerdossier", "kamerdossier.py")
bouw = _laad("bouw_wetsgeschiedenis", "bouw-wetsgeschiedenis.py")
import keten  # noqa: E402

BWB = re.compile(r"BWBR\d{7}")
DOSSIER = re.compile(r"\d{5}(?:-[A-Z0-9]{1,6})?")
ID = re.compile(r"[0-9A-Za-z-]{1,24}")


def lees_json(pad, anders):
    try:
        return json.loads(Path(pad).read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return anders


def index():
    """De keuzelijst: eerst wat in het image zit, dan wat is opgehaald."""
    vast = lees_json(VAST / "dossiers.json", [])
    eigen = [d for d in lees_json(OPGEHAALD / "dossiers.json", []) if d["dossier"] not in {v["dossier"] for v in vast}]
    return vast + [dict(d, opgehaald=True) for d in eigen]


def bewaar(t):
    """Zet een tijdlijn bij de opgehaalde; de oudste valt weg boven MAX."""
    OPGEHAALD.mkdir(parents=True, exist_ok=True)
    (OPGEHAALD / f"{t['dossier']}.json").write_text(json.dumps(t, ensure_ascii=False, separators=(",", ":")),
                                                    encoding="utf-8")
    lijst = [d for d in lees_json(OPGEHAALD / "dossiers.json", []) if d["dossier"] != t["dossier"]]
    lijst.append({"dossier": t["dossier"], "naam": t["naam"], "soort": t["soort"]})
    for weg in lijst[:-MAX]:
        with contextlib.suppress(OSError):
            (OPGEHAALD / f"{weg['dossier']}.json").unlink()
    (OPGEHAALD / "dossiers.json").write_text(json.dumps(lijst[-MAX:], ensure_ascii=False, indent=1), encoding="utf-8")


# ---------------------------------------------------------------- de opdracht
class Taak:
    """Eén opdracht tegelijk: de keten schrijft in gedeelde mappen en print
    naar stdout, en twee tegelijk zouden elkaars log en bron delen."""

    slot = threading.Lock()
    huidig = None

    def __init__(self, wat, bwb, dossier=None):
        self.id = f"{int(time.time())}"
        self.wat, self.bwb, self.dossier = wat, bwb, dossier
        self.status, self.uitkomst, self.fout = "bezig", None, None
        self.log = io.StringIO()

    def regels(self):
        return [r for r in self.log.getvalue().splitlines() if r.strip()][-40:]

    def als_json(self):
        return {"id": self.id, "wat": self.wat, "bwb": self.bwb, "dossier": self.dossier, "status": self.status,
                "uitkomst": self.uitkomst, "fout": self.fout, "log": self.regels()}

    def draai(self):
        try:
            with contextlib.redirect_stdout(self.log):
                if self.wat == "keten":
                    print(f"De versies van {self.bwb} en de Staatsbladen ertussen ophalen")
                    kamerdossier.main([self.bwb, "--keten"])
                    print("Elke overgang afspelen")
                    t = keten.bouw(self.bwb)
                else:
                    print(f"Dossier {self.dossier} ophalen")
                    kamerdossier.main([self.bwb, "--dossier", self.dossier])
                    print(f"Dossier {self.dossier} afspelen")
                    t, _ = bouw.tijdlijn(self.dossier)
                bewaar(t)
                print(f"Klaar: {t['naam']}")
            self.status, self.uitkomst = "klaar", t["dossier"]
        except SystemExit as e:
            self.status, self.fout = "fout", str(e.code or "gestopt")
        except Exception as e:  # noqa: BLE001 -- de bezoeker hoort wat er misging
            self.status, self.fout = "fout", f"{type(e).__name__}: {e}"
            traceback.print_exc()
        finally:
            Taak.slot.release()

    @classmethod
    def start(cls, wat, bwb, dossier=None):
        if not cls.slot.acquire(blocking=False):
            return None
        cls.huidig = Taak(wat, bwb, dossier)
        threading.Thread(target=cls.huidig.draai, daemon=True).start()
        return cls.huidig


def regeling(bwb):
    """Wat wetten.overheid.nl over deze wet weet, met de dossiers per versie."""
    reg = kamerdossier.regeling(bwb)
    beschikbaar = {d["dossier"] for d in index()}
    return {
        "bwb": reg["bwb"], "titel": reg["titel"].split("|")[0], "afkorting": reg.get("afkorting"),
        "keten": bwb in beschikbaar,
        "versies": [{"inwerking": v["inwerking"], "betreft": v["betreft"], "staatsblad": v["staatsblad"],
                     "dossiers": [{"nr": d, "beschikbaar": d in beschikbaar} for d in v["dossiers"]],
                     "artikelen": len(v.get("artikelen") or [])}
                    for v in reg["versies"] if v["dossiers"]],
    }


# ---------------------------------------------------------------- HTTP
class Verzoek(BaseHTTPRequestHandler):
    server_version = "wetsgeschiedenis"

    def log_message(self, fmt, *args):  # één regel per verzoek, zonder query
        sys.stderr.write("%s %s\n" % (self.command, self.path.split("?")[0]))

    def stuur(self, status, body, soort="application/json; charset=utf-8", kop=None):
        if not isinstance(body, (bytes, bytearray)):
            body = json.dumps(body, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", soort)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store" if "json" in soort else "public, max-age=300")
        for k, v in (kop or {}).items():
            self.send_header(k, v)
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(body)

    def pad(self):
        return self.path.split("?", 1)[0]

    def do_HEAD(self):
        self.do_GET()

    def do_GET(self):
        p = self.pad()
        if p == "/healthz":
            return self.stuur(200, b"ok", "text/plain; charset=utf-8")
        # Los gedraaid (zonder portaal ervoor) komt een bezoeker op de wortel binnen.
        if p == BASIS.rstrip("/") or (p == "/" and BASIS != "/"):
            return self.stuur(308, b"", "text/plain", {"Location": BASIS})
        if not p.startswith(BASIS):
            return self.stuur(404, {"fout": "onbekend pad"})
        rest = p[len(BASIS):]
        if rest == "api/status":
            return self.stuur(200, {"ophalen": True, "taak": Taak.huidig.als_json() if Taak.huidig else None})
        if rest == "api/taak":
            return self.stuur(200, Taak.huidig.als_json() if Taak.huidig else None)
        m = re.fullmatch(r"api/regeling/(.+)", rest)
        if m:
            if not BWB.fullmatch(m[1]):
                return self.stuur(400, {"fout": "een BWB-id ziet eruit als BWBR0007376"})
            try:
                return self.stuur(200, regeling(m[1]))
            except Exception as e:  # noqa: BLE001
                return self.stuur(502, {"fout": f"wetten.overheid.nl gaf geen antwoord voor {m[1]} ({e})"})
        if rest == "gegevens/dossiers.json":
            return self.stuur(200, index())
        m = re.fullmatch(r"gegevens/(.+)\.json", rest)
        if m:
            if not ID.fullmatch(m[1]):
                return self.stuur(404, {"fout": "onbekend"})
            for map_ in (VAST, OPGEHAALD):
                if (map_ / f"{m[1]}.json").is_file():
                    return self.stuur(200, (map_ / f"{m[1]}.json").read_bytes())
            return self.stuur(404, {"fout": f"{m[1]} is er niet"})
        return self.bestand(rest)

    def bestand(self, rest):
        """De gebouwde frontend; wat er niet is, krijgt de pagina zelf."""
        wortel = STATISCH.resolve()
        doel = (wortel / rest).resolve()
        if wortel not in doel.parents or not doel.is_file():
            doel = wortel / "index.html"
        if not doel.is_file():
            return self.stuur(503, b"de frontend is niet gebouwd", "text/plain; charset=utf-8")
        soort = mimetypes.guess_type(doel.name)[0] or "application/octet-stream"
        if soort.startswith("text/") or soort.endswith("javascript"):
            soort += "; charset=utf-8"
        return self.stuur(200, doel.read_bytes(), soort)

    def do_POST(self):
        if self.pad() != BASIS + "api/haal":
            return self.stuur(404, {"fout": "onbekend pad"})
        try:
            n = int(self.headers.get("Content-Length") or 0)
            vraag = json.loads(self.rfile.read(min(n, 4096)) or b"{}")
        except ValueError:
            return self.stuur(400, {"fout": "geen JSON"})
        bwb, nr = str(vraag.get("bwb") or ""), vraag.get("dossier")
        if not BWB.fullmatch(bwb):
            return self.stuur(400, {"fout": "een BWB-id ziet eruit als BWBR0007376"})
        if vraag.get("keten"):
            taak = Taak.start("keten", bwb)
        elif nr and DOSSIER.fullmatch(str(nr)):
            taak = Taak.start("dossier", bwb, str(nr))
        else:
            return self.stuur(400, {"fout": "kies een dossier (5 cijfers) of de hele wet"})
        if taak is None:
            return self.stuur(409, {"fout": "er loopt al een opdracht", "taak": Taak.huidig.als_json()})
        return self.stuur(202, taak.als_json())


def main():
    DATA.mkdir(parents=True, exist_ok=True)
    print(f"wetsgeschiedenis luistert op :{POORT}{BASIS} (gegevens in {DATA})", flush=True)
    ThreadingHTTPServer(("0.0.0.0", POORT), Verzoek).serve_forever()


if __name__ == "__main__":
    main()
