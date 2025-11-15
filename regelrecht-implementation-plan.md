# RegelRecht - Complete Implementatieplan & Architectuur
*Voor implementatie met Claude Code*
*Versie 1.0 - November 2025*

---

## Executive Summary

RegelRecht is een editor voor het omzetten van analoge wetgeving naar machine-uitvoerbare code (NRML). Dit document beschrijft de volledige technische architectuur, implementatiestrategie, en development workflow voor het bouwen van de applicatie.

**Kernfunctionaliteit:**
- Multi-wet editor met tabbed interface en animaties
- YAML/Blockly dual-mode voor NRML authoring
- Bidirectionele sync tussen natuurlijke taal en NRML
- Full-corpus wetgeving zoeken met @-mentions
- Gherkin scenario testing en validatie
- AI-assisted NRML generatie

**Technologie:**
- Frontend: React + TypeScript + Tailwind
- Backend: Python FastAPI
- Database: PostgreSQL + Redis
- Search: Elasticsearch
- LLM: Claude via Anthropic API
- Deployment: Docker + Kubernetes (Nederlandse cloud)

---

## 1. Tech Stack Rationale

### 1.1 Frontend

**React 18+ met TypeScript**
- ✅ Rijke component ecosystem
- ✅ Excellent TypeScript support
- ✅ Good performance met concurrent rendering
- ✅ Team bij MinBZK kent React
- ⚠️ Bundle size management nodig

**Alternatief overwogen:** Svelte (kleiner, sneller) - afgewezen omdat React bekender is bij team

**Key libraries:**
```json
{
  "react": "^18.2.0",
  "typescript": "^5.3.0",
  "vite": "^5.0.0",           // Build tool (sneller dan webpack)
  "react-router-dom": "^6.20.0",
  "zustand": "^4.4.0",         // State management (simpeler dan Redux)
  "tanstack-query": "^5.0.0",  // Server state management
  "tailwindcss": "^3.4.0",     // Styling
  "framer-motion": "^10.16.0", // Animaties (bladder-effect)
  "monaco-editor": "^0.45.0",  // YAML editor met syntax highlighting
  "react-blockly": "^8.0.0",   // Blockly integration
  "yaml": "^2.3.0",            // YAML parsing
  "lucide-react": "^0.300.0",  // Icons
  "cmdk": "^0.2.0",            // Command palette voor @-mentions
  "@tanstack/react-virtual": "^3.0.0" // Virtualization voor lange wetten
}
```

### 1.2 Backend

**Python FastAPI**
- ✅ Async support voor goede performance
- ✅ Automatic OpenAPI docs
- ✅ Type hints met Pydantic
- ✅ Makkelijk te integreren met ML/NLP libraries
- ✅ Team heeft Python expertise

**Alternatief overwogen:** Node.js (uniform met frontend) - afgewezen omdat Python beter is voor data processing en NLP

**Key libraries:**
```python
fastapi==0.109.0
uvicorn[standard]==0.27.0   # ASGI server
sqlalchemy==2.0.25          # ORM
alembic==1.13.0             # Database migrations
pydantic==2.5.0             # Data validation
python-jose==3.3.0          # JWT tokens
passlib[bcrypt]==1.7.4      # Password hashing
anthropic==0.8.0            # Claude API
elasticsearch==8.11.0       # Search
redis==5.0.1                # Caching
aiohttp==3.9.0              # Async HTTP
pytest==7.4.0               # Testing
```

### 1.3 Database

**PostgreSQL 16**
- ✅ JSONB support voor flexible data (NRML metadata)
- ✅ Full-text search (fallback voor Elasticsearch)
- ✅ Mature, reliable
- ✅ Government approved
- ✅ Open source

**Schema design filosofie:**
- Normalized voor structurele data (users, wetten, articles)
- JSONB voor semi-structured data (NRML, mappings, annotations)
- Audit tables voor compliance (wie wijzigde wat wanneer)

**Redis 7**
- Session storage
- Cache voor frequently accessed wetten
- Rate limiting voor LLM calls
- Real-time collaboration pub/sub

### 1.4 Search

**Elasticsearch 8**
- Full-text search over wetgeving corpus
- Multi-field search (titel, artikelnummer, inhoud)
- Fuzzy matching voor typo's
- Highlighting van zoekresultaten
- Aggregaties voor faceted search

**Data pipeline:**
```
wetten.overheid.nl API 
  → Scraper (Python)
  → ETL pipeline
  → Elasticsearch index
  → Update weekly
```

### 1.5 LLM Integration

**Anthropic Claude (via API)**
- Text → NRML generatie
- NRML → Text proposal generatie
- Semantic search improvement
- Code review suggestions

**Rate limiting & costs:**
- Cache common prompts
- Rate limit: 100 req/min per user
- Estimated cost: ~€500/month voor 50 actieve gebruikers

---

## 2. Project Structure

