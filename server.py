"""
Simpele HTTP server voor de RegelRecht browser interface.
Serveert zowel statische bestanden als een JSON API voor verordeningen.
"""

import json
import os
import re
import sys
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from urllib.parse import urlparse

import yaml

# Add project root to path for imports
sys.path.insert(0, str(Path(__file__).parent))

from engine.bwb_resolver import BWBResolver

# Paths - configurable via environment variables
PROJECT_ROOT = Path(__file__).parent
REGULATION_DIR = Path(os.environ.get("REGELRECHT_REGULATION_DIR", PROJECT_ROOT / "regulation" / "nl"))
FRONTEND_DIR = Path(os.environ.get("REGELRECHT_FRONTEND_DIR", PROJECT_ROOT / "frontend"))
ANNOTATIONS_DIR = Path(os.environ.get("REGELRECHT_ANNOTATIONS_DIR", PROJECT_ROOT / "annotations"))

# Server configuration
SERVER_PORT = int(os.environ.get("REGELRECHT_PORT", "8000"))

# Security constants
MAX_BODY_SIZE = 10 * 1024  # 10KB max request body

# W3C Web Annotation vocabulary - valid motivations (RFC-005)
# See: https://www.w3.org/TR/annotation-model/#motivation-and-purpose
VALID_MOTIVATIONS = {
    "assessing",      # Quality assessment
    "bookmarking",    # Bookmark for later
    "classifying",    # Formal classification
    "commenting",     # Human explanation or note
    "describing",     # Metadata description
    "editing",        # Request or suggest edit
    "highlighting",   # Visual emphasis
    "identifying",    # Identify the target
    "linking",        # Link to another resource
    "moderating",     # Moderation action
    "questioning",    # Open question
    "replying",       # Reply to another annotation
    "tagging",        # Classification tag
}

# RFC-005: Resolution status - whether the selector found the text
VALID_RESOLUTIONS = {"found", "orphaned"}

# RFC-005: Workflow status - for questioning/reviewing motivations
VALID_WORKFLOWS = {"open", "resolved"}

# Valid classification values (regelrecht extension)
VALID_CLASSIFICATIONS = {"definition", "input", "output", "logic", "open_norm", "parameter"}

# Valid data types (regelrecht extension)
VALID_DATA_TYPES = {"boolean", "integer", "number", "string", "date", "money", "amount"}


def validate_article_nr(article_nr: str) -> bool:
    """Valideer artikel nummer format."""
    if not article_nr:
        return False
    # Allow alphanumeric with optional suffix (e.g., "3", "10a", "2bis")
    return bool(re.match(r"^[0-9a-z]{1,10}$", article_nr.lower()))


def validate_regulation_id(reg_id: str) -> bool:
    """Valideer regulation ID format."""
    if not reg_id:
        return False
    # Allow lowercase letters, numbers, and underscores
    return bool(re.match(r"^[a-z0-9_]{1,100}$", reg_id))


def validate_motivation(motivation: str) -> bool:
    """Valideer W3C motivation vocabulary."""
    return motivation in VALID_MOTIVATIONS


def get_safe_path(base_dir: Path, relative_path: str) -> Path:
    """Veilig pad resolutie - voorkom path traversal."""
    # Resolve both paths to absolute
    full_path = (base_dir / relative_path).resolve()
    base_resolved = base_dir.resolve()
    # Check that the result is still within base_dir
    if not str(full_path).startswith(str(base_resolved)):
        raise ValueError("Invalid path: path traversal detected")
    return full_path


