# 3D Print Quote Service

Service de devis instantané pour impression 3D. Téléchargez vos fichiers STL/3MF, configurez les matériaux et obtenez un devis détaillé en temps réel.

## Fonctionnalités

- **Upload de fichiers 3D** : Support STL et 3MF
- **Visualisation 3D** : Previsualisation interactive avec Three.js (rotation, zoom, pan)
- **Selection de materiaux** : PLA, ABS, PETG, Resine avec prix personnalises
- **Devis instantane** : Calcul automatique base sur le volume et le materiau
- **Comptes utilisateurs** : Inscription, connexion, validation admin, historique des devis
- **Mode demo** : Les visiteurs peuvent tester l'upload et la visualisation sans compte (generation de devis reservee aux comptes valides)
- **Interface admin** : Gestion des prix, materiaux et comptes utilisateurs
- **MCP (Model Context Protocol)** : API programmatique pour IA et outils d'automatisation
- **Nettoyage automatique** : Suppression des sessions expirees et fichiers uploades
- **Accessibilite** : Conformite RGAA/WCAG 2.1 AA

## Architecture

```
3d-assistant/
├── src/               # Code Rust (API + SSR)
│   ├── api/          # Routes et handlers
│   ├── models/       # Entités (Material, Quote, Session)
│   └── services/     # Logique métier (pricing, file processing, templates)
├── static/           # Assets web (JS + CSS)
│   ├── js/
│   │   ├── components/  # Web Components (file-uploader, model-viewer, etc.)
│   │   ├── services/    # API client, session manager
│   │   └── utils/       # Formatters, accessibilité
│   └── css/          # Styles (main, accessibility, components)
├── templates/        # Templates SSR (Tera)
├── e2e/              # Tests E2E Playwright
└── uploads/          # Fichiers uploadés
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
- Interface admin: http://127.0.0.1:3000/admin.html (utilisez votre ADMIN_TOKEN)
- Health check: http://127.0.0.1:3000/health

### Méthode 2: Docker Compose (production-like)

**Prérequis**: Docker et Docker Compose installés

```bash
# 1. Cloner le projet
git clone https://github.com/ziggornif/3d-assistant.git
cd 3d-assistant

# 2. Configurer les variables d'environnement
cp .env.example .env
# ⚠️ Éditez .env et changez ADMIN_TOKEN et MCP_TOKEN

# 3. Démarrer les services (PostgreSQL + Application)
docker compose up -d

# 4. Vérifier les logs
docker compose logs -f

# 5. Arrêter les services
docker compose down
```

L'application sera accessible sur `http://localhost:3000`.

**Note**: Le `docker-compose.yml` utilise des mots de passe de démonstration pour PostgreSQL. Changez-les en production.

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
# Serveur
HOST=127.0.0.1              # Adresse d'écoute (défaut: 127.0.0.1)
PORT=3000                   # Port du serveur (défaut: 3000)

# Uploads de fichiers 3D
MAX_FILE_SIZE_MB=50         # Taille max par fichier (défaut: 50 MB)
UPLOAD_DIR=./uploads        # Répertoire de stockage (défaut: ./uploads)

# Gestion des sessions
SESSION_EXPIRY_HOURS=24     # Durée de vie des sessions (défaut: 24h)

# Logging (optionnel)
RUST_LOG=info              # Niveau de logs: error, warn, info, debug, trace
```

#### Exemple complet (.env.example)

Un fichier `.env.example` est fourni avec des valeurs de démonstration. **Ne l'utilisez jamais en production** sans changer les tokens.

```bash
cp .env.example .env
# Éditez .env et changez ADMIN_TOKEN et MCP_TOKEN
```

### Docker Compose

```bash
docker compose up -d
```

Cela démarre PostgreSQL + l'application avec les variables d'environnement configurées.

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

- [x] Support fichiers 3MF
- [x] MCP (Model Context Protocol) pour permettre à un modèle IA de faire un devis sans utiliser le front
- [ ] Système de webhooks pour exporter les devis vers des plateformes externes (Notion, Obsidian, Odoo, etc.)
- [ ] Export PDF des devis
- [ ] Notifications email
- [ ] Multi-langue (i18n)
- [ ] OAuth/JWT pour admin

## Licence

MIT

## Auteur

3D Print Quote Service Team
