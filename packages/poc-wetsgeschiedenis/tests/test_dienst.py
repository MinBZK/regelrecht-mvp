# -*- coding: utf-8 -*-
"""De dienst zonder netwerk: elk verzoek gaat over een socketpair, niet over
een poort, en een opdracht die het internet op zou gaan, wordt vervangen.

    python3 -m unittest discover packages/poc-wetsgeschiedenis/tests
"""
import json
import os
import socket
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path

HIER = Path(__file__).resolve().parent
TMP = tempfile.mkdtemp(prefix="wg-dienst-")
VAST = Path(TMP) / "vast"
STATISCH = Path(TMP) / "dist"
VAST.mkdir()
STATISCH.mkdir()
(VAST / "dossiers.json").write_text(json.dumps([{"dossier": "35968", "naam": "Archiefwet 2026", "soort": "nieuwe wet"}]))
(VAST / "35968.json").write_text(json.dumps({"dossier": "35968", "naam": "Archiefwet 2026"}))
(STATISCH / "index.html").write_text("<!doctype html><title>t</title>")
(Path(TMP) / "geheim.txt").write_text("buiten de frontend")
os.environ.update(WG_VAST_DIR=str(VAST), WG_STATIC_DIR=str(STATISCH), WG_DATA=str(Path(TMP) / "data"), WG_MAX="2")
sys.path.insert(0, str(HIER.parent))
import dienst  # noqa: E402


def vraag(methode, pad, body=None, stukken=False):
    """Een verzoek; met `stukken` gaat de body zoals het portaal hem doorstuurt:
    in stukken (chunked), zonder Content-Length."""
    a, b = socket.socketpair()
    data = json.dumps(body).encode() if body is not None else b""
    if stukken:
        midden = len(data) // 2
        delen = b"".join(b"%x\r\n%s\r\n" % (len(d), d) for d in (data[:midden], data[midden:]) if d)
        b.sendall(f"{methode} {pad} HTTP/1.1\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n".encode()
                  + delen + b"0\r\n\r\n")
    else:
        b.sendall(f"{methode} {pad} HTTP/1.0\r\nContent-Length: {len(data)}\r\n\r\n".encode() + data)
    t = threading.Thread(target=dienst.Verzoek, args=(a, ("test", 0), None))
    t.start()
    t.join(10)
    a.close()
    antwoord = b""
    while chunk := b.recv(65536):
        antwoord += chunk
    b.close()
    kop, _, inhoud = antwoord.partition(b"\r\n\r\n")
    status = int(kop.split()[1])
    try:
        return status, json.loads(inhoud)
    except ValueError:
        return status, inhoud