def validate_annotation(annotation: dict) -> tuple[bool, str | None]:
    """Valideer W3C annotation structuur (RFC-005 compliant)."""
    # Check required top-level fields
    required = ["type", "motivation", "target", "body"]
    for field in required:
        if field not in annotation:
            return False, f"Missing required field: {field}"

    # Validate type
    if annotation["type"] != "Annotation":
        return False, "Invalid type: must be 'Annotation'"

    # Validate motivation (RFC-005: required, W3C vocabulary)
    if not validate_motivation(annotation["motivation"]):
        return False, f"Invalid motivation: {annotation['motivation']}"

    # Validate resolution if present (RFC-005)
    if "resolution" in annotation:
        if annotation["resolution"] not in VALID_RESOLUTIONS:
            return False, f"Invalid resolution: {annotation['resolution']}"

    # Validate workflow if present (RFC-005)
    if "workflow" in annotation:
        if annotation["workflow"] not in VALID_WORKFLOWS:
            return False, f"Invalid workflow: {annotation['workflow']}"

    # Validate target
    target = annotation.get("target", {})
    if not target.get("source"):
        return False, "Target missing source"

    # Validate selector if present (RFC-005: TextQuoteSelector)
    selector = target.get("selector", {})
    if selector:
        if selector.get("type") != "TextQuoteSelector":
            return False, "Invalid selector type: must be 'TextQuoteSelector'"
        if not selector.get("exact"):
            return False, "Selector missing exact text"
        # Validate exact text length
        if len(selector.get("exact", "")) > 500:
            return False, "Selector exact text too long (max 500 chars)"

    # Validate body
    body = annotation.get("body", {})
    if not body.get("type"):
        return False, "Body missing type"

    # Validate body type (RFC-005: TextualBody or SpecificResource)
    if body["type"] not in {"TextualBody", "SpecificResource"}:
        return False, f"Invalid body type: {body['type']} (must be TextualBody or SpecificResource)"

    # Validate body.purpose if present (RFC-005: same W3C vocabulary as motivation)
    if body.get("purpose"):
        if body["purpose"] not in VALID_MOTIVATIONS:
            return False, f"Invalid body purpose: {body['purpose']}"

    # Optional: validate classification if present (regelrecht extension)
    if body.get("classification"):
        if body["classification"] not in VALID_CLASSIFICATIONS:
            return False, f"Invalid classification: {body['classification']}"

    # Optional: validate data_type if present (regelrecht extension)
    if body.get("data_type"):
        if body["data_type"] not in VALID_DATA_TYPES:
            return False, f"Invalid data_type: {body['data_type']}"

    return True, None


def load_all_regulations() -> dict:
    """Laad alle verordeningen uit de regulation folder."""
    regulations = {}

    for yaml_file in REGULATION_DIR.rglob("*.yaml"):
        try:
            with open(yaml_file, encoding="utf-8") as f:
                data = yaml.safe_load(f)
                if data and "$id" in data:
                    reg_id = data["$id"]
                    regulations[reg_id] = {
                        "path": str(yaml_file.relative_to(REGULATION_DIR)),
                        "data": data,
                    }
        except Exception as e:
            print(f"Fout bij laden van {yaml_file}: {e}")

    return regulations


def extract_relations(regulations: dict) -> dict:
    """Extract relaties tussen verordeningen (welke refereren naar welke)."""
    relations = {}

    for reg_id, reg_info in regulations.items():
        data = reg_info["data"]
        relations[reg_id] = {
            "depends_on": set(),  # Wetten waar deze van afhankelijk is
            "used_by": set(),  # Wetten die deze gebruiken
        }

        # Zoek in alle artikelen naar sources/regulation referenties
        for article in data.get("articles", []):
            machine_readable = article.get("machine_readable", {})
            execution = machine_readable.get("execution", {})

            # Check input sources
            for input_item in execution.get("input", []):
                source = input_item.get("source", {})
                if "regulation" in source:
                    ref_reg = source["regulation"]
                    relations[reg_id]["depends_on"].add(ref_reg)

            # Check legal_basis
            for basis in data.get("legal_basis", []):
                if "law" in basis:
                    relations[reg_id]["depends_on"].add(basis["law"])

    # Nu de omgekeerde relatie vullen (used_by)
    for reg_id, rels in relations.items():
        for dep in rels["depends_on"]:
            if dep in relations:
                relations[dep]["used_by"].add(reg_id)

    # Convert sets to lists for JSON serialization
    for reg_id in relations:
        relations[reg_id]["depends_on"] = list(relations[reg_id]["depends_on"])
        relations[reg_id]["used_by"] = list(relations[reg_id]["used_by"])

    return relations


