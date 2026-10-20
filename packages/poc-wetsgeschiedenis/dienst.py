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
  WG_MAX         hoeveel opgehaalde dossiers en wetten er ten hoogste blijven staan (150)
  WG_LOG         hoeveel regels het logboek in het geheugen houdt (400)
  WG_CONNECT_TIMEOUT  seconden die curl krijgt om een verbinding te maken (15)

INZICHT. Elk verzoek, elke opdracht en elke vraag die de dienst naar buiten
stelt, komt als één JSON-regel op stderr (de podlogs) en in een logboek in het
geheugen. Twee eindpunten maken dat zichtbaar zonder toegang tot de pod:

  api/log        het logboek, de nieuwste regel achteraan
  api/diagnose   kan de dienst schrijven, heeft hij curl, bereikt hij elke host;
                 met ?bwb=BWBR... ook de opzoeking zelf, met haar duur

De afspeler roept curl en urllib zelf aan. De dienst meet die aanroepen door ze
in te pakken (`_curl`, `_open`); aan de afspeler verandert niets. Bij curl zet
hij er twee vlaggen bij: -S, zodat een fout haar reden op stderr zegt, en
--connect-timeout, zodat een host die niet antwoordt na seconden faalt in
plaats van na minuten (curl wacht anders tot 300 s, per poging).
"""
import collections
import contextlib
import importlib.util
import io
import json
import mimetypes
import os
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import traceback
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

HIER = Path(__file__).resolve().parent
POORT = int(os.environ.get("WG_PORT") or 8420)
BASIS = "/" + (os.environ.get("WG_BASE_PATH") or "/wetsgeschiedenis/").strip("/") + "/"
STATISCH = Path(os.environ.get("WG_STATIC_DIR") or HIER.parent.parent / "frontend-poc-wetsgeschiedenis" / "dist")
VAST = Path(os.environ.get("WG_VAST_DIR") or HIER.parent.parent / "corpus-poc" / "wetsgeschiedenis" / "gegevens")
DATA_GEVRAAGD = Path(os.environ.get("WG_DATA") or HIER / "data")


def _schrijfbaar(map_):
    try:
        map_.mkdir(parents=True, exist_ok=True)
        proef = map_ / ".schrijfproef"
        proef.write_text("ok")
        proef.unlink()
        return True
    except OSError:
        return False


# Een volume dat als root is aangekoppeld, is niet van de gebruiker `app`
# (gemeten in de preview, 20-10-2026). Dan wijkt de dienst uit naar een
# tijdelijke map: ophalen werkt, maar wat is opgehaald, is na een herstart weg.
# De diagnose zegt het.
DATA = DATA_GEVRAAGD if _schrijfbaar(DATA_GEVRAAGD) else Path(tempfile.gettempdir()) / "wetsgeschiedenis"
# Een wet met haar Kamerdossiers telt al snel twintig (de Archiefwet 1995: 24).
MAX = int(os.environ.get("WG_MAX") or 150)
OPGEHAALD = DATA / "gegevens"
VERBINDEN = str(int(os.environ.get("WG_CONNECT_TIMEOUT") or 15))


# ---------------------------------------------------------------- het logboek
LOGBOEK = collections.deque(maxlen=int(os.environ.get("WG_LOG") or 400))
_TAAK = threading.local()  # gezet in de draad van een opdracht

# Wat curl met een afsluitcode bedoelt; de meeste fouten in een nieuwe omgeving
# zijn er een van deze.
CURL_FOUT = {
    2: "curl kon niet starten",
    5: "de proxy is niet te vinden",
    6: "de host is niet te vinden (DNS)",
    7: "geen verbinding met de host (uitgaand verkeer dicht?)",
    22: "de server antwoordde met een HTTP-fout",
    18: "de overdracht brak halverwege af (een grote pagina; opnieuw proberen helpt meestal)",
    28: "geen antwoord binnen de tijd (uitgaand verkeer dicht, of de host is traag)",
    35: "de TLS-verbinding mislukte",
    47: "te vaak doorgestuurd",
    52: "de server gaf een leeg antwoord",
    56: "de verbinding brak af",
    60: "het certificaat van de host wordt niet vertrouwd",
}


def logboek(soort, **velden):
    """Eén regel: in het geheugen (api/log) en als JSON op stderr (de podlogs)."""
    regel = {"t": time.strftime("%Y-%m-%dT%H:%M:%S"), "soort": soort, **velden}
    LOGBOEK.append(regel)
    sys.stderr.write(json.dumps(regel, ensure_ascii=False) + "\n")
    sys.stderr.flush()


def _kort(url):
    u = urlsplit(url)
    pad = u.path + ("?" + u.query if u.query else "")
    return u.netloc, (pad[:160] + "…") if len(pad) > 160 else pad


def _uit(via, url, t0, **velden):
    """Een vraag naar buiten, in het logboek; een mislukte ook in het log van
    de opdracht, zodat de bezoeker haar ziet."""
    host, pad = _kort(url)
    ms = int((time.monotonic() - t0) * 1000)
    velden = {k: v for k, v in velden.items() if v is not None}
    logboek("uit", via=via, host=host, pad=pad, ms=ms, **velden)
    mislukt = velden.get("code") or velden.get("fout") or (velden.get("http") or 0) >= 400
    if mislukt and getattr(_TAAK, "actief", False):
        print(f"  ! {host}{pad[:80]}: {velden.get('uitleg') or velden.get('fout') or velden.get('http')} ({ms} ms)")


def _stderr(b):
    t = b.decode("utf-8", "replace") if isinstance(b, bytes) else (b or "")
    return " ".join(t.split())[:300] or None


_run = subprocess.run


def _curl(cmd, *args, **kw):
    """subprocess.run, maar een curl-aanroep wordt gemeten en krijgt -S en
    een verbindingstijd (zie boven)."""
    if not (isinstance(cmd, (list, tuple)) and cmd and cmd[0] == "curl"):
        return _run(cmd, *args, **kw)
    cmd = ["curl", "-S", "--connect-timeout", VERBINDEN, *cmd[1:]]
    url = next((x for x in reversed(cmd) if str(x).startswith("http")), "?")
    t0 = time.monotonic()
    try:
        r = _run(cmd, *args, **kw)
    except subprocess.CalledProcessError as e:
        _uit("curl", url, t0, code=e.returncode, uitleg=CURL_FOUT.get(e.returncode), fout=_stderr(e.stderr))
        raise
    _uit("curl", url, t0, code=r.returncode or None, uitleg=CURL_FOUT.get(r.returncode),
         fout=_stderr(r.stderr) if r.returncode else None, bytes=len(r.stdout or b"") if not r.returncode else None)
    return r


_urlopen = urllib.request.urlopen


def _open(req, *args, **kw):
    """urllib.request.urlopen, gemeten."""
    url = getattr(req, "full_url", None) or str(req)
    t0 = time.monotonic()
    try:
        r = _urlopen(req, *args, **kw)
    except urllib.error.HTTPError as e:
        _uit("urllib", url, t0, http=e.code)
        raise
    except Exception as e:
        _uit("urllib", url, t0, fout=f"{type(e).__name__}: {e}"[:300])
        raise
    _uit("urllib", url, t0, http=getattr(r, "status", None))
    return r


subprocess.run = _curl
urllib.request.urlopen = _open


def uitleg(e):
    """Een fout in woorden die een bezoeker kan doorgeven."""
    if isinstance(e, subprocess.CalledProcessError) and e.cmd and e.cmd[0] == "curl":
        url = next((x for x in reversed(e.cmd) if str(x).startswith("http")), "?")
        reden = CURL_FOUT.get(e.returncode, "onbekende fout")
        detail = _stderr(e.stderr)
        return f"{_kort(url)[0]}: {reden} (curl {e.returncode}{': ' + detail if detail else ''})"
    if isinstance(e, urllib.error.HTTPError):
        return f"{_kort(e.url or '?')[0]} antwoordde met HTTP {e.code}"
    return f"{type(e).__name__}: {e}"

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


def bewaar(t, bij=None):
    """Zet een tijdlijn bij de opgehaalde; de oudste valt weg boven MAX. `bij`
    is de wet waarvoor de rij dit dossier ophaalde."""
    OPGEHAALD.mkdir(parents=True, exist_ok=True)
    (OPGEHAALD / f"{t['dossier']}.json").write_text(json.dumps(t, ensure_ascii=False, separators=(",", ":")),
                                                    encoding="utf-8")
    lijst = [d for d in lees_json(OPGEHAALD / "dossiers.json", []) if d["dossier"] != t["dossier"]]
    oud = next((d for d in lees_json(OPGEHAALD / "dossiers.json", []) if d["dossier"] == t["dossier"]), {})
    lijst.append({"dossier": t["dossier"], "naam": t["naam"], "soort": t["soort"],
                  **({"bij": bij or oud.get("bij")} if bij or oud.get("bij") else {})})
    for weg in lijst[:-MAX]:
        with contextlib.suppress(OSError):
            (OPGEHAALD / f"{weg['dossier']}.json").unlink()
    (OPGEHAALD / "dossiers.json").write_text(json.dumps(lijst[-MAX:], ensure_ascii=False, indent=1), encoding="utf-8")


# ---------------------------------------------------------------- de opdracht
class Taak:
    """Eén opdracht: een wet door de tijd (keten), of één Kamerdossier. Ze
    lopen na elkaar in de rij (Rij); de keten schrijft in gedeelde mappen en
    print naar stdout, en twee tegelijk zouden elkaars log en bron delen."""

    _teller = 0

    def __init__(self, wat, bwb, dossier=None, voor=None):
        Taak._teller += 1
        self.id = f"{int(time.time())}-{Taak._teller}"
        self.wat, self.bwb, self.dossier = wat, bwb, dossier
        # Een dossier dat de rij voor een wet ophaalt: na afloop wordt die wet
        # opnieuw opgebouwd, met de toelichting van dit dossier erin.
        self.voor = voor
        self.status, self.uitkomst, self.fout = "wacht", None, None
        self.log = io.StringIO()

    def sleutel(self):
        return (self.wat, self.bwb if self.wat == "keten" else None, self.dossier)

    def regels(self):
        return [r for r in self.log.getvalue().splitlines() if r.strip()][-40:]

    def als_json(self):
        return {"id": self.id, "wat": self.wat, "bwb": self.bwb, "dossier": self.dossier, "voor": self.voor,
                "status": self.status, "plaats": Rij.plaats(self), "uitkomst": self.uitkomst, "fout": self.fout,
                "log": self.regels()}

    def kort(self):
        return {"id": self.id, "wat": self.wat, "bwb": self.bwb, "dossier": self.dossier, "voor": self.voor,
                "status": self.status}

    def draai(self):
        _TAAK.actief = True
        self.status = "bezig"
        t0 = time.monotonic()
        logboek("taak", id=self.id, wat=self.wat, bwb=self.bwb, dossier=self.dossier, voor=self.voor, status="bezig")
        try:
            with contextlib.redirect_stdout(self.log):
                if self.wat == "keten":
                    print(f"De versies van {self.bwb} en de bekendmakingen ertussen ophalen")
                    kamerdossier.main([self.bwb, "--keten"])
                    print("Elke overgang afspelen, met de toelichting die er al is")
                    t = keten.bouw(self.bwb, tijdlijn_van=tijdlijn_van, status_van=status_van)
                    bewaar(t)
                    n = verdiep(self.bwb)
                    print(f"Klaar: {t['naam']}" + (f"; de rij haalt nog {n} Kamerdossier(s) op" if n else ""))
                else:
                    print(f"Dossier {self.dossier} ophalen")
                    kamerdossier.main([self.bwb, "--dossier", self.dossier])
                    print(f"Dossier {self.dossier} afspelen")
                    t, _ = bouw.tijdlijn(self.dossier)
                    bewaar(t, bij=self.voor)
                    MISLUKT.pop(self.dossier, None)
                    print(f"Klaar: {t['naam']}")
                    if self.voor and (keten.REGELINGEN / self.voor / "keten.json").exists():
                        print(f"{self.voor} opnieuw opbouwen, met de toelichting van dossier {self.dossier}")
                        bewaar(keten.bouw(self.voor, tijdlijn_van=tijdlijn_van, status_van=status_van))
            self.status, self.uitkomst = "klaar", t["dossier"]
        except SystemExit as e:
            self.status, self.fout = "fout", str(e.code or "gestopt")
        except Exception as e:  # noqa: BLE001 -- de bezoeker hoort wat er misging
            self.status, self.fout = "fout", uitleg(e)
            traceback.print_exc(file=self.log)
            traceback.print_exc()
        finally:
            _TAAK.actief = False
            if self.status == "fout" and self.wat == "dossier":
                mislukt(self.dossier, self.fout)
            logboek("taak", id=self.id, wat=self.wat, bwb=self.bwb, dossier=self.dossier, voor=self.voor,
                    status=self.status, s=round(time.monotonic() - t0, 1), uitkomst=self.uitkomst, fout=self.fout)


class Rij:
    """De opdrachten, één tegelijk. Wat een bezoeker vraagt, gaat vóór wat de
    rij op de achtergrond ophaalt; dezelfde opdracht staat er nooit twee keer
    in."""

    wacht = threading.Condition()
    wachtend = collections.deque()
    huidig = None
    alle = collections.OrderedDict()  # id -> Taak, de laatste honderd
    draad = None

    @classmethod
    def voeg(cls, taak, achtergrond=False):
        with cls.wacht:
            for t in ([cls.huidig] if cls.huidig and cls.huidig.status == "bezig" else []) + list(cls.wachtend):
                if t.sleutel() == taak.sleutel():
                    if not achtergrond and t in cls.wachtend and t.voor:
                        # Wat de bezoeker nu vraagt, gaat voor.
                        cls.wachtend.remove(t)
                        cls.wachtend.appendleft(t)
                    return t
            if achtergrond:
                cls.wachtend.append(taak)
            else:
                # Vóór de achtergrond, achter wat andere bezoekers al vroegen.
                i = next((n for n, t in enumerate(cls.wachtend) if t.voor), len(cls.wachtend))
                cls.wachtend.insert(i, taak)
            cls.alle[taak.id] = taak
            while len(cls.alle) > 100:
                cls.alle.popitem(last=False)
            cls.wacht.notify()
        cls.start()
        return taak

    @classmethod
    def plaats(cls, taak):
        return list(cls.wachtend).index(taak) + 1 if taak in cls.wachtend else None

    @classmethod
    def start(cls):
        if cls.draad is None or not cls.draad.is_alive():
            cls.draad = threading.Thread(target=cls.werk, daemon=True)
            cls.draad.start()

    @classmethod
    def werk(cls):
        while True:
            with cls.wacht:
                while not cls.wachtend:
                    cls.wacht.wait()
                cls.huidig = cls.wachtend.popleft()
                cls.huidig.status = "bezig"  # meteen, anders lijkt de rij even leeg
            cls.huidig.draai()

    @classmethod
    def in_de_rij(cls, nr):
        t = cls.huidig
        if t and t.status == "bezig" and t.dossier == nr:
            return "bezig"
        return "in de rij" if any(x.dossier == nr for x in list(cls.wachtend)) else None

    @classmethod
    def overzicht(cls):
        return {"huidig": cls.huidig.kort() if cls.huidig and cls.huidig.status == "bezig" else None,
                "wachtend": [t.kort() for t in list(cls.wachtend)]}


# ---------------------------------------------------------------- wat er is, en wat niet
def tijdlijn_van(nr):
    """De tijdlijn van een dossier of wet, uit het image of opgehaald."""
    for map_ in (OPGEHAALD, VAST):  # opnieuw opgehaald is rijker dan het image
        if ID.fullmatch(nr) and (map_ / f"{nr}.json").is_file():
            return lees_json(map_ / f"{nr}.json", None)
    return None


MISLUKT_PAD = OPGEHAALD / "mislukt.json"
MISLUKT = lees_json(MISLUKT_PAD, {}) if MISLUKT_PAD.exists() else {}


def mislukt(nr, reden):
    """Een dossier dat niet lukte: de rij probeert het niet vanzelf opnieuw (een
    bezoeker kan het wel)."""
    MISLUKT[nr] = {"reden": reden, "t": time.strftime("%Y-%m-%d")}
    with contextlib.suppress(OSError):
        OPGEHAALD.mkdir(parents=True, exist_ok=True)
        MISLUKT_PAD.write_text(json.dumps(MISLUKT, ensure_ascii=False, indent=1), encoding="utf-8")


def status_van(nr):
    """Waarom een dossier er (nog) niet is; de keten zet het bij zijn overgang."""
    rij = Rij.in_de_rij(nr)
    if rij:
        return {"status": rij}
    if nr in MISLUKT:
        return {"status": "mislukt", "reden": MISLUKT[nr]["reden"]}
    return {"status": "niet opgehaald"}


def verdiep(bwb):
    """Zet elk Kamerdossier van deze wet dat er nog niet is in de rij, de
    jongste eerst: die hebben het vaakst XML en zijn het rijkst. Geeft hoeveel."""
    k = lees_json(keten.REGELINGEN / bwb / "keten.json", None)
    if not k:
        return 0
    nrs = list(dict.fromkeys(nr for o in sorted(k["overgangen"], key=lambda o: o["tot"], reverse=True)
                             for nr in o["dossiers"]))
    n = 0
    for nr in nrs:
        if tijdlijn_van(nr) or nr in MISLUKT:
            continue
        Rij.voeg(Taak("dossier", bwb, nr, voor=bwb), achtergrond=True)
        n += 1
    return n


def herstel():
    """Na een herstart: zet weer in de rij wat de opgehaalde wetten nog missen."""
    n = 0
    for d in index():
        if d.get("soort") == "keten" and (keten.REGELINGEN / d["dossier"] / "keten.json").exists():
            n += verdiep(d["dossier"])
    if n:
        logboek("rij", herstel=n)
    return n


def regeling(bwb):
    """Wat wetten.overheid.nl over deze wet weet, met de dossiers per versie."""
    t0 = time.monotonic()
    reg = kamerdossier.regeling(bwb)
    logboek("regeling", bwb=bwb, versies=len(reg["versies"]),
            met_dossier=sum(1 for v in reg["versies"] if v["dossiers"]), s=round(time.monotonic() - t0, 1))
    beschikbaar = {d["dossier"] for d in index()}
    return {
        "bwb": reg["bwb"], "titel": reg["titel"].split("|")[0], "afkorting": reg.get("afkorting"),
        "keten": bwb in beschikbaar,
        "versies": [{"inwerking": v["inwerking"], "betreft": v["betreft"], "staatsblad": v["staatsblad"],
                     "dossiers": [{"nr": d, "beschikbaar": d in beschikbaar, "rij": Rij.in_de_rij(d),
                                   **({"mislukt": MISLUKT[d]["reden"]} if d in MISLUKT and d not in beschikbaar else {})}
                                  for d in v["dossiers"]],
                     "artikelen": len(v.get("artikelen") or [])}
                    for v in reg["versies"] if v["dossiers"]],
    }


# ---------------------------------------------------------------- diagnose
HOSTS = {
    "wetten.overheid.nl": "nodig: de versies van een wet en de dossiers die haar maakten",
    "zoek.officielebekendmakingen.nl": "nodig: Kamerstukken en Staatsbladen",
    "repository.overheid.nl": "nodig: de zoekdienst naar Kamerstukken (SRU)",
    "gegevensmagazijn.tweedekamer.nl": "nodig: documenten en stemmingen van de Tweede Kamer",
    "repository.officiele-overheidspublicaties.nl": "niet nodig: de BWB-XML, alleen als wetten.overheid.nl een versie mist",
}


def _host(host, uit):
    """Eén echte vraag naar de host. Elk HTTP-antwoord, ook 404, betekent dat
    het uitgaande verkeer er doorheen komt."""
    cmd = ["curl", "-sS", "-o", os.devnull, "--connect-timeout", VERBINDEN, "--max-time", "20",
           "-A", "Mozilla/5.0 (regelrecht-werkbank)",
           "-w", "%{http_code} %{remote_ip} %{time_namelookup} %{time_connect} %{time_total}", f"https://{host}/"]
    try:
        r = _run(cmd, capture_output=True, text=True, timeout=30)
    except Exception as e:  # noqa: BLE001
        uit[host] = {"ok": False, "fout": f"{type(e).__name__}: {e}"}
        return
    code, ip, dns, verbinding, totaal = (r.stdout.split() + ["", "", "", "", ""])[:5]
    uit[host] = {"ok": r.returncode == 0 and code not in ("", "000"), "http": code, "ip": ip or None,
                 "dns_s": dns, "verbinding_s": verbinding, "totaal_s": totaal,
                 "curl": r.returncode or None, "uitleg": CURL_FOUT.get(r.returncode), "fout": _stderr(r.stderr)}


def _zonder_wachtwoord(url):
    return re.sub(r"//[^@/]*@", "//***@", url)


def diagnose(bwb=None):
    """Wat een beheerder wil weten als ophalen faalt, zonder in de pod te kijken."""
    t0 = time.monotonic()
    d = {"tijd": time.strftime("%Y-%m-%dT%H:%M:%S"), "basis": BASIS, "python": sys.version.split()[0],
         "verbindingstijd_s": int(VERBINDEN)}
    try:
        d["curl"] = _run(["curl", "--version"], capture_output=True, text=True).stdout.splitlines()[0]
    except OSError as e:
        d["curl"] = f"ontbreekt ({e})"
    d["data"] = {"pad": str(DATA), "schrijfbaar": _schrijfbaar(DATA), "proces": f"uid {os.getuid()}, gid {os.getgid()}"}
    if d["data"]["schrijfbaar"]:
        d["data"]["vrij_mb"] = shutil.disk_usage(DATA).free // 2**20
    if DATA != DATA_GEVRAAGD:
        try:
            st = DATA_GEVRAAGD.stat()
            eigen = f"van uid {st.st_uid}, gid {st.st_gid}, modus {oct(st.st_mode & 0o777)}"
        except OSError as e:
            eigen = f"niet te lezen ({e})"
        d["data"]["uitgeweken"] = (f"{DATA_GEVRAAGD} is niet schrijfbaar ({eigen}); de dienst schrijft in "
                                   f"{DATA}, en wat is opgehaald, is na een herstart weg")
    d["proxy"] = {k: _zonder_wachtwoord(v) for k in ("HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy",
                                                     "NO_PROXY", "no_proxy") if (v := os.environ.get(k))}
    d["dossiers"] = {"in_het_image": len(lees_json(VAST / "dossiers.json", [])),
                     "opgehaald": len(lees_json(OPGEHAALD / "dossiers.json", [])), "max": MAX}
    hosts = {}
    draden = [threading.Thread(target=_host, args=(h, hosts)) for h in HOSTS]
    for t in draden:
        t.start()
    for t in draden:
        t.join()
    d["hosts"] = {h: dict(hosts.get(h, {}), waarvoor=w) for h, w in HOSTS.items()}
    if bwb:
        t1 = time.monotonic()
        try:
            r = regeling(bwb)
            d["opzoeking"] = {"bwb": bwb, "ok": True, "titel": r["titel"], "versies_met_dossier": len(r["versies"])}
        except SystemExit as e:
            d["opzoeking"] = {"bwb": bwb, "ok": False, "fout": str(e.code)}
        except Exception as e:  # noqa: BLE001
            d["opzoeking"] = {"bwb": bwb, "ok": False, "fout": uitleg(e)}
        d["opzoeking"]["s"] = round(time.monotonic() - t1, 1)
    d["taak"] = Rij.huidig.als_json() if Rij.huidig else None
    d["rij"] = Rij.overzicht()
    nodig = [h for h, w in HOSTS.items() if w.startswith("nodig") and not d["hosts"][h].get("ok")]
    d["oordeel"] = ("ophalen kan" if not nodig and d["data"]["schrijfbaar"] and not d["curl"].startswith("ontbreekt")
                    else "ophalen faalt: " + "; ".join(
                        ([f"{h} niet bereikbaar" for h in nodig])
                        + ([] if d["data"]["schrijfbaar"] else [f"{DATA} niet schrijfbaar"])
                        + (["curl ontbreekt"] if d["curl"].startswith("ontbreekt") else [])))
    if d["oordeel"] == "ophalen kan" and DATA != DATA_GEVRAAGD:
        d["oordeel"] = "ophalen kan, maar niet blijvend: " + d["data"]["uitgeweken"]
    d["s"] = round(time.monotonic() - t0, 1)
    logboek("diagnose", oordeel=d["oordeel"], s=d["s"])
    return d


# ---------------------------------------------------------------- HTTP
class Verzoek(BaseHTTPRequestHandler):
    server_version = "wetsgeschiedenis"

    # Het pollen van de voortgang en van het logboek zelf zou het logboek vullen.
    STIL = ("api/taak", "api/log")

    def handle_one_request(self):
        self._t0, self._status = time.monotonic(), None
        super().handle_one_request()
        if self._status is None or not self.command:
            return
        pad = (self.path or "").split("?")[0]
        if self._status < 400 and pad.endswith(self.STIL):
            return
        logboek("verzoek", methode=self.command, pad=pad, status=self._status,
                ms=int((time.monotonic() - self._t0) * 1000))

    def send_response(self, code, message=None):
        self._status = code
        super().send_response(code, message)

    def log_message(self, fmt, *args):  # elk verzoek staat al in het logboek
        pass

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
            h = Rij.huidig if Rij.huidig and Rij.huidig.status == "bezig" else None
            return self.stuur(200, {"ophalen": True, "taak": h.als_json() if h else None, "rij": Rij.overzicht()})
        if rest == "api/taak":
            # ?id=: die ene opdracht, ook als zij nog wacht of al klaar is.
            tid = (parse_qs(urlsplit(self.path).query).get("id") or [""])[0]
            t = Rij.alle.get(tid) if tid else Rij.huidig
            return self.stuur(200 if t or not tid else 404, t.als_json() if t else None)
        if rest == "api/log":
            return self.stuur(200, list(LOGBOEK))
        if rest == "api/diagnose":
            bwb = (parse_qs(urlsplit(self.path).query).get("bwb") or [""])[0].strip().upper()
            if bwb and not BWB.fullmatch(bwb):
                return self.stuur(400, {"fout": "een BWB-id ziet eruit als BWBR0007376"})
            return self.stuur(200, diagnose(bwb or None))
        m = re.fullmatch(r"api/regeling/(.+)", rest)
        if m:
            if not BWB.fullmatch(m[1]):
                return self.stuur(400, {"fout": "een BWB-id ziet eruit als BWBR0007376"})
            try:
                return self.stuur(200, regeling(m[1]))
            except SystemExit as e:  # kamerdossier stopt met een zin
                return self.stuur(404, {"fout": str(e.code)})
            except Exception as e:  # noqa: BLE001
                logboek("fout", bij=f"api/regeling/{m[1]}", fout=uitleg(e))
                return self.stuur(502, {"fout": f"De opzoeking van {m[1]} op wetten.overheid.nl mislukte. {uitleg(e)}"})
        if rest == "gegevens/dossiers.json":
            return self.stuur(200, index())
        m = re.fullmatch(r"gegevens/(.+)\.json", rest)
        if m:
            if not ID.fullmatch(m[1]):
                return self.stuur(404, {"fout": "onbekend"})
            for map_ in (OPGEHAALD, VAST):  # opnieuw opgehaald is rijker dan het image
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

    def lees_body(self, grens=4096):
        """De body, met Content-Length of in stukken (chunked). Het portaal
        stuurt een verzoek als stroom door; dan is er geen Content-Length."""
        if "chunked" in (self.headers.get("Transfer-Encoding") or "").lower():
            data = b""
            while True:
                n = int(self.rfile.readline(64).split(b";")[0].strip() or b"0", 16)
                if n == 0:
                    while self.rfile.readline(1024) not in (b"\r\n", b"\n", b""):
                        pass
                    return data
                if len(data) + n > grens:
                    raise ValueError("te groot")
                data += self.rfile.read(n)
                self.rfile.readline(8)
        return self.rfile.read(min(int(self.headers.get("Content-Length") or 0), grens))

    def do_POST(self):
        if self.pad() != BASIS + "api/haal":
            return self.stuur(404, {"fout": "onbekend pad"})
        try:
            ruw = self.lees_body()
            vraag = json.loads(ruw or b"{}")
        except ValueError as e:
            return self.stuur(400, {"fout": f"geen JSON ({e})"})
        if not ruw:
            # Zo kwam het binnen toen het portaal de body in stukken doorgaf
            # en de dienst alleen Content-Length las (preview, 20-10-2026).
            logboek("fout", bij="api/haal", fout="lege body", content_length=self.headers.get("Content-Length"),
                    transfer_encoding=self.headers.get("Transfer-Encoding"))
        bwb, nr = str(vraag.get("bwb") or ""), vraag.get("dossier")
        if not BWB.fullmatch(bwb):
            return self.stuur(400, {"fout": "een BWB-id ziet eruit als BWBR0007376"})
        if vraag.get("keten"):
            taak = Rij.voeg(Taak("keten", bwb))
        elif nr and DOSSIER.fullmatch(str(nr)):
            taak = Rij.voeg(Taak("dossier", bwb, str(nr)))
        else:
            return self.stuur(400, {"fout": "kies een dossier (5 cijfers) of de hele wet"})
        return self.stuur(202, taak.als_json())


def main():
    DATA.mkdir(parents=True, exist_ok=True)
    print(f"wetsgeschiedenis luistert op :{POORT}{BASIS} (gegevens in {DATA})", flush=True)
    herstel()
    ThreadingHTTPServer(("0.0.0.0", POORT), Verzoek).serve_forever()


if __name__ == "__main__":
    main()
