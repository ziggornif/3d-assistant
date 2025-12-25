# 3D Print Quote Service

Service de devis instantané pour impression 3D. Téléchargez vos fichiers STL/3MF, configurez les matériaux et obtenez un devis détaillé en temps réel.

## Fonctionnalités

### Métier
- **Upload de fichiers 3D** : Support STL et 3MF
- **Visualisation 3D** : Prévisualisation interactive avec Three.js (rotation, zoom, pan)
- **Sélection de matériaux** : PLA, ABS, PETG, Résine avec prix personnalisés
- **Devis instantané** : Calcul automatique basé sur le volume et le matériau
- **Interface admin** : Gestion des prix et matériaux sans code
- **MCP (Model Context Protocol)** : API programmatique pour IA et outils d'automatisation
- **Nettoyage automatique** : Suppression des sessions expirées et fichiers uploadés

### Production-Ready
- **OpenTelemetry Observability** : Traces distribuées, métriques métier/techniques, logs structurés
- **Health Checks** : Liveness (`/health`) et readiness (`/ready`) pour Kubernetes/Docker
- **Configuration Production** : Variables d'environnement, validation fail-fast, secrets sécurisés
- **Docker Compose** : Stack complète avec PostgreSQL, application et observabilité (SigNoz)
- **Documentation Déploiement** : Guides complets VPS, Docker, sécurité, troubleshooting

### Qualité & Accessibilité
- **Accessibilité** : Conformité RGAA/WCAG 2.1 AA
- **Sécurité** : Rate limiting, CORS, validation stricte, protection XSS/SQL injection
- **Tests** : 46+ tests E2E Playwright, tests unitaires Rust

## Architecture

```
3d-assistant/
├── src/                    # Code Rust (API + SSR)
│   ├── api/               # Routes et handlers
│   │   └── health.rs      # Health checks (/health, /ready)
│   ├── models/            # Entités (Material, Quote, Session)
│   ├── business/          # Logique métier (pricing, file processing, templates)
│   ├── observability/     # OpenTelemetry (traces, metrics, logs)
│   ├── db/                # Database (migrations, seed)
│   ├── mcp/               # Model Context Protocol
│   └── config.rs          # Configuration avec validation
├── static/                # Assets web (JS + CSS)
│   ├── js/
│   │   ├── components/   # Web Components (file-uploader, model-viewer, etc.)
│   │   ├── services/     # API client, session manager
│   │   └── utils/        # Formatters, accessibilité
│   └── css/              # Styles (main, accessibility, components)
├── templates/             # Templates SSR (Tera)
├── deployment/            # Déploiement production
│   ├── docker-compose.observability.yml  # Stack SigNoz pour dev
│   ├── docker-compose.prod.yml           # Production complète
│   └── otel-collector-config.yml         # Config OpenTelemetry
├── docs/
│   ├── deployment/        # Guides de déploiement
│   │   ├── docker-compose.md  # Déploiement Docker
│   │   ├── vps.md             # Déploiement VPS/VM
│   │   ├── security.md        # Checklist sécurité
│   │   └── troubleshooting.md # Dépannage
│   └── api/               # Documentation API
│       └── health-checks.md   # Health checks K8s
├── e2e/                   # Tests E2E Playwright
└── uploads/               # Fichiers uploadés
```

## Prérequis

- **Rust** 1.75+ (`rustup install stable`)
- **Node.js** 22+ (recommandé 24+)
- **pnpm** 10+

## Installation

### Méthode 1: Installation locale (développement)

#### Prérequis
- Rust 1.75+ et Cargo
- PostgreSQL 14+
- Node.js 22+ et pnpm 10+ (pour les tests E2E uniquement)

#### Étapes

**1. Cloner le projet**

```bash
git clone https://github.com/ziggornif/3d-assistant.git
cd 3d-assistant
```

**2. Configurer PostgreSQL**

```bash
# Créer la base de données
createdb quotes

# Ou avec psql
psql -c "CREATE DATABASE quotes;"
```