```
regelrecht/
├── frontend/                    # React applicatie
│   ├── public/
│   ├── src/
│   │   ├── components/         # React components
│   │   │   ├── editor/
│   │   │   │   ├── LeftPanel/
│   │   │   │   │   ├── WetTabs.tsx          # Tab navigatie
│   │   │   │   │   ├── ArticleList.tsx      # Artikel lijst
│   │   │   │   │   ├── ArticleEditor.tsx    # Markdown editor
│   │   │   │   │   ├── AtMentionSearch.tsx  # @-mention UI
│   │   │   │   │   └── Annotations.tsx      # Annotaties systeem
│   │   │   │   ├── MiddlePanel/
│   │   │   │   │   ├── NRMLEditor.tsx       # YAML/Blockly container
│   │   │   │   │   ├── YAMLEditor.tsx       # Monaco editor wrapper
│   │   │   │   │   ├── BlocklyEditor.tsx    # Blockly wrapper
│   │   │   │   │   ├── ModeToggle.tsx       # YAML/Blockly switch
│   │   │   │   │   └── SyncIndicator.tsx    # Toont sync status
│   │   │   │   ├── RightPanel/
│   │   │   │   │   ├── ScenarioRunner.tsx   # Gherkin test runner
│   │   │   │   │   ├── ValidationPanel.tsx  # NRML validatie
│   │   │   │   │   ├── CoverageView.tsx     # Coverage analysis
│   │   │   │   │   └── DependencyGraph.tsx  # Wet dependencies
│   │   │   │   ├── PanelResizer.tsx        # Drag-to-resize logic
│   │   │   │   ├── EditorContainer.tsx     # Main layout container
│   │   │   │   └── PageFlipAnimation.tsx   # Bladder animatie
│   │   │   ├── common/
│   │   │   │   ├── Button.tsx
│   │   │   │   ├── Modal.tsx
│   │   │   │   ├── Toast.tsx
│   │   │   │   ├── LoadingSpinner.tsx
│   │   │   │   └── ColorPicker.tsx         # Voor wet kleuren
│   │   │   └── auth/
│   │   │       ├── LoginForm.tsx
│   │   │       └── PrivateRoute.tsx
│   │   ├── hooks/
│   │   │   ├── useWetData.ts              # Fetch wet data
│   │   │   ├── useNRMLSync.ts             # Bidirectional sync logic
│   │   │   ├── useAtMentionSearch.ts      # @ search functionaliteit
│   │   │   ├── useScrollSync.ts           # Scroll synchronisatie
│   │   │   ├── usePanelResize.ts          # Panel resize state
│   │   │   ├── useAIGeneration.ts         # LLM calls
│   │   │   └── useKeyboardShortcuts.ts    # Shortcuts
│   │   ├── stores/
│   │   │   ├── editorStore.ts             # Zustand store voor editor state
│   │   │   ├── wetStore.ts                # Wetten state
│   │   │   ├── authStore.ts               # Auth state
│   │   │   └── uiStore.ts                 # UI preferences
│   │   ├── services/
│   │   │   ├── api.ts                     # API client (axios/fetch wrapper)
│   │   │   ├── wetService.ts              # Wet CRUD operations
│   │   │   ├── nrmlService.ts             # NRML operations
│   │   │   ├── searchService.ts           # Search API calls
│   │   │   ├── llmService.ts              # LLM API calls
│   │   │   └── authService.ts             # Authentication
│   │   ├── types/
│   │   │   ├── wet.ts                     # Wet TypeScript types
│   │   │   ├── nrml.ts                    # NRML types
│   │   │   ├── scenario.ts                # Gherkin scenario types
│   │   │   └── api.ts                     # API response types
│   │   ├── utils/
│   │   │   ├── yaml-parser.ts             # YAML utilities
│   │   │   ├── blockly-converter.ts       # YAML ↔ Blockly
│   │   │   ├── diff.ts                    # Text diffing
│   │   │   ├── colorGenerator.ts          # Wet color generation
│   │   │   └── validators.ts              # Input validation
│   │   ├── constants/
│   │   │   ├── colors.ts                  # Color schemes
│   │   │   ├── shortcuts.ts               # Keyboard shortcuts
│   │   │   └── nrml-templates.ts          # NRML code snippets
│   │   ├── App.tsx
│   │   ├── main.tsx
│   │   └── index.css
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── tailwind.config.js
│
├── backend/                     # Python FastAPI
│   ├── app/
│   │   ├── main.py             # FastAPI app entry point
│   │   ├── config.py           # Configuration (env vars)
│   │   ├── dependencies.py     # Dependency injection
│   │   ├── api/
│   │   │   ├── v1/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── auth.py             # /auth endpoints
│   │   │   │   ├── wetten.py           # /wetten endpoints
│   │   │   │   ├── nrml.py             # /nrml endpoints
│   │   │   │   ├── search.py           # /search endpoints
│   │   │   │   ├── scenarios.py        # /scenarios endpoints
│   │   │   │   ├── llm.py              # /llm endpoints
│   │   │   │   └── sync.py             # /sync endpoints
│   │   │   └── __init__.py
│   │   ├── core/
│   │   │   ├── security.py     # JWT, password hashing
│   │   │   ├── cache.py        # Redis utilities
│   │   │   └── logging.py      # Structured logging
│   │   ├── db/
│   │   │   ├── session.py      # SQLAlchemy session
│   │   │   ├── base.py         # Base model
│   │   │   └── init_db.py      # Database initialization
│   │   ├── models/             # SQLAlchemy models
│   │   │   ├── user.py
│   │   │   ├── wet.py
│   │   │   ├── article.py
│   │   │   ├── nrml.py
│   │   │   ├── annotation.py
│   │   │   ├── scenario.py
│   │   │   └── audit_log.py
│   │   ├── schemas/            # Pydantic schemas (API contracts)
│   │   │   ├── user.py
│   │   │   ├── wet.py
│   │   │   ├── article.py
│   │   │   ├── nrml.py
│   │   │   ├── search.py
│   │   │   └── llm.py
│   │   ├── services/           # Business logic
│   │   │   ├── wet_service.py
│   │   │   ├── nrml_service.py
│   │   │   ├── search_service.py
│   │   │   ├── llm_service.py          # Claude API integration
│   │   │   ├── sync_service.py         # Bidirectional sync
│   │   │   ├── scenario_service.py     # Gherkin runner
│   │   │   └── scraper_service.py      # wetten.nl scraper
│   │   ├── utils/
│   │   │   ├── nrml_parser.py          # NRML YAML parsing
│   │   │   ├── text_differ.py          # Diff algorithm
│   │   │   ├── mapping.py              # Text ↔ NRML mapping
│   │   │   └── validators.py
│   │   └── tests/
│   │       ├── unit/
│   │       ├── integration/
│   │       └── conftest.py
│   ├── alembic/                # Database migrations
│   │   ├── versions/
│   │   └── env.py
│   ├── requirements.txt
│   ├── pyproject.toml
│   └── pytest.ini
│
├── search/                      # Elasticsearch setup
│   ├── mappings/
│   │   └── wetgeving.json      # ES index mapping
│   ├── scripts/
│   │   ├── create_index.py
│   │   ├── scrape_wetten.py    # Scraper voor wetten.overheid.nl
│   │   └── index_documents.py
│   └── config/
│       └── elasticsearch.yml
│
├── docker/
│   ├── Dockerfile.frontend
│   ├── Dockerfile.backend
│   ├── Dockerfile.search
│   └── docker-compose.yml      # Local development
│
├── kubernetes/                  # K8s manifests voor productie
│   ├── namespace.yaml
│   ├── configmap.yaml
│   ├── secrets.yaml
│   ├── deployments/
│   │   ├── frontend.yaml
│   │   ├── backend.yaml
│   │   └── elasticsearch.yaml
│   ├── services/
│   └── ingress.yaml
│
├── docs/
│   ├── API.md                  # API documentatie
│   ├── ARCHITECTURE.md         # Dit document
│   ├── DEVELOPMENT.md          # Development workflow
│   ├── DEPLOYMENT.md           # Deployment guide
│   └── NRML_SPEC.md           # NRML language spec
│
├── scripts/
│   ├── setup.sh               # Initial setup
│   ├── test.sh                # Run all tests
│   └── deploy.sh              # Deployment script
│
├── .github/
│   └── workflows/
│       ├── ci.yml             # CI pipeline
│       └── deploy.yml         # CD pipeline
│
├── .env.example
├── README.md
└── LICENSE
```

---

## 3. Data Models

### 3.1 Database Schema (PostgreSQL)

```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    hashed_password VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    role VARCHAR(50) NOT NULL, -- 'wetgevings_jurist', 'wetsanalist', 'admin'
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Wetten table
CREATE TABLE wetten (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bwb_id VARCHAR(100) UNIQUE,  -- Basis Wetten Bestand ID (van wetten.nl)
    title VARCHAR(500) NOT NULL,
    short_title VARCHAR(200),
    category VARCHAR(100),       -- 'wet', 'amvb', 'ministeriele_regeling'
    color VARCHAR(7) NOT NULL,   -- Hex color code (bijv. #3B82F6)
    status VARCHAR(50) DEFAULT 'draft', -- 'draft', 'review', 'published'
    source_url TEXT,             -- URL naar wetten.overheid.nl
    last_synced_at TIMESTAMP,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB DEFAULT '{}'  -- Extra metadata
);

-- Articles table
CREATE TABLE articles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wet_id UUID REFERENCES wetten(id) ON DELETE CASCADE,
    article_number VARCHAR(50) NOT NULL,  -- bijv. '2.1', '3.1.1'
    title VARCHAR(500),
    content TEXT NOT NULL,       -- Markdown formatted
    nrml_yaml TEXT,              -- NRML YAML code
    nrml_ast JSONB,              -- Parsed NRML voor Blockly
    position INTEGER NOT NULL,   -- Voor sortering
    parent_id UUID REFERENCES articles(id), -- Voor hierarchische structuur
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(wet_id, article_number)
);

-- Mapping tussen text en NRML (voor sync)
CREATE TABLE text_nrml_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID REFERENCES articles(id) ON DELETE CASCADE,
    text_range JSONB NOT NULL,   -- {start: int, end: int} character offsets
    nrml_range JSONB NOT NULL,   -- {start_line: int, end_line: int}
    mapping_type VARCHAR(50),    -- 'direct', 'inferred', 'ai_suggested'
    confidence FLOAT,            -- 0.0 - 1.0
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Annotations (comments, notes)
CREATE TABLE annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID REFERENCES articles(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id),
    content TEXT NOT NULL,
    annotation_type VARCHAR(50), -- 'comment', 'question', 'warning'
    position JSONB,              -- {line: int, column: int} of {start: int, end: int}
    resolved BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Cross-wet references
CREATE TABLE wet_references (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_article_id UUID REFERENCES articles(id) ON DELETE CASCADE,
    target_wet_id UUID REFERENCES wetten(id),
    target_article_id UUID REFERENCES articles(id),
    reference_text TEXT,         -- De @ mention text
    reference_type VARCHAR(50),  -- 'explicit', 'implicit', 'concept'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Gherkin scenarios
CREATE TABLE scenarios (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wet_id UUID REFERENCES wetten(id) ON DELETE CASCADE,
    name VARCHAR(500) NOT NULL,
    gherkin_text TEXT NOT NULL,
    status VARCHAR(50),          -- 'passed', 'failed', 'pending'
    last_run_at TIMESTAMP,
    last_run_result JSONB,       -- Test output
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Audit log (compliance)
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,  -- 'create', 'update', 'delete', 'view'
    entity_type VARCHAR(100),      -- 'wet', 'article', 'nrml'
    entity_id UUID,
    changes JSONB,                 -- Wat is er gewijzigd
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- NRML generation jobs (async)
CREATE TABLE nrml_generation_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID REFERENCES articles(id),
    status VARCHAR(50) DEFAULT 'pending', -- 'pending', 'processing', 'completed', 'failed'
    prompt TEXT,
    generated_nrml TEXT,
    model VARCHAR(100),            -- 'claude-sonnet-4-5'
    tokens_used INTEGER,
    error TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);

-- Indexes voor performance
CREATE INDEX idx_articles_wet_id ON articles(wet_id);
CREATE INDEX idx_articles_position ON articles(wet_id, position);
CREATE INDEX idx_mappings_article_id ON text_nrml_mappings(article_id);
CREATE INDEX idx_annotations_article_id ON annotations(article_id);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX idx_wet_references_source ON wet_references(source_article_id);
CREATE INDEX idx_wet_references_target ON wet_references(target_wet_id);
```

