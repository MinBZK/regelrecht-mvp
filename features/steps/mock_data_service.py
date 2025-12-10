"""
Mock Data Service for BDD Tests

Provides mock implementations of external law services (RvIG, RVZ, Belastingdienst)
that would normally be called via cross-law references.
"""

from datetime import datetime
from typing import Optional
from engine.engine import ArticleResult


class MockDataService:
    """Mock external data sources for testing"""

    def __init__(self):
        # Service-oriented storage: service -> datasource -> BSN -> data
        self.services = {
            "RVIG": {},  # BRP data sources
            "RVZ": {},  # Healthcare insurance data sources
            "BELASTINGDIENST": {},  # Tax service data sources
        }

    def store_data(self, service: str, datasource: str, data: dict):
        """
        Store data for a service datasource by BSN

        Args:
            service: Service name (e.g., "BELASTINGDIENST", "RVIG")
            datasource: Datasource name (e.g., "box1", "personal_data")
            data: Data dict containing at least "bsn" field
        """
        bsn = data["bsn"]

        if service not in self.services:
            self.services[service] = {}

        if datasource not in self.services[service]:
            self.services[service][datasource] = {}

        self.services[service][datasource][bsn] = data

    def get_mock_result(
        self, uri: str, parameters: dict, field: Optional[str] = None
    ) -> ArticleResult:
        """
        Return mock data based on URI

        Args:
            uri: The URI being requested (e.g., "regulation/nl/wet/wet_brp#leeftijd")
            parameters: Request parameters (including BSN)
            field: Requested output field

        Returns:
            ArticleResult with mocked outputs
        """
        bsn = parameters.get("BSN")
        outputs = {}

        # BRP (Basisregistratie Personen) - Personal data
        if "wet_brp" in uri or "brp" in uri:
            personal_data = self.services.get("RVIG", {}).get("personal_data", {})
            if bsn in personal_data:
                data = personal_data[bsn]
                # Calculate age from birth date
                birth_date = datetime.strptime(data["geboortedatum"], "%Y-%m-%d").date()
                today = datetime.now().date()
                age = (
                    today.year
                    - birth_date.year
                    - ((today.month, today.day) < (birth_date.month, birth_date.day))
                )
                outputs["LEEFTIJD"] = age

        # ZVW (Zorgverzekeringswet) - Health insurance
        elif "zvw" in uri:
            insurance_data = self.services.get("RVZ", {}).get("insurance", {})
            if bsn in insurance_data:
                data = insurance_data[bsn]
                outputs["IS_VERZEKERD"] = data["polis_status"] == "ACTIEF"

        # AWIR/Toeslagpartner - Relationship and income data
        elif "awir" in uri or "toeslagpartner" in uri:
            if "toeslagpartner" in uri or field == "heeft_toeslagpartner":
                relationship_data = self.services.get("RVIG", {}).get(
                    "relationship_data", {}
                )
                if bsn in relationship_data:
                    data = relationship_data[bsn]
                    outputs["HEEFT_TOESLAGPARTNER"] = (
                        data["partnerschap_type"] != "GEEN"
                    )
            if "toetsingsinkomen" in uri or field == "toetsingsinkomen":
                # Calculate total income from box 1 and box 2
                belastingdienst = self.services.get("BELASTINGDIENST", {})
                box1 = belastingdienst.get("box1", {}).get(bsn, {})
                box2 = belastingdienst.get("box2", {}).get(bsn, {})

                # Box 1: Sum all income sources (already in eurocent)
                box1_total = (
                    int(box1.get("loon_uit_dienstbetrekking", 0))
                    + int(box1.get("uitkeringen_en_pensioenen", 0))
                    + int(box1.get("winst_uit_onderneming", 0))
                    + int(box1.get("resultaat_overige_werkzaamheden", 0))
                    + int(box1.get("eigen_woning", 0))
                )

                # Box 2: Sum capital gains (already in eurocent)
                box2_total = int(box2.get("reguliere_voordelen", 0)) + int(
                    box2.get("vervreemdingsvoordelen", 0)
                )

                outputs["TOETSINGSINKOMEN"] = box1_total + box2_total

        # Belastingdienst/Inkomstenbelasting - Assets (rendementsgrondslag)
        elif (
            "belastingdienst" in uri or "inkomstenbelasting" in uri
        ) and "rendementsgrondslag" in uri:
            box3_data = self.services.get("BELASTINGDIENST", {}).get("box3", {})
            if bsn in box3_data:
                data = box3_data[bsn]
                # Calculate total assets (already in eurocent)
                total_assets = (
                    int(data.get("spaargeld", 0))
                    + int(data.get("beleggingen", 0))
                    + int(data.get("onroerend_goed", 0))
                    - int(data.get("schulden", 0))
                )
                outputs["RENDEMENTSGRONDSLAG"] = total_assets

        # Return result
        return ArticleResult(
            article_number="mock",
            law_id="mock",
            output=outputs,
            input={},
            path=None,
        )