**3. Configurer les variables d'environnement**

```bash
# Copier le fichier d'exemple
cp .env.example .env

# Éditer .env avec vos valeurs
# IMPORTANT: Changez ADMIN_TOKEN et MCP_TOKEN !
nano .env  # ou votre éditeur préféré
```

Exemple de `.env` pour développement local:
```env
DATABASE_URL=postgres://postgres:password@localhost:5432/quotes
ADMIN_TOKEN=$(openssl rand -base64 32)
MCP_TOKEN=$(openssl rand -base64 32)
HOST=127.0.0.1
PORT=3000
```

**4. Initialiser la base de données**

La base est initialisée automatiquement au premier démarrage (migrations embarquées).

**5. Compiler et lancer**

```bash
# Mode développement (avec logs détaillés)
RUST_LOG=debug cargo run

# Ou mode release (plus rapide)
cargo build --release
./target/release/quote-service
```

Le serveur démarre sur `http://127.0.0.1:3000`.

**6. Vérifier l'installation**

- Interface utilisateur: http://127.0.0.1:3000/
- Interface admin: http://127.0.0.1:3000/admin (utilisez votre ADMIN_TOKEN)
- Health check: http://127.0.0.1:3000/health
- Readiness check: http://127.0.0.1:3000/ready

### Méthode 2: Docker Compose (production)

**Prérequis**: Docker et Docker Compose installés

```bash
# 1. Cloner le projet
git clone https://github.com/ziggornif/3d-assistant.git
cd 3d-assistant

# 2. Configurer l'environnement production
cp deployment/.env.production.example deployment/.env
# ⚠️ IMPORTANT: Éditez deployment/.env et changez tous les secrets

# 3. Démarrer la stack complète (app + db + observabilité)
docker-compose -f deployment/docker-compose.prod.yml --profile observability up -d

# OU démarrer sans observabilité (plus léger)
docker-compose -f deployment/docker-compose.prod.yml up -d postgres app

# 4. Vérifier le statut
docker-compose -f deployment/docker-compose.prod.yml ps
curl http://localhost:8000/ready

# 5. Accéder aux services
# - Application: http://localhost:8000
# - SigNoz (observabilité): http://localhost:3301

# 6. Voir les logs
docker-compose -f deployment/docker-compose.prod.yml logs -f app

# 7. Arrêter les services
docker-compose -f deployment/docker-compose.prod.yml down
```

**Stack d'observabilité locale (développement)**:

```bash
# Démarrer SigNoz seulement (sans l'app)
docker-compose -f deployment/docker-compose.observability.yml up -d

# Lancer l'app localement avec OTEL
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 cargo run

# SigNoz UI: http://localhost:3301
```

**Documentation complète**: Voir [docs/deployment/](docs/deployment/README.md)

### 3. Tests

```bash
# Tests unitaires Rust
cargo test

# Tests E2E / API (Playwright)
pnpm install  # Si pas déjà fait
pnpm test:e2e

# Lint JavaScript
pnpm lint
```

## Configuration

### Variables d'environnement (.env)

Toutes les variables ci-dessous doivent être définies dans un fichier `.env` à la racine du projet.

#### Obligatoires

```env
# Base de données PostgreSQL
DATABASE_URL=postgres://user:password@localhost:5432/quotes
# Format: postgres://USER:PASSWORD@HOST:PORT/DATABASE
# Exemple: postgres://quote_user:secret@localhost:5432/quotes_prod

# Token d'authentification admin (interface /admin.html)
ADMIN_TOKEN=your-secure-admin-token-here
# ⚠️ IMPORTANT: Changez cette valeur en production !
# Générez un token sécurisé: openssl rand -base64 32

# Token d'authentification MCP (API programmatique /mcp)
MCP_TOKEN=your-secure-mcp-token-here
# ⚠️ IMPORTANT: Changez cette valeur en production !
# Générez un token sécurisé: openssl rand -base64 32
```