### 3.2 TypeScript Types (Frontend)

```typescript
// types/wet.ts
export interface Wet {
  id: string;
  bwbId?: string;
  title: string;
  shortTitle?: string;
  category: 'wet' | 'amvb' | 'ministeriele_regeling';
  color: string;
  status: 'draft' | 'review' | 'published';
  sourceUrl?: string;
  articles: Article[];
  createdBy: string;
  createdAt: string;
  updatedAt: string;
  metadata: Record<string, any>;
}

export interface Article {
  id: string;
  wetId: string;
  articleNumber: string;
  title?: string;
  content: string;  // Markdown
  nrmlYaml?: string;
  nrmlAst?: any;
  position: number;
  parentId?: string;
  children?: Article[];
  mappings: TextNRMLMapping[];
  annotations: Annotation[];
  createdAt: string;
  updatedAt: string;
}

export interface TextNRMLMapping {
  id: string;
  articleId: string;
  textRange: { start: number; end: number };
  nrmlRange: { startLine: number; endLine: number };
  mappingType: 'direct' | 'inferred' | 'ai_suggested';
  confidence: number;
}

export interface Annotation {
  id: string;
  articleId: string;
  userId: string;
  content: string;
  annotationType: 'comment' | 'question' | 'warning';
  position: { line: number; column: number } | { start: number; end: number };
  resolved: boolean;
  createdAt: string;
}

export interface WetReference {
  id: string;
  sourceArticleId: string;
  targetWetId: string;
  targetArticleId?: string;
  referenceText: string;
  referenceType: 'explicit' | 'implicit' | 'concept';
}

export interface Scenario {
  id: string;
  wetId: string;
  name: string;
  gherkinText: string;
  status: 'passed' | 'failed' | 'pending';
  lastRunAt?: string;
  lastRunResult?: any;
}

// types/nrml.ts
export interface NRMLDefinition {
  type: string;
  properties: Record<string, any>;
}

export interface NRMLRule {
  name: string;
  when: Condition[];
  then: Action[];
}

export interface Condition {
  type: 'comparison' | 'logical' | 'function';
  operator?: string;
  left?: any;
  right?: any;
  conditions?: Condition[];
}

export interface Action {
  type: 'assignment' | 'calculation' | 'decision';
  target: string;
  value: any;
}

// types/editor.ts
export interface EditorState {
  activeWetId: string | null;
  activeArticleId: string | null;
  openWets: Wet[];
  panels: {
    left: PanelState;
    middle: PanelState;
    right: PanelState;
  };
  middleMode: 'yaml' | 'blockly';
  rightTab: 'scenarios' | 'validation';
}

export interface PanelState {
  visible: boolean;
  width: number;  // Percentage
}

export interface SyncProposal {
  id: string;
  type: 'nrml_to_text' | 'text_to_nrml';
  sourceChange: string;
  proposedChange: string;
  confidence: number;
  status: 'pending' | 'accepted' | 'rejected';
}

// types/search.ts
export interface SearchResult {
  wet: {
    id: string;
    bwbId: string;
    title: string;
  };
  article: {
    id: string;
    number: string;
    title: string;
    snippet: string;  // Highlighted excerpt
  };
  score: number;
}
```

---

## 4. API Design

### 4.1 REST API Endpoints

**Base URL:** `https://api.regelrecht.minbzk.nl/api/v1`

```yaml
# Authentication
POST   /auth/login
POST   /auth/logout
POST   /auth/refresh
GET    /auth/me

# Wetten
GET    /wetten                    # List all wetten (paginated)
POST   /wetten                    # Create new wet
GET    /wetten/{wet_id}           # Get wet details
PUT    /wetten/{wet_id}           # Update wet
DELETE /wetten/{wet_id}           # Delete wet
GET    /wetten/{wet_id}/articles  # Get all articles for wet
POST   /wetten/{wet_id}/color     # Assign/update color

# Articles
GET    /articles/{article_id}
PUT    /articles/{article_id}
DELETE /articles/{article_id}
POST   /articles/{article_id}/nrml          # Update NRML
GET    /articles/{article_id}/mappings      # Get text-NRML mappings
PUT    /articles/{article_id}/mappings      # Update mappings

# Annotations
GET    /articles/{article_id}/annotations
POST   /articles/{article_id}/annotations
PUT    /annotations/{annotation_id}
DELETE /annotations/{annotation_id}

# Search (wetgeving corpus)
GET    /search?q={query}&type={type}&limit={limit}
POST   /search/advanced          # Advanced search met filters
GET    /search/suggestions       # Autocomplete voor @-mentions

# References
GET    /articles/{article_id}/references     # Get outgoing references
GET    /articles/{article_id}/referenced-by  # Get incoming references
POST   /articles/{article_id}/references     # Create new reference

# Scenarios
GET    /wetten/{wet_id}/scenarios
POST   /wetten/{wet_id}/scenarios
PUT    /scenarios/{scenario_id}
DELETE /scenarios/{scenario_id}
POST   /scenarios/{scenario_id}/run          # Run single scenario
POST   /wetten/{wet_id}/scenarios/run-all    # Run all scenarios

# LLM Operations
POST   /llm/generate-nrml        # Generate NRML from text
POST   /llm/text-proposal        # Generate text from NRML change
POST   /llm/improve-nrml         # Get improvement suggestions

# Sync
POST   /sync/proposal            # Create sync proposal (text ↔ NRML)
PUT    /sync/proposal/{id}/accept
PUT    /sync/proposal/{id}/reject
GET    /sync/proposals           # Get pending proposals

# Validation
POST   /validation/nrml          # Validate NRML syntax
GET    /validation/coverage/{wet_id}  # Get coverage statistics

# Audit
GET    /audit/logs               # Get audit logs (admin only)
```

### 4.2 WebSocket API (Real-time collaboration)

```yaml
WS     /ws/editor/{wet_id}       # Real-time updates for collaborative editing

# Messages:
# Client → Server
{
  "type": "cursor_position",
  "user_id": "uuid",
  "article_id": "uuid",
  "position": { "line": 10, "column": 5 }
}

{
  "type": "text_edit",
  "article_id": "uuid",
  "changes": [/* diff */]
}

# Server → Client
{
  "type": "user_joined",
  "user": { "id": "uuid", "name": "Anne Schuth" }
}

{
  "type": "article_updated",
  "article_id": "uuid",
  "updated_by": "uuid",
  "changes": [/* diff */]
}
```

---

## 5. LLM Integration Architecture

### 5.1 Claude API Integration

```python
# backend/app/services/llm_service.py

from anthropic import Anthropic
from app.core.cache import cache
import hashlib

class LLMService:
    def __init__(self):
        self.client = Anthropic(api_key=settings.ANTHROPIC_API_KEY)
        self.model = "claude-sonnet-4-5-20250929"
    
    async def generate_nrml(
        self, 
        article_text: str, 
        context: dict
    ) -> str:
        """
        Generate NRML from natural language article text.
        
        Args:
            article_text: The article content in Dutch
            context: Additional context (wet title, related articles, etc.)
        
        Returns:
            Generated NRML YAML code
        """
        # Cache key based on content hash
        cache_key = f"nrml_gen:{hashlib.sha256(article_text.encode()).hexdigest()}"
        
        # Check cache first
        cached = await cache.get(cache_key)
        if cached:
            return cached
        
        prompt = self._build_nrml_generation_prompt(article_text, context)
        
        response = await self.client.messages.create(
            model=self.model,
            max_tokens=4000,
            messages=[{"role": "user", "content": prompt}]
        )
        
        nrml_code = response.content[0].text
        
        # Cache for 7 days
        await cache.set(cache_key, nrml_code, ttl=604800)
        
        return nrml_code
    
    def _build_nrml_generation_prompt(
        self, 
        article_text: str, 
        context: dict
    ) -> str:
        return f"""Je bent een expert in het omzetten van Nederlandse wetgeving naar NRML (Normalized Rule Model Language).

CONTEXT:
Wet: {context.get('wet_title')}
Artikel: {context.get('article_number')}

ARTIKEL TEKST:
{article_text}

TAAK:
Converteer bovenstaande artikel naar valide NRML YAML code. Volg deze richtlijnen:

1. Identificeer alle begrippen/definities en maak `definitions` sectie
2. Extraheer alle voorwaarden en acties naar `rules`
3. Gebruik duidelijke variable namen
4. Voeg comments toe voor complexe logica
5. Zorg voor correcte YAML syntax

BELANGRIJKE NRML STRUCTUUR:
```yaml
# Begrippen
definitions:
  begrip_naam:
    type: Type
    constraint: voorwaarde
    source: bron

