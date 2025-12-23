# Architecture du Service de Devis 3D

## Vue d'ensemble

Le service de devis 3D est une application web full-stack construite avec Rust (backend) et JavaScript vanilla (frontend), permettant aux utilisateurs de télécharger des modèles 3D et d'obtenir des devis instantanés pour l'impression 3D.

## Diagramme d'architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Clients                                  │
├──────────────┬────────────────────┬─────────────────────────────┤
│  Navigateur  │   MCP Clients      │   Admin Interface           │
│  Web         │   (AI Models)      │   (/admin.html)             │
└──────┬───────┴────────┬───────────┴─────────┬───────────────────┘
       │                │                     │
       │ HTTP           │ HTTP                │ HTTP + Cookie Auth
       │                │ Bearer Token        │
       │                │                     │
┌──────▼────────────────▼─────────────────────▼───────────────────┐
│                    Axum Web Server                               │
│                   (Port 3000, Rust)                              │
├──────────────────────────────────────────────────────────────────┤
│  Middleware Stack:                                               │
│  ├─ Security Headers (CSP, HSTS, etc.)                          │
│  ├─ Rate Limiting (100 req/s global)                            │
│  ├─ CORS                                                         │
│  ├─ Authentication (admin_auth, mcp_auth)                       │
│  └─ Tracing/Logging                                             │
├──────────────────────────────────────────────────────────────────┤
│  Routes:                                                         │
│  ├─ /                  → SSR Pages (index, admin)               │
│  ├─ /api/sessions      → Session Management                     │
│  ├─ /api/materials     → Material Catalog                       │
│  ├─ /api/admin/*       → Admin Endpoints (auth required)        │
│  ├─ /mcp               → MCP Protocol (auth required)           │
│  ├─ /static/*          → Static Assets (JS, CSS)                │
│  └─ /uploads/*         → Uploaded 3D Models                     │
└──────┬────────────────┬──────────────┬────────────┬─────────────┘
       │                │              │            │
       │ HTTP           │ HTTP         │ File I/O   │ SQL
       │ Handlers       │ Handlers     │            │
       │                │              │            │
┌──────▼────────┐ ┌─────▼──────┐ ┌────▼──────┐ ┌──▼──────────────┐
│   API Layer   │ │  MCP Layer │ │  Business │ │  Persistence    │
│               │ │            │ │  Logic    │ │  Layer          │
├───────────────┤ ├────────────┤ ├───────────┤ ├─────────────────┤
│ • upload.rs   │ │quote_tools │ │• pricing  │ │ • sessions.rs   │
│ • quote.rs    │ │   .rs      │ │• file_    │ │ • materials.rs  │
│ • materials.rs│ │            │ │  processor│ │ • models.rs     │
│ • admin.rs    │ │            │ │• templates│ │ • quotes.rs     │
│ • ssr.rs      │ │            │ │           │ │                 │
└───────┬───────┘ └─────┬──────┘ └─────┬─────┘ └────────┬────────┘
        │               │              │                │
        └───────────────┴──────────────┴────────────────┘
                              │
                              │ SQL (sqlx)
                              │
                    ┌─────────▼──────────┐
                    │   PostgreSQL DB    │
                    │                    │
                    │  Tables:           │
                    │  • quote_sessions  │
                    │  • uploaded_models │
                    │  • materials       │
                    │  • quotes          │
                    │  • pricing_history │
                    └────────────────────┘
```

## Couches de l'application

### 1. Couche Présentation

**Emplacement**: `static/js/`, `templates/`

- **Web Components** (Custom Elements):
  - `file-uploader` - Upload de fichiers 3D avec drag & drop
  - `model-viewer` - Visualisation 3D interactive (Three.js)
  - `material-selector` - Sélection de matériaux
  - `quote-summary` - Affichage du devis

- **SSR (Server-Side Rendering)**:
  - Templates Tera pour le rendu côté serveur
  - Pages: index.html, admin.html

### 2. Couche API

**Emplacement**: `src/api/`

- **Handlers** (`src/api/handlers/`):
  - `upload.rs` - Upload de modèles 3D, gestion des sessions
  - `quote.rs` - Génération et récupération de devis
  - `materials.rs` - Catalogue de matériaux
  - `admin.rs` - Gestion admin (matériaux, pricing, cleanup)
  - `ssr.rs` - Rendu des pages HTML

- **Middleware** (`src/api/middleware/`):
  - `auth.rs` - Authentification admin & MCP
  - `rate_limit.rs` - Limitation de débit
  - `sanitize.rs` - Validation et sanitization des entrées
  - `security_headers.rs` - Headers de sécurité
  - `error.rs` - Gestion centralisée des erreurs

- **Routes** (`src/api/routes.rs`):
  - Configuration des endpoints et application des middlewares

### 3. Couche MCP (Model Context Protocol)

**Emplacement**: `src/mcp/`

Interface programmatique pour permettre aux modèles IA d'interagir avec le service.

- **Outils MCP**:
  - `list_materials` - Liste des matériaux disponibles
  - `upload_model` - Upload de modèles en base64
  - `configure_model` - Configuration matériau/quantité
  - `generate_quote` - Génération de devis

- **Transport**: StreamableHTTP (rmcp library)

### 4. Couche Métier

**Emplacement**: `src/business/`

- **file_processor.rs**:
  - Validation de fichiers (STL, 3MF)
  - Extraction de métadonnées (volume, dimensions)
  - Protection anti-zip bomb pour 3MF

- **pricing.rs**:
  - Calcul de prix par modèle
  - Génération de devis avec frais
  - Application du minimum de commande (10€)

- **templates.rs**:
  - Gestion du moteur de templates Tera
  - Rendu des pages SSR

- **session.rs**:
  - Gestion du cycle de vie des sessions
  - Nettoyage des sessions expirées

### 5. Couche Persistance

**Emplacement**: `src/persistence/`

Abstraction de la base de données utilisant sqlx avec prepared statements.

- **sessions.rs** - CRUD sessions
- **models.rs** - CRUD modèles uploadés
- **materials.rs** - CRUD matériaux + pricing history
- **quotes.rs** - CRUD devis

### 6. Couche Base de Données

**Emplacement**: `src/db/`

- **Migrations embarquées** (`src/db/migrations/`):
  - Exécutées automatiquement au démarrage
  - Migration incrémentale avec versioning

- **Connexion**: Pool de connexions PostgreSQL (sqlx)

## Modèles de données

### quote_sessions
```sql
id               VARCHAR(26) PRIMARY KEY (ULID)
expires_at       TIMESTAMP
created_at       TIMESTAMP
```

### uploaded_models
```sql
id               VARCHAR(26) PRIMARY KEY (ULID)
session_id       VARCHAR(26) REFERENCES quote_sessions
filename         VARCHAR(255)
file_format      VARCHAR(10)  -- 'stl' ou '3mf'
file_size_bytes  BIGINT
volume_cm3       DOUBLE PRECISION
dimensions_mm    JSONB        -- {x, y, z}
triangle_count   INT
material_id      VARCHAR(50) REFERENCES materials
file_path        TEXT
preview_url      TEXT
created_at       TIMESTAMP
support_analysis JSONB
```

### materials
```sql
id                  VARCHAR(50) PRIMARY KEY
service_type_id     VARCHAR(50)
name                VARCHAR(100)
description         TEXT
price_per_cm3       DOUBLE PRECISION
color               VARCHAR(7)
properties          JSONB
active              BOOLEAN
created_at          TIMESTAMP
updated_at          TIMESTAMP
```

### quotes
```sql
id              VARCHAR(26) PRIMARY KEY (ULID)
session_id      VARCHAR(26) REFERENCES quote_sessions
total_price     DOUBLE PRECISION
breakdown       JSONB
status          VARCHAR(20)  -- 'pending', 'accepted', 'rejected'
created_at      TIMESTAMP
```

### pricing_history
```sql
id              SERIAL PRIMARY KEY
material_id     VARCHAR(50) REFERENCES materials
old_price       DOUBLE PRECISION
new_price       DOUBLE PRECISION
changed_by      VARCHAR(100)
changed_at      TIMESTAMP
```

## Flux de données principaux

### 1. Génération de devis (User Flow)

```
User Browser
    │
    ├─→ POST /api/sessions
    │       │
    │       └─→ SessionService.create_session()
    │               │
    │               └─→ DB INSERT quote_sessions
    │
    ├─→ POST /api/sessions/{id}/models (multipart/form-data)
    │       │
    │       ├─→ validate_file() → file_processor
    │       │       │
    │       │       └─→ process_stl_file() ou process_3mf_file()
    │       │               │
    │       │               └─→ Extract volume, dimensions, triangle count
    │       │
    │       └─→ DB INSERT uploaded_models
    │
    ├─→ PATCH /api/sessions/{id}/models/{model_id}
    │       │   Body: {"material_id": "pla", "quantity": 1}
    │       │
    │       └─→ DB UPDATE uploaded_models SET material_id
    │
    └─→ POST /api/sessions/{id}/quote
            │
            ├─→ DB SELECT models WHERE session_id
            │
            ├─→ pricing::calculate_model_price() for each model
            │       │
            │       └─→ price = volume_cm3 × material.price_per_cm3
            │
            ├─→ pricing::generate_quote_breakdown()
            │       │
            │       └─→ total = max(subtotal + fees, MINIMUM_ORDER)
            │
            └─→ DB INSERT quotes
```

### 2. Intégration MCP (AI Flow)

```
AI Client
    │
    ├─→ POST /mcp (Bearer: MCP_TOKEN)
    │       │
    │       ├─→ mcp_auth middleware
    │       │
    │       └─→ StreamableHttpService
    │               │
    │               ├─→ list_materials tool
    │               │       │
    │               │       └─→ DB SELECT materials WHERE active=true
    │               │
    │               ├─→ upload_model tool
    │               │       │
    │               │       ├─→ Base64 decode
    │               │       ├─→ validate_file()
    │               │       └─→ DB INSERT uploaded_models
    │               │
    │               ├─→ configure_model tool
    │               │       │
    │               │       └─→ DB UPDATE uploaded_models
    │               │
    │               └─→ generate_quote tool
    │                       │
    │                       ├─→ calculate_model_price()
    │                       └─→ DB INSERT quotes
    │
    └─→ Response (JSON-RPC 2.0)
```

### 3. Administration (Admin Flow)

```
Admin Browser
    │
    ├─→ GET /admin.html
    │       │
    │       └─→ SSR: tera.render("admin.html")
    │
    ├─→ POST /api/admin/login
    │       │   Body: {"token": "..."}
    │       │
    │       ├─→ admin_auth middleware (cookie)
    │       │
    │       └─→ Set-Cookie: admin_token=...
    │
    ├─→ GET /api/admin/materials
    │       │
    │       ├─→ admin_auth middleware
    │       │
    │       └─→ DB SELECT * FROM materials
    │
    ├─→ PATCH /api/admin/materials/{id}
    │       │   Body: {"price_per_cm3": 0.15}
    │       │
    │       ├─→ admin_auth middleware
    │       │
    │       ├─→ DB UPDATE materials
    │       │
    │       └─→ DB INSERT pricing_history
    │
    └─→ POST /api/admin/cleanup
            │
            ├─→ SessionService.cleanup_expired()
            │       │
            │       ├─→ DB DELETE FROM quote_sessions WHERE expires_at < NOW()
            │       │
            │       └─→ fs::remove_dir_all(uploads/{session_id}/)
            │
            └─→ Return CleanupResult
```

## Sécurité

### Authentification

- **Admin**: Cookie-based avec token secret
- **MCP**: Bearer token (Authorization header)
- **Public**: Aucune auth requise pour les endpoints publics

### Protection des entrées

- **Sanitization**: Tous les noms de fichiers et inputs utilisateur
- **Validation**:
  - Format de fichier (magic numbers)
  - Taille de fichier (MAX_FILE_SIZE_MB)
  - Zip bomb protection (3MF)
  - SQL injection: Prepared statements (sqlx)

### Headers de sécurité

- Content-Security-Policy
- X-Frame-Options
- X-Content-Type-Options
- Strict-Transport-Security
- Referrer-Policy

### Rate Limiting

- Global: 100 req/s, burst 500
- Login: 5 req/s, burst 20
- Upload: 10 req/s, burst 50

## Performance

### Optimisations

- **Connection pooling**: Pool PostgreSQL (max 5 connexions)
- **File processing**: Streaming pour les gros fichiers
- **Static assets**: Serveur de fichiers statiques
- **SSR**: Templates compilés (Tera)

### Points à surveiller

- N+1 queries dans la génération de devis (une query par modèle)
- File I/O synchrone (devrait être tokio::fs)
- Pas de cache pour les matériaux
- Pas de CDN pour les assets statiques

## Observabilité

- **Logging**: `tracing` crate avec niveaux configurables
- **Métriques**: Aucune (TODO: Prometheus)
- **Tracing distribué**: Aucun (TODO: OpenTelemetry)

## Déploiement

### Environnements

- **Développement**: `cargo run` avec RUST_LOG=debug
- **Production**: Docker Compose ou build release

### Configuration requise

- Rust 1.75+
- PostgreSQL 14+
- 512 MB RAM minimum
- 1 GB stockage pour uploads

## Évolutions futures

- JWT pour l'authentification admin
- Système de webhooks pour export de devis
- Export PDF des devis
- Multi-langue (i18n)
- Cache Redis pour les matériaux
- File I/O asynchrone (tokio::fs)
- Métriques Prometheus
- Tracing OpenTelemetry