#### Optionnelles (avec valeurs par défaut)

```env
# Environnement
ENVIRONMENT=development     # Environnement: development ou production

# Serveur
HOST=127.0.0.1              # Adresse d'écoute (défaut: 127.0.0.1)
PORT=3000                   # Port du serveur (défaut: 3000)

# Uploads de fichiers 3D
MAX_FILE_SIZE_MB=50         # Taille max par fichier (défaut: 50 MB)
UPLOAD_DIR=./uploads        # Répertoire de stockage (défaut: ./uploads)

# Gestion des sessions
SESSION_EXPIRY_HOURS=24     # Durée de vie des sessions (défaut: 24h)

# OpenTelemetry (observabilité)
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317  # Endpoint OTLP collector
OTEL_SERVICE_NAME=quote-service                    # Nom du service

# Logging (optionnel)
RUST_LOG=info              # Niveau de logs: error, warn, info, debug, trace
```

#### Exemple complet (.env.example)

Un fichier `.env.example` est fourni avec des valeurs de démonstration. **Ne l'utilisez jamais en production** sans changer les tokens.

```bash
cp .env.example .env
# Éditez .env et changez ADMIN_TOKEN et MCP_TOKEN
```

## Déploiement Production

### Options de déploiement

1. **Docker Compose** (recommandé) - [Guide complet](docs/deployment/docker-compose.md)
   - Stack complète avec PostgreSQL et observabilité
   - Configuration via variables d'environnement
   - Health checks et resource limits

2. **VPS/VM Ubuntu** - [Guide complet](docs/deployment/vps.md)
   - Déploiement traditionnel avec systemd
   - Nginx reverse proxy avec SSL/TLS
   - Backups automatisés

3. **Kubernetes** - Documentation à venir
   - Manifests avec liveness/readiness probes
   - Auto-scaling basé sur les métriques

### Checklist de sécurité