# Regels
rule: regel_naam
when:
  - voorwaarde1
  - voorwaarde2
then:
  - actie1
  - actie2
```

OUTPUT (alleen YAML, geen tekst ervoor of erna):
"""
    
    async def propose_text_change(
        self,
        original_text: str,
        nrml_change: str,
        nrml_context: str
    ) -> dict:
        """
        Propose natural language text change based on NRML edit.
        
        Returns:
            {
                "proposed_text": str,
                "explanation": str,
                "confidence": float
            }
        """
        prompt = f"""Een gebruiker heeft NRML code aangepast. Stel een passende wijziging voor in de natuurlijke taal tekst.

ORIGINELE TEKST:
{original_text}

NRML WIJZIGING:
{nrml_change}

VOLLEDIGE NRML CONTEXT:
{nrml_context}

TAAK:
Genereer een voorstel voor hoe de natuurlijke taal tekst aangepast moet worden om consistent te zijn met de NRML wijziging.

OUTPUT (JSON format):
{{
  "proposed_text": "De aangepaste tekst...",
  "explanation": "Deze wijziging is nodig omdat...",
  "confidence": 0.85
}}
"""
        
        response = await self.client.messages.create(
            model=self.model,
            max_tokens=2000,
            messages=[{"role": "user", "content": prompt}]
        )
        
        import json
        return json.loads(response.content[0].text)
    
    async def improve_nrml(self, nrml_code: str) -> list[dict]:
        """
        Get improvement suggestions for NRML code.
        
        Returns list of suggestions with severity and fix.
        """
        # Implementation similar to above
        pass
```

### 5.2 Rate Limiting & Cost Management

```python
# backend/app/core/rate_limiter.py

from redis import Redis
from datetime import datetime, timedelta

class LLMRateLimiter:
    def __init__(self, redis: Redis):
        self.redis = redis
        self.limits = {
            "user": 100,  # per hour
            "global": 1000  # per hour
        }
    
    async def check_and_increment(self, user_id: str) -> bool:
        """
        Check if user can make LLM call and increment counter.
        
        Returns True if allowed, False if rate limit exceeded.
        """
        hour_key = datetime.now().strftime("%Y%m%d%H")
        
        user_key = f"llm_rate:user:{user_id}:{hour_key}"
        global_key = f"llm_rate:global:{hour_key}"
        
        user_count = await self.redis.get(user_key) or 0
        global_count = await self.redis.get(global_key) or 0
        
        if int(user_count) >= self.limits["user"]:
            return False
        if int(global_count) >= self.limits["global"]:
            return False
        
        # Increment counters
        pipe = self.redis.pipeline()
        pipe.incr(user_key)
        pipe.expire(user_key, 3600)
        pipe.incr(global_key)
        pipe.expire(global_key, 3600)
        await pipe.execute()
        
        return True
```

---

## 6. Search Implementation

### 6.1 Elasticsearch Index Mapping

```json
{
  "mappings": {
    "properties": {
      "bwb_id": {
        "type": "keyword"
      },
      "wet_title": {
        "type": "text",
        "fields": {
          "keyword": { "type": "keyword" },
          "suggest": { "type": "completion" }
        },
        "analyzer": "dutch"
      },
      "wet_category": {
        "type": "keyword"
      },
      "article_number": {
        "type": "keyword"
      },
      "article_title": {
        "type": "text",
        "analyzer": "dutch"
      },
      "article_content": {
        "type": "text",
        "analyzer": "dutch",
        "term_vector": "with_positions_offsets"
      },
      "publication_date": {
        "type": "date"
      },
      "last_modified": {
        "type": "date"
      },
      "status": {
        "type": "keyword"
      },
      "tags": {
        "type": "keyword"
      }
    }
  },
  "settings": {
    "analysis": {
      "analyzer": {
        "dutch": {
          "type": "standard",
          "stopwords": "_dutch_"
        }
      }
    },
    "number_of_shards": 3,
    "number_of_replicas": 2
  }
}
```

### 6.2 Search Service

```python
# backend/app/services/search_service.py

from elasticsearch import AsyncElasticsearch
from typing import List
from app.schemas.search import SearchResult, SearchQuery

class SearchService:
    def __init__(self, es_client: AsyncElasticsearch):
        self.es = es_client
        self.index = "wetgeving"
    
    async def search(
        self, 
        query: SearchQuery
    ) -> List[SearchResult]:
        """
        Search wetgeving corpus with advanced options.
        """
        body = {
            "query": {
                "bool": {
                    "must": [
                        {
                            "multi_match": {
                                "query": query.q,
                                "fields": [
                                    "wet_title^3",
                                    "article_title^2",
                                    "article_content"
                                ],
                                "type": "best_fields",
                                "fuzziness": "AUTO"
                            }
                        }
                    ],
                    "filter": []
                }
            },
            "highlight": {
                "fields": {
                    "article_content": {
                        "fragment_size": 150,
                        "number_of_fragments": 3
                    }
                }
            },
            "size": query.limit,
            "from": query.offset
        }
        
        # Add filters
        if query.category:
            body["query"]["bool"]["filter"].append({
                "term": {"wet_category": query.category}
            })
        
        if query.date_from:
            body["query"]["bool"]["filter"].append({
                "range": {
                    "publication_date": {"gte": query.date_from}
                }
            })
        
        response = await self.es.search(
            index=self.index,
            body=body
        )
        
        return [
            SearchResult(
                wet=hit["_source"]["wet_title"],
                article_number=hit["_source"]["article_number"],
                snippet=hit["highlight"]["article_content"][0],
                score=hit["_score"]
            )
            for hit in response["hits"]["hits"]
        ]
    
    async def autocomplete(self, prefix: str) -> List[str]:
        """
        Autocomplete for @-mentions.
        """
        body = {
            "suggest": {
                "wet-suggest": {
                    "prefix": prefix,
                    "completion": {
                        "field": "wet_title.suggest",
                        "size": 10,
                        "fuzzy": {"fuzziness": 1}
                    }
                }
            }
        }
        
        response = await self.es.search(index=self.index, body=body)
        
        return [
            option["text"]
            for option in response["suggest"]["wet-suggest"][0]["options"]
        ]
```

### 6.3 wetten.nl Scraper

```python
# search/scripts/scrape_wetten.py

import asyncio
import aiohttp
from bs4 import BeautifulSoup
from elasticsearch import AsyncElasticsearch
import logging

class WettenScraper:
    """
    Scraper voor wetten.overheid.nl
    """
    
    BASE_URL = "https://wetten.overheid.nl"
    
    async def scrape_all(self):
        """
        Scrape all wetten and index in Elasticsearch.
        """
        async with aiohttp.ClientSession() as session:
            # Get list of all BWB IDs
            bwb_list = await self._fetch_bwb_list(session)
            
            logging.info(f"Found {len(bwb_list)} wetten to scrape")
            
            # Process in batches
            batch_size = 10
            for i in range(0, len(bwb_list), batch_size):
                batch = bwb_list[i:i+batch_size]
                await asyncio.gather(*[
                    self._scrape_wet(session, bwb_id)
                    for bwb_id in batch
                ])
                
                logging.info(f"Processed {i+len(batch)}/{len(bwb_list)}")
    
    async def _fetch_bwb_list(self, session) -> List[str]:
        """
        Fetch list of all BWB IDs from wetten.nl API.
        """
        # Implementation depends on wetten.nl API structure
        # This is a placeholder
        pass
    
    async def _scrape_wet(self, session, bwb_id: str):
        """
        Scrape single wet and index all articles.
        """
        url = f"{self.BASE_URL}/BWBR{bwb_id}"
        
        async with session.get(url) as response:
            html = await response.text()
            soup = BeautifulSoup(html, 'html.parser')
            
            # Extract metadata
            wet_title = soup.find('h1').text
            
            # Extract articles
            articles = soup.find_all('article')
            
            for article in articles:
                article_number = article.get('data-number')
                article_content = article.get_text()
                
                # Index in Elasticsearch
                await self._index_article(
                    bwb_id=bwb_id,
                    wet_title=wet_title,
                    article_number=article_number,
                    article_content=article_content
                )
    
    async def _index_article(self, **kwargs):
        """
        Index article in Elasticsearch.
        """
        # Implementation
        pass
```

---

## 7. Frontend State Management

### 7.1 Zustand Store

