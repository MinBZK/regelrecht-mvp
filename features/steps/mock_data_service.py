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
        self.rvig_personal_data = {}
        self.rvig_relationship_data = {}
        self.rvz_insurance_data = {}
        self.belastingdienst_box1_data = {}
        self.belastingdienst_box2_data = {}
        self.belastingdienst_box3_data = {}

    def store_rvig_personal_data(self, data: dict):
        """Store RvIG personal data by BSN"""
        bsn = data["bsn"]
        self.rvig_personal_data[bsn] = data

    def store_rvig_relationship_data(self, data: dict):
        """Store RvIG relationship data by BSN"""
        bsn = data["bsn"]
        self.rvig_relationship_data[bsn] = data

    def store_rvz_insurance_data(self, data: dict):
        """Store RVZ insurance data by BSN"""
        bsn = data["bsn"]
        self.rvz_insurance_data[bsn] = data

    def store_belastingdienst_box1_data(self, data: dict):
        """Store Belastingdienst box 1 data by BSN"""
        bsn = data["bsn"]
        self.belastingdienst_box1_data[bsn] = data

    def store_belastingdienst_box2_data(self, data: dict):
        """Store Belastingdienst box 2 data by BSN"""
        bsn = data["bsn"]
        self.belastingdienst_box2_data[bsn] = data

    def store_belastingdienst_box3_data(self, data: dict):
        """Store Belastingdienst box 3 data by BSN"""
        bsn = data["bsn"]
        self.belastingdienst_box3_data[bsn] = data

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
            if bsn in self.rvig_personal_data:
                data = self.rvig_personal_data[bsn]
                # Calculate age from birth date
                birth_date = datetime.strptime(data["geboortedatum"], "%Y-%m-%d").date()
                today = datetime.now().date()
                age = (
                    today.year
                    - birth_date.year
                    - ((today.month, today.day) < (birth_date.month, birth_date.day))
                )
                outputs["leeftijd"] = age

        # ZVW (Zorgverzekeringswet) - Health insurance
        elif "zvw" in uri:
            if bsn in self.rvz_insurance_data:
                data = self.rvz_insurance_data[bsn]
                outputs["is_verzekerd"] = data["polis_status"] == "ACTIEF"

        # AWIR/Toeslagpartner - Relationship and income data
        elif "awir" in uri or "toeslagpartner" in uri:
            if "toeslagpartner" in uri or field == "heeft_toeslagpartner":
                if bsn in self.rvig_relationship_data:
                    data = self.rvig_relationship_data[bsn]
                    outputs["heeft_toeslagpartner"] = (
                        data["partnerschap_type"] != "GEEN"
                    )
            if "toetsingsinkomen" in uri or field == "toetsingsinkomen":
                # Calculate total income from box 1 and box 2
                box1 = self.belastingdienst_box1_data.get(bsn, {})
                box2 = self.belastingdienst_box2_data.get(bsn, {})

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

                outputs["toetsingsinkomen"] = box1_total + box2_total

        # Belastingdienst/Inkomstenbelasting - Assets (rendementsgrondslag)
        elif (
            "belastingdienst" in uri or "inkomstenbelasting" in uri
        ) and "rendementsgrondslag" in uri:
            if bsn in self.belastingdienst_box3_data:
                data = self.belastingdienst_box3_data[bsn]
                # Calculate total assets (already in eurocent)
                total_assets = (
                    int(data.get("spaargeld", 0))
                    + int(data.get("beleggingen", 0))
                    + int(data.get("onroerend_goed", 0))
                    - int(data.get("schulden", 0))
                )
                outputs["rendementsgrondslag"] = total_assets

        # Return result
        return ArticleResult(
            article_number="mock",
            law_id="mock",
            law_uuid="mock-uuid",
            output=outputs,
            input={},
            path=None,
        )