class Dienst(unittest.TestCase):
    def test_status_zegt_dat_er_opgehaald_kan_worden(self):
        status, x = vraag("GET", "/wetsgeschiedenis/api/status")
        self.assertEqual(status, 200)
        self.assertTrue(x["ophalen"])

    def test_de_keuzelijst_heeft_wat_in_het_image_zit(self):
        status, x = vraag("GET", "/wetsgeschiedenis/gegevens/dossiers.json")
        self.assertEqual((status, x[0]["dossier"]), (200, "35968"))
        status, x = vraag("GET", "/wetsgeschiedenis/gegevens/35968.json")
        self.assertEqual((status, x["naam"]), (200, "Archiefwet 2026"))

    def test_een_vreemd_id_of_pad_geeft_niets_van_buiten(self):
        self.assertEqual(vraag("GET", "/wetsgeschiedenis/gegevens/..%2F..%2Fgeheim.json")[0], 404)
        status, x = vraag("GET", "/wetsgeschiedenis/../geheim.txt")
        self.assertNotIn(b"buiten de frontend", x if isinstance(x, bytes) else b"")
        status, x = vraag("GET", "/wetsgeschiedenis/assets/../../geheim.txt")
        self.assertNotIn(b"buiten de frontend", x if isinstance(x, bytes) else b"")

    def test_een_onbekend_pad_krijgt_de_pagina(self):
        status, x = vraag("GET", "/wetsgeschiedenis/iets/anders")
        self.assertEqual(status, 200)
        self.assertIn(b"<!doctype html>", x)

    def test_een_verkeerd_bwb_id_wordt_geweigerd(self):
        self.assertEqual(vraag("GET", "/wetsgeschiedenis/api/regeling/BWB123")[0], 400)
        self.assertEqual(vraag("POST", "/wetsgeschiedenis/api/haal", {"bwb": "x; rm -rf /", "keten": True})[0], 400)
        self.assertEqual(vraag("POST", "/wetsgeschiedenis/api/haal", {"bwb": "BWBR0007376", "dossier": "../1"})[0], 400)

    def nep(self, poort=None):
        """Het internet en de keten vervangen: een opdracht wacht (op `poort`)
        en zet dan een dossier neer."""
        oud = (dienst.kamerdossier.main, dienst.bouw.tijdlijn, dienst.keten.bouw)

        def main(args):
            if poort and "--dossier" in args:
                poort.wait(5)
            print("stukken opgehaald")

        dienst.kamerdossier.main = main
        dienst.bouw.tijdlijn = lambda nr: ({"dossier": nr, "naam": f"Dossier {nr}", "soort": "wijzigingswet"}, None)
        dienst.keten.bouw = lambda bwb, **kw: {"dossier": bwb, "naam": f"{bwb} door de tijd", "soort": "keten",
                                               "dossiers_er": [nr for nr in ("35968", "10101", "20202", "30303")
                                                               if kw["tijdlijn_van"](nr)]}
        return oud

    def herstel(self, oud):
        dienst.kamerdossier.main, dienst.bouw.tijdlijn, dienst.keten.bouw = oud

    def wacht_op(self, tid):
        for _ in range(100):
            t = vraag("GET", f"/wetsgeschiedenis/api/taak?id={tid}")[1]
            if t["status"] in ("klaar", "fout"):
                return t
            time.sleep(0.05)
        self.fail(f"opdracht {tid} werd niet klaar")

    def test_opdrachten_lopen_na_elkaar_en_hun_uitkomst_komt_in_de_lijst(self):
        poort = threading.Event()
        oud = self.nep(poort)
        try:
            # Zoals achter het portaal: de body in stukken (preview, 20-10-2026).
            status, t1 = vraag("POST", "/wetsgeschiedenis/api/haal", {"bwb": "BWBR0007376", "dossier": "36889"},
                               stukken=True)
            self.assertEqual(status, 202)
            time.sleep(0.1)
            status, t2 = vraag("POST", "/wetsgeschiedenis/api/haal", {"bwb": "BWBR0007376", "dossier": "20202"})
            self.assertEqual((status, t2["status"], t2["plaats"]), (202, "wacht", 1))
            # Dezelfde opdracht nog eens: geen tweede in de rij.
            self.assertEqual(vraag("POST", "/wetsgeschiedenis/api/haal", {"bwb": "BWBR0007376", "dossier": "20202"})[1]["id"],
                             t2["id"])
            poort.set()
            a, b = self.wacht_op(t1["id"]), self.wacht_op(t2["id"])
            self.assertEqual((a["status"], a["uitkomst"], b["uitkomst"]), ("klaar", "36889", "20202"))
            self.assertIn("stukken opgehaald", a["log"])
            lijst = vraag("GET", "/wetsgeschiedenis/gegevens/dossiers.json")[1]
            self.assertIn("36889", [d["dossier"] for d in lijst])
            self.assertTrue(next(d for d in lijst if d["dossier"] == "36889")["opgehaald"])
        finally:
            self.herstel(oud)

    def test_een_wet_zet_haar_dossiers_op_de_achtergrond_de_jongste_eerst(self):
        bwb = "BWBR0099999"
        map_ = dienst.keten.REGELINGEN / bwb
        map_.mkdir(parents=True, exist_ok=True)
        (map_ / "keten.json").write_text(json.dumps({"overgangen": [
            {"tot": "2001-01-01", "dossiers": ["10101"]},
            {"tot": "2020-01-01", "dossiers": ["30303", "35968"]},   # 35968 zit in het image
            {"tot": "2010-01-01", "dossiers": ["20202", "40404"]}]}))
        dienst.mislukt("40404", "geen XML")
        poort = threading.Event()
        oud = self.nep(poort)
        try:
            status, t = vraag("POST", "/wetsgeschiedenis/api/haal", {"bwb": bwb, "keten": True})
            self.assertEqual(status, 202)
            for _ in range(100):
                rij = vraag("GET", "/wetsgeschiedenis/api/status")[1]["rij"]
                if len(rij["wachtend"]) >= 2:
                    break
                time.sleep(0.02)
            # Jongste eerst; wat er al is of mislukte, niet; elk voor deze wet.
            alles = ([rij["huidig"]] if rij["huidig"] else []) + rij["wachtend"]
            self.assertEqual([x["dossier"] for x in alles if x["wat"] == "dossier"], ["30303", "20202", "10101"])
            self.assertTrue(all(x["voor"] == bwb for x in alles if x["wat"] == "dossier"))
            self.assertEqual(dienst.status_van("40404"), {"status": "mislukt", "reden": "geen XML"})
            # Een bezoeker die 10101 vraagt, gaat voor op de achtergrond.
            vraag("POST", "/wetsgeschiedenis/api/haal", {"bwb": bwb, "dossier": "10101"})
            wachtend = vraag("GET", "/wetsgeschiedenis/api/status")[1]["rij"]["wachtend"]
            self.assertEqual(wachtend[0]["dossier"], "10101")
            poort.set()
            for _ in range(200):
                if not dienst.Rij.wachtend and not (dienst.Rij.huidig and dienst.Rij.huidig.status == "bezig"):
                    break
                time.sleep(0.02)
            # Na elk dossier is de wet opnieuw opgebouwd, met wat er nu is: het
            # image en het laatst opgehaalde dossier. WG_MAX is hier 2, en de
            # wet zelf telt mee; de eerder opgehaalde vielen eruit.
            w = vraag("GET", f"/wetsgeschiedenis/gegevens/{bwb}.json")[1]
            self.assertEqual(w["dossiers_er"], ["35968", "20202"])
        finally:
            self.herstel(oud)

    def test_wat_is_opgehaald_blijft_onder_het_maximum(self):
        for nr in ("10001", "10002", "10003"):
            dienst.bewaar({"dossier": nr, "naam": nr, "soort": "x"})
        eigen = [d["dossier"] for d in dienst.index() if d.get("opgehaald")]
        self.assertEqual(eigen, ["10002", "10003"])
        self.assertFalse((dienst.OPGEHAALD / "10001.json").exists())

    def test_elk_verzoek_staat_in_het_logboek_maar_het_pollen_niet(self):
        vraag("GET", "/wetsgeschiedenis/gegevens/dossiers.json")
        vraag("GET", "/wetsgeschiedenis/api/taak")
        status, log = vraag("GET", "/wetsgeschiedenis/api/log")
        self.assertEqual(status, 200)
        verzoeken = [r for r in log if r["soort"] == "verzoek"]
        self.assertEqual(verzoeken[-1]["pad"], "/wetsgeschiedenis/gegevens/dossiers.json")
        self.assertEqual(verzoeken[-1]["status"], 200)
        self.assertNotIn("/wetsgeschiedenis/api/taak", [r["pad"] for r in verzoeken])

    def test_een_curlfout_zegt_welke_host_en_waarom(self):
        def dicht(cmd, *a, **k):  # zoals een cluster zonder uitgaand verkeer
            self.assertEqual(cmd[:4], ["curl", "-S", "--connect-timeout", dienst.VERBINDEN])
            raise dienst.subprocess.CalledProcessError(28, cmd, b"", b"curl: (28) Connection timed out after 15001 ms")

        oud = dienst._run
        dienst._run = dicht
        try:
            status, x = vraag("GET", "/wetsgeschiedenis/api/regeling/BWBR0029404")
        finally:
            dienst._run = oud
        self.assertEqual(status, 502)
        self.assertIn("wetten.overheid.nl: geen antwoord binnen de tijd", x["fout"])
        self.assertIn("curl 28", x["fout"])
        uit = [r for r in dienst.LOGBOEK if r["soort"] == "uit"][-1]
        self.assertEqual((uit["host"], uit["code"]), ("wetten.overheid.nl", 28))

    def test_de_diagnose_noemt_de_host_die_niet_bereikbaar_is(self):
        def nep(cmd, *a, **k):
            if cmd[1] == "--version":
                return dienst.subprocess.CompletedProcess(cmd, 0, "curl 8.0.0 (test)\n", "")
            if cmd[-1] == "https://gegevensmagazijn.tweedekamer.nl/":
                return dienst.subprocess.CompletedProcess(cmd, 6, "000  0 0 0", "curl: (6) Could not resolve host")
            return dienst.subprocess.CompletedProcess(cmd, 0, "200 192.0.2.1 0.01 0.02 0.10", "")

        oud = dienst._run
        dienst._run = nep
        try:
            status, d = vraag("GET", "/wetsgeschiedenis/api/diagnose")
        finally:
            dienst._run = oud
        self.assertEqual(status, 200)
        self.assertTrue(d["data"]["schrijfbaar"])
        self.assertTrue(d["hosts"]["wetten.overheid.nl"]["ok"])
        self.assertFalse(d["hosts"]["gegevensmagazijn.tweedekamer.nl"]["ok"])
        self.assertIn("DNS", d["hosts"]["gegevensmagazijn.tweedekamer.nl"]["uitleg"])
        self.assertEqual(d["oordeel"], "ophalen faalt: gegevensmagazijn.tweedekamer.nl niet bereikbaar")
        self.assertEqual(vraag("GET", "/wetsgeschiedenis/api/diagnose?bwb=x")[0], 400)

    def test_een_map_zonder_schrijfrecht_wordt_herkend(self):
        if os.getuid() == 0:
            self.skipTest("root schrijft overal")
        dicht = Path(TMP) / "dicht"
        dicht.mkdir()
        dicht.chmod(0o555)
        try:
            self.assertFalse(dienst._schrijfbaar(dicht))
            self.assertTrue(dienst._schrijfbaar(Path(TMP) / "open"))
        finally:
            dicht.chmod(0o755)


if __name__ == "__main__":
    unittest.main()