```typescript
// frontend/src/stores/editorStore.ts

import create from 'zustand';
import { devtools, persist } from 'zustand/middleware';
import { Wet, Article } from '../types/wet';
import { EditorState, PanelState, SyncProposal } from '../types/editor';

interface EditorStore extends EditorState {
  // Actions
  setActiveWet: (wetId: string) => void;
  setActiveArticle: (articleId: string) => void;
  addOpenWet: (wet: Wet) => void;
  removeOpenWet: (wetId: string) => void;
  togglePanel: (panel: 'left' | 'middle' | 'right') => void;
  setPanelWidth: (panel: 'left' | 'middle' | 'right', width: number) => void;
  setMiddleMode: (mode: 'yaml' | 'blockly') => void;
  setRightTab: (tab: 'scenarios' | 'validation') => void;
  
  // Sync proposals
  syncProposals: SyncProposal[];
  addSyncProposal: (proposal: SyncProposal) => void;
  acceptSyncProposal: (proposalId: string) => void;
  rejectSyncProposal: (proposalId: string) => void;
}

export const useEditorStore = create<EditorStore>()(
  devtools(
    persist(
      (set, get) => ({
        // Initial state
        activeWetId: null,
        activeArticleId: null,
        openWets: [],
        panels: {
          left: { visible: true, width: 33 },
          middle: { visible: true, width: 34 },
          right: { visible: true, width: 33 },
        },
        middleMode: 'yaml',
        rightTab: 'scenarios',
        syncProposals: [],
        
        // Actions
        setActiveWet: (wetId) => set({ activeWetId: wetId }),
        
        setActiveArticle: (articleId) => set({ activeArticleId: articleId }),
        
        addOpenWet: (wet) => set((state) => ({
          openWets: [...state.openWets, wet],
          activeWetId: wet.id,
        })),
        
        removeOpenWet: (wetId) => set((state) => {
          const newOpenWets = state.openWets.filter(w => w.id !== wetId);
          return {
            openWets: newOpenWets,
            activeWetId: newOpenWets.length > 0 ? newOpenWets[0].id : null,
          };
        }),
        
        togglePanel: (panel) => set((state) => ({
          panels: {
            ...state.panels,
            [panel]: {
              ...state.panels[panel],
              visible: !state.panels[panel].visible,
            },
          },
        })),
        
        setPanelWidth: (panel, width) => set((state) => ({
          panels: {
            ...state.panels,
            [panel]: { ...state.panels[panel], width },
          },
        })),
        
        setMiddleMode: (mode) => set({ middleMode: mode }),
        
        setRightTab: (tab) => set({ rightTab: tab }),
        
        addSyncProposal: (proposal) => set((state) => ({
          syncProposals: [...state.syncProposals, proposal],
        })),
        
        acceptSyncProposal: async (proposalId) => {
          const proposal = get().syncProposals.find(p => p.id === proposalId);
          if (!proposal) return;
          
          // Apply the change (implementation depends on proposal type)
          // ...
          
          set((state) => ({
            syncProposals: state.syncProposals.filter(p => p.id !== proposalId),
          }));
        },
        
        rejectSyncProposal: (proposalId) => set((state) => ({
          syncProposals: state.syncProposals.filter(p => p.id !== proposalId),
        })),
      }),
      {
        name: 'regelrecht-editor',
        partialize: (state) => ({
          panels: state.panels,
          middleMode: state.middleMode,
          rightTab: state.rightTab,
        }),
      }
    )
  )
);
```

---

## 8. Development Workflow with Claude Code

### 8.1 Setting up Claude Code

```bash
# Install Claude Code
npm install -g @anthropic-ai/claude-code

# Authenticate
claude-code auth

# Initialize in project
cd regelrecht
claude-code init
```

### 8.2 Claude Code Usage Patterns

**Pattern 1: Implementing a new feature**

```bash
# Example: Implementing @-mention search
claude-code task "Implement @-mention autocomplete search in the left panel editor. 
When user types @ followed by text, show dropdown with search results from Elasticsearch.
Use the cmdk library for the command palette UI. 
Connect to /api/v1/search/suggestions endpoint.
Follow the existing patterns in AtMentionSearch.tsx component skeleton."

# Claude Code will:
# 1. Read existing code (AtMentionSearch.tsx, searchService.ts)
# 2. Implement the autocomplete logic
# 3. Add proper TypeScript types
# 4. Write unit tests
# 5. Update documentation
```

**Pattern 2: Fixing bugs**

```bash
claude-code task "There's a bug in scroll synchronization - when clicking an article in the left panel,
the middle panel doesn't scroll to the correct NRML block. Debug and fix the useScrollSync hook.
The issue seems to be related to the article-to-NRML mapping calculation."

# Claude will:
# 1. Analyze the useScrollSync hook
# 2. Check the mapping logic
# 3. Add debug logging
# 4. Fix the bug
# 5. Add test case for this scenario
```

**Pattern 3: Refactoring**

```bash
claude-code task "Refactor the bidirectional sync logic in sync_service.py.
Split it into separate functions for:
- Text change detection
- NRML change detection  
- Proposal generation
- Proposal application
Add proper error handling and logging throughout."
```

**Pattern 4: Code review**

```bash
claude-code review backend/app/services/llm_service.py
# Claude will analyze code quality, suggest improvements, check for bugs
```

### 8.3 Prompting Strategy for Claude Code

**Good prompts for Claude Code:**

✅ **Specific and contextual**
```
"Add a color picker component to WetTabs.tsx that allows users to change 
the color of a wet. Store the color in the wetten table. Use the existing 
ColorPicker component from common/ and follow the pattern in ArticleEditor.tsx 
for handling updates."
```

✅ **Reference existing patterns**
```
"Create a new API endpoint for NRML validation following the same pattern
as /validation/nrml but for batch validation. Take inspiration from 
scenarios/run-all endpoint structure."
```

✅ **Include acceptance criteria**
```
"Implement page flip animation between wet tabs. Requirements:
- Use framer-motion library
- Animation duration: 400ms
- For adjacent tabs: simple slide
- For distant tabs (>2 apart): flip effect with multiple pages
- Maintain scroll position during transition
- Add unit tests for animation triggers"
```

❌ **Vague prompts to avoid**
```
"Make the editor better"
"Fix performance issues"
"Add some tests"
```

### 8.4 Development Workflow

```mermaid
graph TD
    A[Feature Request] --> B[Create Issue]
    B --> C[Write Claude Code prompt]
    C --> D[Claude generates code]
    D --> E[Review & Test]
    E --> F{Acceptable?}
    F -->|No| G[Refine prompt]
    G --> C
    F -->|Yes| H[Commit & PR]
    H --> I[CI Pipeline]
    I --> J{Tests Pass?}
    J -->|No| K[Fix issues]
    K --> E
    J -->|Yes| L[Merge]
```

---

## 9. Testing Strategy

### 9.1 Frontend Testing

```typescript
// frontend/src/components/editor/LeftPanel/__tests__/AtMentionSearch.test.tsx

import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { AtMentionSearch } from '../AtMentionSearch';
import { searchService } from '../../../../services/searchService';

jest.mock('../../../../services/searchService');

describe('AtMentionSearch', () => {
  it('shows dropdown when typing @ followed by text', async () => {
    render(<AtMentionSearch />);
    
    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: '@inkomen' } });
    
    await waitFor(() => {
      expect(screen.getByRole('listbox')).toBeInTheDocument();
    });
  });
  
  it('calls search API with debounced query', async () => {
    const mockSearch = jest.fn().mockResolvedValue([
      { wet: 'Wet IB 2001', article: '2.1', snippet: '...' }
    ]);
    searchService.autocomplete = mockSearch;
    
    render(<AtMentionSearch />);
    
    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: '@inkomen' } });
    
    // Should debounce (wait 300ms)
    expect(mockSearch).not.toHaveBeenCalled();
    
    await waitFor(() => {
      expect(mockSearch).toHaveBeenCalledWith('inkomen');
    }, { timeout: 500 });
  });
  
  it('inserts reference when selecting result', async () => {
    const mockOnInsert = jest.fn();
    render(<AtMentionSearch onInsertReference={mockOnInsert} />);
    
    // ... select a result
    
    expect(mockOnInsert).toHaveBeenCalledWith({
      wetId: 'uuid',
      articleId: 'uuid',
      text: 'Wet IB 2001 - Art. 2.1'
    });
  });
});
```

### 9.2 Backend Testing

