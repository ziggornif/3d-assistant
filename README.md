# 3D Print Quote Service

Service de devis instantané pour impression 3D. Téléchargez vos fichiers STL/3MF, configurez les matériaux et obtenez un devis détaillé en temps réel.

## Fonctionnalités

- **Upload de fichiers 3D** : Support STL et 3MF
- **Visualisation 3D** : Prévisualisation interactive avec Three.js (rotation, zoom, pan)
- **Sélection de matériaux** : PLA, ABS, PETG, Résine avec prix personnalisés
- **Devis instantané** : Calcul automatique basé sur le volume et le matériau
- **Interface admin** : Gestion des prix et matériaux sans code
- **Nettoyage automatique** : Suppression des sessions expirées et fichiers uploadés
- **Accessibilité** : Conformité RGAA/WCAG 2.1 AA

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

### 1. Cloner le projet

```bash
git clone <repo-url>
cd 3d-assistant
```

### 2. Application (Rust + SSR)

```bash
# Copier la configuration
cp .env.example .env

# Éditer .env si nécessaire (port, upload dir, admin token)
# ADMIN_TOKEN=votre-token-secret

# Compiler et lancer
cargo build --release
cargo run
```

Le serveur démarre sur `http://127.0.0.1:3000` avec SSR (Server-Side Rendering). L'application est accessible directement à cette adresse.

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

```env
# Base de données PostgreSQL
DATABASE_URL=postgres://user:password@localhost:5432/quotes

# Serveur
HOST=127.0.0.1
PORT=3000

# Uploads
MAX_FILE_SIZE_MB=50
UPLOAD_DIR=./uploads

# Sessions
SESSION_EXPIRY_HOURS=24

# Admin
ADMIN_TOKEN=admin-secret-token-2025
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
- [ ] MCP (Model Context Protocol) pour permettre à un modèle IA de faire un devis sans utiliser le front
- [ ] Système de webhooks pour exporter les devis vers des plateformes externes (Notion, Obsidian, Odoo, etc.)
- [ ] Export PDF des devis
- [ ] Notifications email
- [ ] Multi-langue (i18n)
- [ ] OAuth/JWT pour admin

## Licence

MIT

## Auteur

3D Print Quote Service Team