Avant de déployer en production:
- ✅ Tokens changés (`ADMIN_TOKEN`, `MCP_TOKEN`)
- ✅ Base de données PostgreSQL sécurisée
- ✅ HTTPS activé (Let's Encrypt)
- ✅ Firewall configuré
- ✅ Backups automatisés configurés
- ✅ Monitoring activé (SigNoz ou autre)

**Documentation complète**: [docs/deployment/security.md](docs/deployment/security.md)

## Observabilité

L'application inclut une observabilité complète via OpenTelemetry:

### Traces distribuées
- Toutes les requêtes HTTP automatiquement tracées
- Corrélation entre logs et traces
- Visualisation dans SigNoz

### Métriques
**Business**:
- `quotes_generated_total` - Nombre de devis générés
- `models_uploaded_total` - Modèles uploadés
- `quote_calculation_duration_ms` - Temps de calcul
- `file_upload_size_bytes` - Taille des fichiers

**Techniques**:
- `http_request_duration_ms` - Latence des requêtes
- `db_connections_active` - Connexions DB actives
- `http_requests_total` - Requêtes totales

### Logs structurés
- Format JSON en production (avec trace_id)
- Format pretty en développement
- Niveaux configurables via `RUST_LOG`

**Accéder à SigNoz**: http://localhost:3301 (après `docker-compose up`)

## API Endpoints

### Sessions
- `POST /api/sessions` - Créer une session
- `POST /api/sessions/{id}/models` - Upload d'un modèle 3D
- `DELETE /api/sessions/{id}/models/{model_id}` - Supprimer un modèle

### Matériaux
- `GET /api/materials` - Liste des matériaux disponibles

### Devis
- `GET /api/sessions/{id}/quote` - Obtenir le devis actuel
- `POST /api/sessions/{id}/quote` - Générer et sauvegarder le devis
- `PATCH /api/sessions/{id}/models/{model_id}` - Configurer un modèle (matériau, quantité)

### Admin (requiert Bearer token)
- `GET /api/admin/materials` - Liste tous les matériaux
- `POST /api/admin/materials` - Créer un matériau
- `PATCH /api/admin/materials/{id}` - Mettre à jour un matériau
- `GET /api/admin/pricing-history` - Historique des changements de prix
- `POST /api/admin/cleanup` - Nettoyer les sessions expirées et fichiers uploadés

### Health Checks (Kubernetes/Docker)
- `GET /health` - Liveness probe (service est-il up?)
- `GET /ready` - Readiness probe (service peut-il traiter des requêtes?)

**Documentation**: [docs/api/health-checks.md](docs/api/health-checks.md)

### MCP (Model Context Protocol)
- `POST /mcp` - Endpoint MCP pour accès programmatique

**Outils MCP disponibles :**
- `list_materials` - Lister les matériaux et prix
- `upload_model` - Uploader un modèle 3D (base64)
- `configure_model` - Configurer un modèle avec matériau/quantité
- `generate_quote` - Générer un devis complet

**Documentation :** [Français](docs/fr/mcp.md) | [English](docs/en/mcp.md)

## Interface Admin

Accédez à `/admin.html` et utilisez votre token admin pour :
- Voir et modifier les prix des matériaux
- Ajouter de nouveaux matériaux
- Activer/Désactiver des matériaux
- Consulter l'historique des prix

## Sécurité

- **Rate Limiting** : 100 req/s global, burst 500
- **Input Sanitization** : Protection contre directory traversal, XSS, validation des entrées
- **SQL Injection** : Requêtes paramétrées via SQLx
- **CORS** : Configuration stricte
- **Admin Auth** : Token Bearer simple (JWT en production)

## Accessibilité (RGAA/WCAG)

- Skip link pour navigation clavier
- Focus indicators visibles (3px outline)
- ARIA labels sur tous les éléments interactifs
- Support `prefers-reduced-motion`
- Contrast ratio minimum 4.5:1
- Min touch targets 44x44px
- Screen reader announcements (`aria-live`)

## Tests

- **Tests Rust** : Validation fichiers (STL/3MF), pricing, sanitization, auth
- **46 tests E2E Playwright** : Contrats API, admin, sessions, materials, quotes, cleanup, 3MF upload, UI

## Formule de Prix

```
prix_modèle = volume_cm³ × prix_matériau_par_cm³
sous_total = Σ(prix_modèle × quantité)
frais = 2.00€ (frais de base)
total = max(sous_total + frais, 10.00€)  // Minimum 10€
```

## Développement

### Linter & Formatter

```bash
# Rust
cargo clippy
cargo fmt

# Frontend
pnpm lint
pnpm format
```

### Ajouter un matériau

Via API admin ou directement dans `src/db/seed.sql`.

### Structure des Web Components

Chaque composant (file-uploader, model-viewer, material-selector, quote-summary) :
- Étend `HTMLElement`
- Utilise Shadow DOM pour encapsulation
- Émet des événements custom pour communication
- Supporte les attributs ARIA pour accessibilité

## Roadmap

### ✅ Réalisé
- [x] Support fichiers STL et 3MF
- [x] MCP (Model Context Protocol) pour IA
- [x] OpenTelemetry observability (traces, metrics, logs)
- [x] Health checks Kubernetes (liveness/readiness)
- [x] Documentation déploiement complète
- [x] Docker Compose production-ready
- [x] Configuration production avec validation
- [x] Nettoyage automatique sessions

### 🚧 En cours
- [ ] Phase 4: Instrumentation métier complète (traces custom sur operations)

### 📋 Planifié
- [ ] Export PDF des devis
- [ ] Système de webhooks (Notion, Obsidian, Odoo)
- [ ] Notifications email
- [ ] Multi-langue (i18n)
- [ ] OAuth/JWT pour admin
- [ ] Kubernetes manifests et Helm charts
- [ ] Dashboards Grafana pré-configurés

## Licence

MIT

## Auteur

3D Print Quote Service Team