```python
# backend/app/tests/unit/test_llm_service.py

import pytest
from app.services.llm_service import LLMService
from unittest.mock import AsyncMock, patch

@pytest.fixture
def llm_service():
    return LLMService()

@pytest.mark.asyncio
async def test_generate_nrml(llm_service):
    """Test NRML generation from article text."""
    
    article_text = """
    Een burger komt in aanmerking voor een uitkering indien:
    a. het inkomen niet hoger is dan € 30.000;
    b. de burger jonger is dan 65 jaar.
    """
    
    context = {
        "wet_title": "Test Wet",
        "article_number": "2.1"
    }
    
    with patch.object(llm_service.client.messages, 'create') as mock_create:
        mock_create.return_value = AsyncMock(
            content=[
                AsyncMock(text="""
rule: voorwaarden_uitkering
when:
  - inkomen <= 30000
  - leeftijd < 65
then:
  - eligible: true
""")
            ]
        )
        
        result = await llm_service.generate_nrml(article_text, context)
        
        assert "rule: voorwaarden_uitkering" in result
        assert "inkomen <= 30000" in result
        assert "leeftijd < 65" in result

@pytest.mark.asyncio
async def test_rate_limiting(llm_service):
    """Test rate limiting prevents excessive calls."""
    
    # Make 100 calls (should succeed)
    for i in range(100):
        can_call = await llm_service.rate_limiter.check_and_increment("user-123")
        assert can_call is True
    
    # 101st call should fail
    can_call = await llm_service.rate_limiter.check_and_increment("user-123")
    assert can_call is False
```

### 9.3 E2E Testing

```typescript
// e2e/tests/editor-workflow.spec.ts

import { test, expect } from '@playwright/test';

test.describe('Editor Workflow', () => {
  test('complete workflow: search wet -> edit NRML -> run scenario', async ({ page }) => {
    await page.goto('/');
    
    // Login
    await page.fill('[name="email"]', 'test@minbzk.nl');
    await page.fill('[name="password"]', 'password');
    await page.click('button[type="submit"]');
    
    // Search and open a wet
    await page.click('[data-testid="search-button"]');
    await page.fill('[data-testid="search-input"]', 'Participatiewet');
    await page.click('[data-testid="search-result-0"]');
    
    // Verify wet is opened with correct color
    await expect(page.locator('[data-testid="wet-tab-active"]')).toHaveCSS(
      'border-color',
      /rgb\(\d+, \d+, \d+\)/  // Some color
    );
    
    // Click on article
    await page.click('[data-testid="article-2.1"]');
    
    // Edit NRML in middle panel
    await page.click('[data-testid="yaml-mode"]');
    const editor = page.locator('.monaco-editor');
    await editor.click();
    await page.keyboard.type('\n  - nieuwe_voorwaarde: true');
    
    // Verify sync proposal appears
    await expect(page.locator('[data-testid="sync-proposal"]')).toBeVisible();
    
    // Run scenario
    await page.click('[data-testid="right-tab-scenarios"]');
    await page.click('[data-testid="run-scenario-0"]');
    
    // Verify result
    await expect(page.locator('[data-testid="scenario-status"]')).toHaveText('passed');
  });
  
  test('@-mention search and reference creation', async ({ page }) => {
    await page.goto('/editor');
    
    // Click in article editor
    const articleEditor = page.locator('[data-testid="article-editor"]');
    await articleEditor.click();
    
    // Type @-mention
    await page.keyboard.type('Dit artikel verwijst naar @inkomen');
    
    // Wait for autocomplete
    await expect(page.locator('[role="listbox"]')).toBeVisible();
    
    // Select first result
    await page.click('[data-testid="autocomplete-option-0"]');
    
    // Verify new tab opened
    await expect(page.locator('[data-testid="wet-tabs"]')).toContainText('Wet IB 2001');
    
    // Verify reference created
    await expect(page.locator('[data-testid="reference-link"]')).toBeVisible();
  });
});
```

### 9.4 CI/CD Pipeline

```yaml
# .github/workflows/ci.yml

name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  frontend-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json
      
      - name: Install dependencies
        working-directory: frontend
        run: npm ci
      
      - name: Type check
        working-directory: frontend
        run: npm run type-check
      
      - name: Lint
        working-directory: frontend
        run: npm run lint
      
      - name: Unit tests
        working-directory: frontend
        run: npm run test:unit -- --coverage
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: frontend/coverage/coverage-final.json
  
  backend-test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      
      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'
          cache: 'pip'
      
      - name: Install dependencies
        working-directory: backend
        run: |
          pip install -r requirements.txt
          pip install pytest-cov
      
      - name: Run migrations
        working-directory: backend
        run: alembic upgrade head
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost/test
      
      - name: Run tests
        working-directory: backend
        run: pytest --cov=app --cov-report=xml
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost/test
          REDIS_URL: redis://localhost:6379/0
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: backend/coverage.xml
  
  e2e-test:
    runs-on: ubuntu-latest
    needs: [frontend-test, backend-test]
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup services (docker-compose)
        run: docker-compose -f docker/docker-compose.test.yml up -d
      
      - name: Wait for services
        run: |
          timeout 60 bash -c 'until curl -f http://localhost:3000/health; do sleep 2; done'
      
      - name: Run Playwright tests
        run: |
          npm install -g @playwright/test
          playwright test
      
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-results
          path: test-results/
```

---

## 10. Security & Compliance

### 10.1 Security Requirements (MinBZK)

**Authentication & Authorization:**
- [ ] OAuth 2.0 / OpenID Connect integration met MinBZK IdP
- [ ] Multi-factor authentication (MFA) verplicht
- [ ] Role-based access control (RBAC)
- [ ] Session timeout: 30 minuten inactiviteit
- [ ] Password policy: min 12 chars, complexity requirements

**Data Protection:**
- [ ] All data encrypted at rest (AES-256)
- [ ] All data encrypted in transit (TLS 1.3)
- [ ] Database credentials in secrets management (Vault/Azure Key Vault)
- [ ] API keys rotated quarterly
- [ ] No sensitive data in logs

**Audit & Compliance:**
- [ ] Comprehensive audit logging (wie, wat, wanneer)
- [ ] Audit logs retention: 7 jaar (wettelijk verplicht)
- [ ] Regular security scans (OWASP ZAP, SonarQube)
- [ ] Penetration testing annually
- [ ] AVG compliance (privacy impact assessment)
- [ ] BIO (Baseline Informatiebeveiliging Overheid) compliant

**Network Security:**
- [ ] API rate limiting (zie sectie 5.2)
- [ ] CORS policy restricted to known domains
- [ ] CSP headers configured
- [ ] No external dependencies from untrusted CDNs
- [ ] All traffic through government network

### 10.2 Implementation: Security Middleware

```python
# backend/app/middleware/security.py

from fastapi import Request, HTTPException
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from jose import jwt, JWTError
import time

class SecurityMiddleware:
    """
    Security middleware for RegelRecht API.
    """
    
    def __init__(self, app):
        self.app = app
        self.security = HTTPBearer()
    
    async def __call__(self, request: Request, call_next):
        # 1. Rate limiting (per IP)
        if not await self.check_rate_limit(request):
            raise HTTPException(status_code=429, detail="Too many requests")
        
        # 2. JWT validation (except public endpoints)
        if not self.is_public_endpoint(request.url.path):
            try:
                token = await self.security(request)
                user = await self.validate_token(token.credentials)
                request.state.user = user
            except Exception as e:
                raise HTTPException(status_code=401, detail="Invalid authentication")
        
        # 3. Audit logging
        start_time = time.time()
        response = await call_next(request)
        duration = time.time() - start_time
        
        await self.log_request(request, response, duration)
        
        # 4. Security headers
        response.headers["X-Content-Type-Options"] = "nosniff"
        response.headers["X-Frame-Options"] = "DENY"
        response.headers["X-XSS-Protection"] = "1; mode=block"
        response.headers["Strict-Transport-Security"] = "max-age=31536000; includeSubDomains"
        
        return response
    
    async def validate_token(self, token: str) -> dict:
        """Validate JWT token and return user claims."""
        try:
            payload = jwt.decode(
                token,
                settings.JWT_PUBLIC_KEY,
                algorithms=["RS256"]
            )
            
            # Check expiration
            if payload.get("exp") < time.time():
                raise HTTPException(status_code=401, detail="Token expired")
            
            # Check required claims
            if not all(k in payload for k in ["sub", "email", "roles"]):
                raise HTTPException(status_code=401, detail="Invalid token claims")
            
            return payload
            
        except JWTError:
            raise HTTPException(status_code=401, detail="Invalid token")
    
    async def log_request(self, request: Request, response, duration: float):
        """Log request for audit trail."""
        from app.models.audit_log import AuditLog
        
        user = getattr(request.state, "user", None)
        
        log = AuditLog(
            user_id=user.get("sub") if user else None,
            action=f"{request.method} {request.url.path}",
            entity_type="api_request",
            ip_address=request.client.host,
            user_agent=request.headers.get("user-agent"),
            metadata={
                "status_code": response.status_code,
                "duration_ms": int(duration * 1000)
            }
        )
        
        # Save to database (async)
        await log.save()
```

