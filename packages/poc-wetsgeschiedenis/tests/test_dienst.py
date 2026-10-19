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


def vraag(methode, pad, body=None):
    a, b = socket.socketpair()
    data = json.dumps(body).encode() if body is not None else b""
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

    def test_een_opdracht_tegelijk_en_haar_uitkomst_komt_in_de_lijst(self):
        poort = threading.Event()

        def nep(args):  # in plaats van het internet: wacht, en zet dan een dossier neer
            poort.wait(5)
            print("stukken opgehaald")

        oud_main, oud_tijdlijn = dienst.kamerdossier.main, dienst.bouw.tijdlijn
        dienst.kamerdossier.main = nep
        dienst.bouw.tijdlijn = lambda nr: ({"dossier": nr, "naam": f"Dossier {nr}", "soort": "wijzigingswet"}, None)
        try:
            status, t = vraag("POST", "/wetsgeschiedenis/api/haal", {"bwb": "BWBR0007376", "dossier": "36889"})
            self.assertEqual((status, t["status"]), (202, "bezig"))
            status, x = vraag("POST", "/wetsgeschiedenis/api/haal", {"bwb": "BWBR0007376", "dossier": "35968"})
            self.assertEqual(status, 409)
            poort.set()
            for _ in range(50):
                t = vraag("GET", "/wetsgeschiedenis/api/taak")[1]
                if t["status"] != "bezig":
                    break
                time.sleep(0.05)
            self.assertEqual((t["status"], t["uitkomst"]), ("klaar", "36889"))
            self.assertIn("stukken opgehaald", t["log"])
            lijst = vraag("GET", "/wetsgeschiedenis/gegevens/dossiers.json")[1]
            self.assertEqual([d["dossier"] for d in lijst], ["35968", "36889"])
            self.assertTrue(lijst[1]["opgehaald"])
        finally:
            dienst.kamerdossier.main, dienst.bouw.tijdlijn = oud_main, oud_tijdlijn

    def test_wat_is_opgehaald_blijft_onder_het_maximum(self):
        for nr in ("10001", "10002", "10003"):
            dienst.bewaar({"dossier": nr, "naam": nr, "soort": "x"})
        eigen = [d["dossier"] for d in dienst.index() if d.get("opgehaald")]
        self.assertEqual(eigen, ["10002", "10003"])
        self.assertFalse((dienst.OPGEHAALD / "10001.json").exists())


if __name__ == "__main__":
    unittest.main()