def format_regulation_list(regulations: dict) -> list:
    """Formatteer lijst van verordeningen voor de UI."""
    result = []
    for reg_id, reg_info in regulations.items():
        data = reg_info["data"]
        result.append(
            {
                "id": reg_id,
                "name": data.get("name") or reg_id.replace("_", " ").title(),
                "regulatory_layer": data.get("regulatory_layer", "ONBEKEND"),
                "publication_date": data.get("publication_date"),
                "valid_from": data.get("valid_from"),
                "bwb_id": data.get("bwb_id"),
                "url": data.get("url"),
                "article_count": len(data.get("articles", [])),
            }
        )
    return sorted(result, key=lambda x: x["name"].lower())


def format_article(article: dict) -> dict:
    """Formatteer een artikel voor de UI."""
    machine_readable = article.get("machine_readable", {})
    execution = machine_readable.get("execution", {})

    # Extract input sources with their regulation references
    inputs = []
    for inp in execution.get("input", []):
        input_info = {
            "name": inp.get("name"),
            "type": inp.get("type"),
            "description": inp.get("description"),
        }
        source = inp.get("source", {})
        if source:
            input_info["source"] = {
                "regulation": source.get("regulation"),
                "output": source.get("output"),
                "human_input": source.get("human_input"),
                "description": source.get("description"),
                "parameters": source.get("parameters"),
            }
        inputs.append(input_info)

    return {
        "number": article.get("number"),
        "text": article.get("text"),
        "url": article.get("url"),
        "definitions": machine_readable.get("definitions", {}),
        "parameters": execution.get("parameters", []),
        "input": inputs,
        "output": execution.get("output", []),
        "actions": execution.get("actions", []),
        "produces": execution.get("produces", {}),
        # Voeg open normen en human assessment info toe
        "machine_readable": {
            "open_norms": machine_readable.get("open_norms", []),
            "requires_human_assessment": machine_readable.get(
                "requires_human_assessment", False
            ),
            "human_assessment_reason": machine_readable.get("human_assessment_reason"),
        },
    }


def format_article_with_annotations(article: dict, annotations: list[dict]) -> dict:
    """Formatteer een artikel met W3C annotaties voor de UI."""
    # Get base article format
    result = format_article(article)

    # Filter annotations for this article
    article_nr = str(article.get("number", ""))
    article_annotations = [
        ann
        for ann in annotations
        if str(ann.get("target", {}).get("article", "")) == article_nr
    ]

    # Add annotations to result
    result["annotations"] = article_annotations

    return result