### 10.3 Secrets Management

```yaml
# kubernetes/secrets.yaml (encrypted with SealedSecrets)

apiVersion: v1
kind: Secret
metadata:
  name: regelrecht-secrets
  namespace: regelrecht
type: Opaque
stringData:
  POSTGRES_PASSWORD: "<encrypted>"
  JWT_PRIVATE_KEY: "<encrypted>"
  JWT_PUBLIC_KEY: "<encrypted>"
  ANTHROPIC_API_KEY: "<encrypted>"
  REDIS_PASSWORD: "<encrypted>"
  ELASTICSEARCH_PASSWORD: "<encrypted>"
```

```python
# backend/app/config.py

from pydantic_settings import BaseSettings
from functools import lru_cache

class Settings(BaseSettings):
    # Database
    DATABASE_URL: str
    DATABASE_POOL_SIZE: int = 20
    
    # Redis
    REDIS_URL: str
    
    # Elasticsearch
    ELASTICSEARCH_URL: str
    ELASTICSEARCH_PASSWORD: str
    
    # JWT
    JWT_PRIVATE_KEY: str
    JWT_PUBLIC_KEY: str
    JWT_ALGORITHM: str = "RS256"
    JWT_EXPIRATION_MINUTES: int = 30
    
    # Anthropic
    ANTHROPIC_API_KEY: str
    
    # Security
    CORS_ORIGINS: list[str] = ["https://regelrecht.minbzk.nl"]
    ALLOWED_HOSTS: list[str] = ["regelrecht.minbzk.nl", "api.regelrecht.minbzk.nl"]
    
    # Rate limiting
    RATE_LIMIT_PER_MINUTE: int = 60
    LLM_RATE_LIMIT_PER_HOUR: int = 100
    
    class Config:
        env_file = ".env"
        case_sensitive = True

@lru_cache()
def get_settings() -> Settings:
    return Settings()
```

---

## 11. Deployment Architecture

### 11.1 Infrastructure (Kubernetes on Nederlandse cloud)

```
┌─────────────────────────────────────────────────────────────┐
│                     Load Balancer (Ingress)                 │
│                   https://regelrecht.minbzk.nl               │
└─────────────────────┬───────────────────────────────────────┘
                      │
        ┌─────────────┴─────────────┐
        │                           │
        ▼                           ▼
┌───────────────┐          ┌────────────────┐
│   Frontend    │          │    Backend     │
│   (React)     │          │   (FastAPI)    │
│   3 replicas  │          │   5 replicas   │
└───────┬───────┘          └────────┬───────┘
        │                           │
        │                           ├──────────────┐
        │                           │              │
        ▼                           ▼              ▼
┌───────────────┐          ┌────────────┐  ┌──────────────┐
│     CDN       │          │ PostgreSQL │  │ Elasticsearch│
│  Static files │          │  Primary   │  │  3 nodes     │
└───────────────┘          │  + Replica │  └──────────────┘
                           └─────┬──────┘
                                 │
                           ┌─────┴──────┐
                           │   Redis    │
                           │  Cluster   │
                           └────────────┘
```

### 11.2 Kubernetes Deployment

```yaml
# kubernetes/deployments/backend.yaml

apiVersion: apps/v1
kind: Deployment
metadata:
  name: regelrecht-backend
  namespace: regelrecht
spec:
  replicas: 5
  selector:
    matchLabels:
      app: regelrecht-backend
  template:
    metadata:
      labels:
        app: regelrecht-backend
    spec:
      containers:
      - name: backend
        image: registry.minbzk.nl/regelrecht/backend:latest
        ports:
        - containerPort: 8000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: regelrecht-secrets
              key: DATABASE_URL
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: regelrecht-secrets
              key: REDIS_URL
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: regelrecht-secrets
              key: ANTHROPIC_API_KEY
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8000
          initialDelaySeconds: 10
          periodSeconds: 5
      
      # Sidecar for log shipping
      - name: fluentd
        image: fluent/fluentd:v1.14
        volumeMounts:
        - name: logs
          mountPath: /var/log
      
      volumes:
      - name: logs
        emptyDir: {}
---
apiVersion: v1
kind: Service
metadata:
  name: regelrecht-backend
  namespace: regelrecht
spec:
  selector:
    app: regelrecht-backend
  ports:
  - port: 80
    targetPort: 8000
  type: ClusterIP
```

### 11.3 Deployment Script

```bash
#!/bin/bash
# scripts/deploy.sh

set -e

ENV=$1  # dev, staging, prod

if [ -z "$ENV" ]; then
  echo "Usage: ./deploy.sh [dev|staging|prod]"
  exit 1
fi

echo "🚀 Deploying RegelRecht to $ENV environment..."

# 1. Build Docker images
echo "📦 Building Docker images..."
docker build -t registry.minbzk.nl/regelrecht/frontend:$ENV -f docker/Dockerfile.frontend .
docker build -t registry.minbzk.nl/regelrecht/backend:$ENV -f docker/Dockerfile.backend .

# 2. Push to registry
echo "📤 Pushing to registry..."
docker push registry.minbzk.nl/regelrecht/frontend:$ENV
docker push registry.minbzk.nl/regelrecht/backend:$ENV

# 3. Run database migrations
echo "🗄️  Running database migrations..."
kubectl exec -n regelrecht deployment/regelrecht-backend -- alembic upgrade head

# 4. Deploy to Kubernetes
echo "☸️  Deploying to Kubernetes..."
kubectl apply -f kubernetes/namespace.yaml
kubectl apply -f kubernetes/configmap-$ENV.yaml
kubectl apply -f kubernetes/secrets-$ENV.yaml
kubectl apply -f kubernetes/deployments/
kubectl apply -f kubernetes/services/
kubectl apply -f kubernetes/ingress-$ENV.yaml

# 5. Wait for rollout
echo "⏳ Waiting for rollout to complete..."
kubectl rollout status deployment/regelrecht-frontend -n regelrecht
kubectl rollout status deployment/regelrecht-backend -n regelrecht

# 6. Run smoke tests
echo "🧪 Running smoke tests..."
./scripts/smoke-test.sh $ENV

echo "✅ Deployment complete!"
```

---

## 12. Monitoring & Observability

### 12.1 Metrics (Prometheus)

```python
# backend/app/middleware/metrics.py

from prometheus_client import Counter, Histogram, Gauge
import time

# Request metrics
http_requests_total = Counter(
    'http_requests_total',
    'Total HTTP requests',
    ['method', 'endpoint', 'status']
)

http_request_duration = Histogram(
    'http_request_duration_seconds',
    'HTTP request duration',
    ['method', 'endpoint']
)

# Business metrics
nrml_generations_total = Counter(
    'nrml_generations_total',
    'Total NRML generations',
    ['status']  # success, failure
)

active_users = Gauge(
    'active_users',
    'Number of active users'
)

open_wetten = Gauge(
    'open_wetten',
    'Number of currently open wetten'
)

class MetricsMiddleware:
    async def __call__(self, request: Request, call_next):
        start_time = time.time()
        
        response = await call_next(request)
        
        duration = time.time() - start_time
        
        http_requests_total.labels(
            method=request.method,
            endpoint=request.url.path,
            status=response.status_code
        ).inc()
        
        http_request_duration.labels(
            method=request.method,
            endpoint=request.url.path
        ).observe(duration)
        
        return response
```

### 12.2 Logging (Structured)

```python
# backend/app/core/logging.py

import structlog
import logging

def setup_logging():
    structlog.configure(
        processors=[
            structlog.stdlib.filter_by_level,
            structlog.stdlib.add_logger_name,
            structlog.stdlib.add_log_level,
            structlog.processors.TimeStamper(fmt="iso"),
            structlog.processors.StackInfoRenderer(),
            structlog.processors.format_exc_info,
            structlog.processors.UnicodeDecoder(),
            structlog.processors.JSONRenderer()
        ],
        context_class=dict,
        logger_factory=structlog.stdlib.LoggerFactory(),
        cache_logger_on_first_use=True,
    )

logger = structlog.get_logger()

# Usage:
logger.info(
    "nrml_generated",
    article_id="uuid",
    wet_id="uuid",
    user_id="uuid",
    tokens_used=1234,
    duration_ms=567
)
```

### 12.3 Alerting (Grafana)