class RegulationAPIHandler(SimpleHTTPRequestHandler):
    """HTTP handler die zowel statische bestanden als API endpoints serveert."""

    regulations = None
    relations = None
    bwb_resolver = None

    def __init__(self, *args, **kwargs):
        # Laad regulations eenmalig
        if RegulationAPIHandler.regulations is None:
            print("Laden van verordeningen...")
            RegulationAPIHandler.regulations = load_all_regulations()
            RegulationAPIHandler.relations = extract_relations(
                RegulationAPIHandler.regulations
            )
            RegulationAPIHandler.bwb_resolver = BWBResolver(
                RegulationAPIHandler.regulations
            )
            print(f"Geladen: {len(RegulationAPIHandler.regulations)} verordeningen")
            print(f"BWB mappings: {len(RegulationAPIHandler.bwb_resolver)}")

        # Set de directory voor statische bestanden
        super().__init__(*args, directory=str(FRONTEND_DIR), **kwargs)

    def do_OPTIONS(self):
        """Handle CORS preflight."""
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    def do_POST(self):
        """Handle POST requests for saving annotations."""
        # Security: Check request size limit
        content_length = int(self.headers.get("Content-Length", 0))
        if content_length > MAX_BODY_SIZE:
            self.send_error(413, "Payload too large")
            return

        parsed = urlparse(self.path)
        path = parsed.path

        # POST /api/annotation/{reg_id}/{idx}/status - Change annotation status
        if "/api/annotation/" in path and "/status" in path:
            parts = path.split("/")
            try:
                reg_id = parts[3]
                ann_idx = int(parts[4])
                if not validate_regulation_id(reg_id):
                    self.send_error(400, "Invalid regulation ID format")
                    return
                self.handle_change_annotation_status(reg_id, ann_idx, content_length)
            except (IndexError, ValueError) as e:
                self.send_error(400, f"Invalid path: {e}")

        # POST /api/sync/{reg_id}/execute - Execute sync
        elif path.startswith("/api/sync/") and path.endswith("/execute"):
            parts = path.split("/")
            try:
                reg_id = parts[3]
                if not validate_regulation_id(reg_id):
                    self.send_error(400, "Invalid regulation ID format")
                    return
                self.handle_sync_execute(reg_id)
            except (IndexError, ValueError) as e:
                self.send_error(400, f"Invalid path: {e}")

        # POST /api/regulation/{id}/annotation - W3C annotation endpoint
        elif path.endswith("/annotation"):
            parts = path.split("/")
            # /api/regulation/{id}/annotation
            try:
                reg_id = parts[3]
                if not validate_regulation_id(reg_id):
                    self.send_error(400, "Invalid regulation ID format")
                    return
                self.handle_add_annotation(reg_id, content_length)
            except (IndexError, ValueError) as e:
                self.send_error(400, f"Invalid path: {e}")

        # POST /api/regulation/{id}/article/{nr}/open_norm (legacy)
        elif "/open_norm" in path:
            parts = path.split("/")
            # /api/regulation/{id}/article/{nr}/open_norm
            try:
                reg_id = parts[3]
                article_nr = parts[5]
                if not validate_regulation_id(reg_id):
                    self.send_error(400, "Invalid regulation ID format")
                    return
                if not validate_article_nr(article_nr):
                    self.send_error(400, "Invalid article number format")
                    return
                self.handle_add_open_norm(reg_id, article_nr, content_length)
            except (IndexError, ValueError) as e:
                self.send_error(400, f"Invalid path: {e}")
        else:
            self.send_error(404, "Endpoint not found")

    def handle_change_annotation_status(
        self, reg_id: str, ann_idx: int, content_length: int
    ):
        """Change the status of an annotation."""
        # Read request body
        body = self.rfile.read(content_length).decode("utf-8")
        try:
            data = json.loads(body)
        except json.JSONDecodeError as e:
            self.send_error(400, f"Invalid JSON: {e}")
            return

        new_status = data.get("status")
        if new_status not in {"draft", "approved", "rejected", "promoted"}:
            self.send_error(
                400, "Invalid status (must be draft, approved, rejected, or promoted)"
            )
            return

        # Load annotations
        try:
            annotations_file = get_safe_path(ANNOTATIONS_DIR, f"{reg_id}.yaml")
        except ValueError as e:
            self.send_error(400, str(e))
            return

        if not annotations_file.exists():
            self.send_error(404, "Annotations file not found")
            return

        try:
            with open(annotations_file, encoding="utf-8") as f:
                ann_data = yaml.safe_load(f) or {}
        except Exception as e:
            self.send_error(500, f"Failed to load annotations: {e}")
            return

        annotations = ann_data.get("annotations", [])
        if ann_idx < 0 or ann_idx >= len(annotations):
            self.send_error(404, f"Annotation index {ann_idx} not found")
            return

        # Update status
        annotations[ann_idx]["status"] = new_status

        # Save back
        try:
            with open(annotations_file, "w", encoding="utf-8") as f:
                yaml.dump(
                    ann_data,
                    f,
                    allow_unicode=True,
                    sort_keys=False,
                    default_flow_style=False,
                )
            self.send_json_response(
                {
                    "status": "ok",
                    "annotation_status": new_status,
                    "index": ann_idx,
                }
            )
        except Exception as e:
            self.send_error(500, f"Failed to save: {e}")

    def handle_sync_execute(self, reg_id: str):
        """Execute sync for a regulation."""
        try:
            from script.sync_annotations import execute_sync

            result = execute_sync(reg_id)
            self.send_json_response(
                {
                    "status": "ok" if result.synced_count > 0 else "no_changes",
                    "regulation_id": result.regulation_id,
                    "synced_count": result.synced_count,
                    "errors": result.errors,
                    "target_file": str(result.target_file)
                    if result.target_file
                    else None,
                }
            )
        except Exception as e:
            self.send_error(500, f"Sync execute failed: {e}")

    def handle_add_annotation(self, reg_id: str, content_length: int):
        """Voeg een W3C annotation toe en sla op in annotations folder."""
        # Lees request body
        body = self.rfile.read(content_length).decode("utf-8")
        try:
            annotation = json.loads(body)
        except json.JSONDecodeError as e:
            self.send_error(400, f"Invalid JSON: {e}")
            return

        # Validate annotation structure
        valid, error = validate_annotation(annotation)
        if not valid:
            self.send_error(400, f"Invalid annotation: {error}")
            return

        # Ensure annotations directory exists
        ANNOTATIONS_DIR.mkdir(exist_ok=True)

        # Load or create annotations file for this regulation
        annotations_file = ANNOTATIONS_DIR / f"{reg_id}.yaml"
        try:
            annotations_file = get_safe_path(ANNOTATIONS_DIR, f"{reg_id}.yaml")
        except ValueError as e:
            self.send_error(400, str(e))
            return

        if annotations_file.exists():
            with open(annotations_file, encoding="utf-8") as f:
                data = yaml.safe_load(f) or {}
        else:
            data = {}

        if "annotations" not in data:
            data["annotations"] = []

        # Check for duplicate (same selector exact text in same article)
        target = annotation.get("target", {})
        selector = target.get("selector", {})
        exact_text = selector.get("exact", "")
        article = target.get("article", "")

        for existing in data["annotations"]:
            ex_target = existing.get("target", {})
            ex_selector = ex_target.get("selector", {})
            if (
                ex_selector.get("exact") == exact_text
                and ex_target.get("article") == article
            ):
                self.send_json_response(
                    {"status": "exists", "message": "Annotation already exists"}
                )
                return

        # Add default status if not present
        if "status" not in annotation:
            annotation["status"] = "draft"

        # Add the annotation
        data["annotations"].append(annotation)

        # Write back to file
        try:
            with open(annotations_file, "w", encoding="utf-8") as f:
                yaml.dump(
                    data,
                    f,
                    allow_unicode=True,
                    sort_keys=False,
                    default_flow_style=False,
                )
            print(f"Saved W3C annotation to {annotations_file}")
            self.send_json_response({"status": "ok", "annotation": annotation})
        except Exception as e:
            self.send_error(500, f"Failed to save: {e}")

    def handle_get_annotations(self, reg_id: str):
        """Haal alle W3C annotaties op voor een regulation."""
        try:
            annotations_file = get_safe_path(ANNOTATIONS_DIR, f"{reg_id}.yaml")
        except ValueError as e:
            self.send_error(400, str(e))
            return

        if not annotations_file.exists():
            self.send_json_response({"annotations": []})
            return

        try:
            with open(annotations_file, encoding="utf-8") as f:
                data = yaml.safe_load(f) or {}
            self.send_json_response({"annotations": data.get("annotations", [])})
        except Exception as e:
            self.send_error(500, f"Failed to load annotations: {e}")

    def handle_add_open_norm(self, reg_id: str, article_nr: str, content_length: int):
        """Voeg een open norm toe aan een artikel en sla op in YAML."""
        # Lees request body
        body = self.rfile.read(content_length).decode("utf-8")
        try:
            data = json.loads(body)
        except json.JSONDecodeError as e:
            self.send_error(400, f"Invalid JSON: {e}")
            return

        term = data.get("term")
        description = data.get("description")

        if not term:
            self.send_error(400, "term is required")
            return

        # Validate term format (prevent injection)
        if not re.match(r"^[a-z0-9_\s]{1,100}$", term.lower()):
            self.send_error(400, "Invalid term format")
            return

        # Vind de regulation
        if reg_id not in RegulationAPIHandler.regulations:
            self.send_error(404, f"Regulation '{reg_id}' not found")
            return

        reg_info = RegulationAPIHandler.regulations[reg_id]
        reg_data = reg_info["data"]

        # Safe path resolution
        try:
            yaml_path = get_safe_path(REGULATION_DIR, reg_info["path"])
        except ValueError as e:
            self.send_error(400, str(e))
            return

        # Vind het artikel
        article = None
        for art in reg_data.get("articles", []):
            if str(art.get("number")) == str(article_nr):
                article = art
                break

        if not article:
            self.send_error(404, f"Article '{article_nr}' not found")
            return

        # Voeg open norm toe
        if "machine_readable" not in article:
            article["machine_readable"] = {}
        if "open_norms" not in article["machine_readable"]:
            article["machine_readable"]["open_norms"] = []

        # Check of term al bestaat
        existing = [
            n
            for n in article["machine_readable"]["open_norms"]
            if n.get("term") == term
        ]
        if existing:
            self.send_json_response(
                {"status": "exists", "message": "Term already exists"}
            )
            return

        # Voeg nieuwe norm toe
        article["machine_readable"]["open_norms"].append(
            {
                "term": term,
                "description": description or f"Open norm: {term}",
            }
        )

        # Schrijf YAML terug
        try:
            with open(yaml_path, "w", encoding="utf-8") as f:
                yaml.dump(
                    reg_data,
                    f,
                    allow_unicode=True,
                    sort_keys=False,
                    default_flow_style=False,
                )
            print(f"Saved open norm '{term}' to {yaml_path}")
            self.send_json_response({"status": "ok", "term": term})
        except Exception as e:
            self.send_error(500, f"Failed to save: {e}")

    def do_GET(self):
        """Handle GET requests."""
        parsed = urlparse(self.path)
        path = parsed.path

        # API endpoints
        if path == "/api/regulations":
            self.send_json_response(
                format_regulation_list(RegulationAPIHandler.regulations)
            )
        elif path == "/api/relations":
            self.send_json_response(RegulationAPIHandler.relations)
        elif path == "/api/bwb":
            # GET /api/bwb - list all BWB mappings
            self.send_json_response(RegulationAPIHandler.bwb_resolver.to_dict())
        elif path.startswith("/api/bwb/"):
            # GET /api/bwb/{bwb_id} - lookup by BWB ID
            bwb_id = path.replace("/api/bwb/", "").strip("/")
            self.handle_bwb_lookup(bwb_id)
        elif path.startswith("/api/sync/") and path.endswith("/preview"):
            # GET /api/sync/{reg_id}/preview - preview sync changes
            parts = path.split("/")
            try:
                reg_id = parts[3]
                if not validate_regulation_id(reg_id):
                    self.send_error(400, "Invalid regulation ID format")
                    return
                self.handle_sync_preview(reg_id)
            except (IndexError, ValueError) as e:
                self.send_error(400, f"Invalid path: {e}")
        elif path.endswith("/annotations"):
            # GET /api/regulation/{id}/annotations - W3C annotations for regulation
            parts = path.split("/")
            try:
                reg_id = parts[3]
                if not validate_regulation_id(reg_id):
                    self.send_error(400, "Invalid regulation ID format")
                    return
                self.handle_get_annotations(reg_id)
            except (IndexError, ValueError) as e:
                self.send_error(400, f"Invalid path: {e}")
        elif path.startswith("/api/regulation/"):
            reg_id = path.replace("/api/regulation/", "").strip("/")
            if reg_id in RegulationAPIHandler.regulations:
                data = RegulationAPIHandler.regulations[reg_id]["data"]
                # Load annotations for this regulation
                annotations = self._load_annotations_for_regulation(reg_id)
                # Format articles with annotations
                formatted_articles = [
                    format_article_with_annotations(art, annotations)
                    for art in data.get("articles", [])
                ]
                response = {
                    "id": data.get("$id"),
                    "name": data.get("name") or reg_id.replace("_", " ").title(),
                    "regulatory_layer": data.get("regulatory_layer"),
                    "publication_date": data.get("publication_date"),
                    "valid_from": data.get("valid_from"),
                    "bwb_id": data.get("bwb_id"),
                    "url": data.get("url"),
                    "articles": formatted_articles,
                    "relations": RegulationAPIHandler.relations.get(reg_id, {}),
                    "raw_yaml": yaml.dump(data, allow_unicode=True, sort_keys=False),
                }
                self.send_json_response(response)
            else:
                self.send_error(404, f"Verordening '{reg_id}' niet gevonden")
        else:
            # Statische bestanden
            super().do_GET()

    def handle_bwb_lookup(self, bwb_id: str):
        """Handle BWB ID lookup."""
        # Validate BWB ID format
        if not re.match(r"^BWBR[0-9]{7}$", bwb_id):
            self.send_error(
                400, "Invalid BWB ID format (expected BWBR followed by 7 digits)"
            )
            return

        mapping = RegulationAPIHandler.bwb_resolver.get_mapping(bwb_id)
        if mapping:
            self.send_json_response(
                {
                    "bwb_id": mapping.bwb_id,
                    "law_id": mapping.law_id,
                    "name": mapping.name,
                    "url": mapping.url,
                }
            )
        else:
            self.send_error(404, f"BWB ID '{bwb_id}' not found in index")

    def handle_sync_preview(self, reg_id: str):
        """Handle sync preview request."""
        try:
            from script.sync_annotations import preview_sync

            preview = preview_sync(reg_id)
            self.send_json_response(
                {
                    "regulation_id": preview.regulation_id,
                    "annotations_count": len(preview.annotations_to_sync),
                    "conversions": [
                        {
                            "success": c.success,
                            "target_path": c.target_path,
                            "schema_data": c.schema_data,
                            "error": c.error,
                        }
                        for c in preview.conversions
                    ],
                    "target_file": str(preview.target_file)
                    if preview.target_file
                    else None,
                    "errors": preview.errors,
                }
            )
        except Exception as e:
            self.send_error(500, f"Sync preview failed: {e}")

    def _load_annotations_for_regulation(self, reg_id: str) -> list[dict]:
        """Load annotations for a regulation from YAML file."""
        try:
            annotations_file = get_safe_path(ANNOTATIONS_DIR, f"{reg_id}.yaml")
            if annotations_file.exists():
                with open(annotations_file, encoding="utf-8") as f:
                    data = yaml.safe_load(f) or {}
                return data.get("annotations", [])
        except Exception:
            pass
        return []

    def send_json_response(self, data):
        """Send JSON response."""
        self.send_response(200)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(json.dumps(data, ensure_ascii=False, indent=2).encode("utf-8"))


def main():
    port = SERVER_PORT
    server = HTTPServer(("", port), RegulationAPIHandler)
    print(f"RegelRecht Browser server gestart op http://localhost:{port}")
    print(f"Configuratie:")
    print(f"  REGELRECHT_REGULATION_DIR: {REGULATION_DIR}")
    print(f"  REGELRECHT_FRONTEND_DIR: {FRONTEND_DIR}")
    print(f"  REGELRECHT_ANNOTATIONS_DIR: {ANNOTATIONS_DIR}")
    print(f"  REGELRECHT_PORT: {port}")
    print("Druk Ctrl+C om te stoppen")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nServer gestopt")


if __name__ == "__main__":
    main()