```yaml
# monitoring/alerts.yaml

groups:
- name: regelrecht
  interval: 30s
  rules:
  
  # High error rate
  - alert: HighErrorRate
    expr: |
      sum(rate(http_requests_total{status=~"5.."}[5m])) 
      / 
      sum(rate(http_requests_total[5m])) > 0.05
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error rate detected"
      description: "Error rate is {{ $value | humanizePercentage }}"
  
  # Slow API responses
  - alert: SlowAPIResponses
    expr: |
      histogram_quantile(0.95, 
        rate(http_request_duration_seconds_bucket[5m])
      ) > 2
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "95th percentile response time > 2s"
  
  # LLM rate limit approaching
  - alert: LLMRateLimitApproaching
    expr: |
      rate(llm_rate_limit_exceeded_total[1h]) > 80
    labels:
      severity: warning
    annotations:
      summary: "Approaching LLM rate limit"
  
  # Database connection pool exhausted
  - alert: DatabasePoolExhausted
    expr: |
      db_connection_pool_in_use / db_connection_pool_size > 0.9
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "Database connection pool > 90% utilized"
```

---

## 13. Implementation Roadmap

### Phase 1: Foundation (Maanden 1-2)

**Week 1-2: Project Setup**
- [ ] Repository structure
- [ ] CI/CD pipeline
- [ ] Development environment (Docker Compose)
- [ ] Database schema & migrations
- [ ] Authentication setup

**Week 3-4: Core Editor UI**
- [ ] Three-panel layout
- [ ] Panel resizing & collapsing
- [ ] Basic wet loading
- [ ] Article list rendering
- [ ] YAML editor (Monaco)

**Week 5-6: Basic NRML**
- [ ] YAML parser
- [ ] NRML validation
- [ ] Article ↔ NRML mapping (manual)
- [ ] Basic scenario runner

**Week 7-8: MVP Testing**
- [ ] End-to-end tests
- [ ] User acceptance testing met 3-5 wetgevings juristen
- [ ] Performance testing
- [ ] Security audit

**Deliverable:** MVP met basis editing functionaliteit

---

### Phase 2: Advanced Features (Maanden 3-4)

**Week 9-10: Blockly Integration**
- [ ] Blockly setup
- [ ] YAML ↔ Blockly converter
- [ ] Custom blocks voor NRML constructs
- [ ] Mode switching

**Week 11-12: Color & Navigation**
- [ ] Wet color system
- [ ] Tab navigation
- [ ] Page flip animations
- [ ] Tab management (close, reorder)

**Week 13-14: Annotations**
- [ ] Comment system
- [ ] Annotation UI
- [ ] Annotation persistence
- [ ] Notification system

**Week 15-16: Validation & Coverage**
- [ ] NRML linter
- [ ] Coverage analysis
- [ ] Dependency detection
- [ ] Conflict detection

**Deliverable:** Feature-complete editor zonder AI

---

### Phase 3: Search & AI (Maanden 5-6)

**Week 17-18: Elasticsearch Setup**
- [ ] ES cluster setup
- [ ] wetten.nl scraper
- [ ] Index creation & mapping
- [ ] Search API

**Week 19-20: @-mention Search**
- [ ] Autocomplete UI
- [ ] Search integration
- [ ] Reference creation
- [ ] Cross-wet navigation

**Week 21-22: LLM Integration**
- [ ] Claude API setup
- [ ] NRML generation prompt engineering
- [ ] Rate limiting
- [ ] Caching strategy

**Week 23-24: Bidirectional Sync**
- [ ] Text change detection
- [ ] NRML change detection
- [ ] Proposal UI
- [ ] Acceptance/rejection workflow

**Deliverable:** Volledig functioneel systeem met AI

---

### Phase 4: Production Readiness (Maanden 7-8)

**Week 25-26: Security Hardening**
- [ ] Security audit
- [ ] Penetration testing
- [ ] AVG compliance review
- [ ] BIO compliance check

**Week 27-28: Performance Optimization**
- [ ] Frontend bundle optimization
- [ ] Database query optimization
- [ ] Caching improvements
- [ ] Load testing

**Week 29-30: Documentation**
- [ ] User manual
- [ ] API documentation
- [ ] Development guide
- [ ] Operations runbook

**Week 31-32: Beta Launch**
- [ ] Beta deployment
- [ ] User training (10-20 wetgevings juristen)
- [ ] Feedback collection
- [ ] Bug fixes

**Deliverable:** Production-ready systeem

---

## 14. Open Questions & Decisions Needed

### 14.1 Technical Decisions

❓ **Blockly vs. custom visual editor**
- Blockly: battle-tested, maar overhead
- Custom: meer controle, maar development tijd
- **Beslissing nodig:** Week 1

❓ **Real-time collaboration scope**
- Volledig Google Docs-achtig? (complex)
- Of alleen "wie is waar" indicators? (simpeler)
- **Beslissing nodig:** Week 4

❓ **NRML execution engine**
- Python interpreter?
- Separate DSL runtime?
- **Beslissing nodig:** Week 6

❓ **Hosting platform**
- Ministerie eigen Kubernetes?
- SSC-ICT (Shared Service Centrum)?
- Commercial cloud met NL data residency?
- **Beslissing nodig:** ASAP

### 14.2 Product Decisions

❓ **Wet versioning**
- Git-achtig versioning systeem?
- Of simpler audit log approach?
- **Beslissing nodig:** Week 2

❓ **Collaboration features**
- Comments/reviews zoals Google Docs?
- Approval workflows?
- **Beslissing nodig:** Week 8

❓ **NRML generation: how much human review?**
- AI generates → human approves? (safer)
- AI generates → auto-applies? (faster but riskier)
- **Beslissing nodig:** Week 20

---

## 15. Success Metrics

**Usage Metrics:**
- [ ] 50+ actieve gebruikers within 6 months
- [ ] 100+ wetten converted to NRML
- [ ] 1000+ scenarios created

**Quality Metrics:**
- [ ] >95% NRML validation pass rate
- [ ] <5% error rate in scenario runs
- [ ] >90% user satisfaction score

**Performance Metrics:**
- [ ] <2s page load time (p95)
- [ ] <500ms API response time (p95)
- [ ] <10s NRML generation time (p95)

**Business Metrics:**
- [ ] 30% reduction in time to digitize wetgeving
- [ ] 50% reduction in NRML errors vs. manual coding
- [ ] Cost savings: €X per wet digitized

---

## 16. Risk Mitigation

**Risk 1: LLM quality voor Nederlandse wetgeving**
- Mitigation: Extensive prompt engineering + human review workflow
- Fallback: Template-based generation voor common patterns

**Risk 2: Performance met grote wetten (1000+ artikelen)**
- Mitigation: Virtualization, lazy loading, pagination
- Fallback: Split large wetten into logical sections

**Risk 3: User adoption (change management)**
- Mitigation: Extensive training, pilot program, champion users
- Fallback: Phased rollout, parallel with existing tools

**Risk 4: Security breach**
- Mitigation: Defense in depth, regular audits, incident response plan
- Fallback: Immediate shutdown, forensics, stakeholder communication

**Risk 5: Vendor lock-in (Anthropic API)**
- Mitigation: Abstraction layer, evaluate alternatives (OpenAI, open-source LLMs)
- Fallback: Template-based generation, manual NRML authoring

---

## Appendix A: Glossary

- **NRML:** Normalized Rule Model Language - DSL voor machine-uitvoerbare wetgeving
- **BWB:** Basis Wetten Bestand - ID systeem van wetten.overheid.nl
- **AMvB:** Algemene Maatregel van Bestuur
- **BIO:** Baseline Informatiebeveiliging Overheid
- **AVG:** Algemene Verordening Gegevensbescherming (GDPR)
- **NL-SBB:** Nederlandse Standaard voor Besturing en Beveiliging
- **NORA:** Nederlandse Overheid Referentie Architectuur

---

## Appendix B: References

- [wetten.overheid.nl API docs](https://data.overheid.nl/dataset/9830-wetten-overheid-nl)
- [NORA Principes](https://www.noraonline.nl/)
- [BIO normenkader](https://www.digitaleoverheid.nl/overzicht-van-alle-onderwerpen/cybersecurity/kaders-voor-cybersecurity/baseline-informatiebeveiliging-overheid/)
- [Anthropic Claude API docs](https://docs.anthropic.com/)
- [React Best Practices](https://react.dev/)
- [FastAPI Documentation](https://fastapi.tiangolo.com/)

---

**Document Versie:** 1.0
**Laatst bijgewerkt:** 15 november 2025
**Auteur:** Claude (met Anne Schuth)
**Status:** Draft - Feedback welkom

---

*Volgende stappen:*
1. Review dit document met team
2. Maak beslissingen op open questions
3. Start met Phase 1, Week 1 tasks
4. Setup eerste Claude Code sessie
5. Begin development! 🚀
