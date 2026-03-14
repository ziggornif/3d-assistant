# Architecture Spec -- Feature 0001 : Gestion de comptes multi-tenants

## Overview

Ajout d'un systeme d'authentification utilisateur (inscription, connexion, sessions) et d'une gestion de comptes admin (validation, desactivation) sur le codebase existant. L'application passe d'un mode sessions anonymes a un mode mixte : visiteurs en mode demo (upload sans devis) et utilisateurs authentifies avec historique de devis. L'isolation des donnees se fait par `user_id` : chaque utilisateur ne voit que ses propres devis. En V2, les prix personnalises seront egalement lies au `user_id`.

## Impact Assessment

### Fichiers impactes

| Fichier | Type de modification | Risque |
|---------|---------------------|--------|
| `src/db/migrations/` | Nouvelles migrations (007, 008) | Faible -- ajout pur |
| `src/db/schema.rs` | Ajout constantes tables | Faible |
| `src/models/` | Nouveau module `user.rs` | Faible -- ajout pur |
| `src/persistence/` | Nouveau module `users.rs` | Faible -- ajout pur |
| `src/persistence/sessions.rs` | Ajout colonne `user_id` aux queries | Moyen -- modification existant |
| `src/persistence/quotes.rs` | Ajout query `find_by_user` | Faible -- ajout |
| `src/business/` | Nouveau module `auth.rs` | Faible -- ajout pur |
| `src/business/session.rs` | Liaison session/user, expiration conditionnelle | Moyen -- modification existant |
| `src/api/middleware/auth.rs` | Nouveau middleware `user_auth` | Moyen -- ajout a cote de l'existant |
| `src/api/middleware/error.rs` | Nouvelles variantes AppError | Faible |
| `src/api/handlers/` | Nouveaux : `auth.rs`, modifications : `upload.rs`, `quote.rs`, `ssr.rs` | Moyen |
| `src/api/routes.rs` | Nouvelles routes auth + user + admin accounts | Moyen |
| `src/config.rs` | Ajout config password hashing | Faible |
| `templates/` | Nouveaux : `login.html`, `register.html`, `my-quotes.html` ; modifies : `base.html`, `index.html`, `admin.html` | Moyen |
| `static/js/` | Nouveau : `services/auth-manager.js` ; modifies : `main.js`, `services/api-client.js`, `services/session-manager.js` | Moyen |
| `Cargo.toml` | Nouvelles dependances | Faible |

### Dependances a ajouter (Cargo.toml)

| Crate | Version | Usage |
|-------|---------|-------|
| `argon2` | `0.5` | Hashing de mots de passe (Argon2id, recommande OWASP) |
| `rand` | `0.8` | Generation de salt et tokens de session |

**Justification du choix Argon2** : Argon2id est le standard recommande par OWASP pour le hashing de mots de passe. Alternative ecartee : bcrypt (plus ancien, pas de resistance memoire).

## Data Model

### Nouvelle table : `users`

```sql
-- Migration: 007_users.sql
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,           -- ULID
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,             -- Argon2id hash
    display_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',  -- pending | active | disabled | rejected
    role TEXT NOT NULL DEFAULT 'user',       -- user | admin
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
```

### Modification table : `quote_sessions`

```sql
-- Migration: 008_sessions_user_id.sql
ALTER TABLE quote_sessions ADD COLUMN user_id TEXT REFERENCES users(id);
ALTER TABLE quote_sessions ADD COLUMN session_type TEXT NOT NULL DEFAULT 'anonymous';
-- session_type: 'anonymous' (demo, expire 24h) | 'authenticated' (lie a un user, expire 30j)

CREATE INDEX IF NOT EXISTS idx_quote_sessions_user_id ON quote_sessions(user_id);
```

**Note** : Les sessions existantes (sans user_id) restent valides et continuent de fonctionner en mode anonymous/demo. Pas de migration destructive.

### Modele Rust : `User`

```
User {
    id: String,              // ULID
    email: String,
    password_hash: String,   // Argon2id
    display_name: String,
    status: String,          // "pending" | "active" | "disabled" | "rejected"
    role: String,            // "user" | "admin"
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}
```

### Modele Rust modifie : `QuoteSession`

```
QuoteSession {
    id: String,
    user_id: Option<String>,       // NOUVEAU - None pour demo
    session_type: String,          // NOUVEAU - "anonymous" | "authenticated"
    created_at: NaiveDateTime,
    expires_at: NaiveDateTime,
    status: String,
}
```

## API Contracts

### Authentification

#### POST /api/auth/register
- Auth: none
- Request body:
```json
{
    "email": "user@example.com",
    "password": "SecurePass123",
    "display_name": "Jean Dupont"
}
```
- Response 201:
```json
{
    "user_id": "01JCV8...",
    "email": "user@example.com",
    "display_name": "Jean Dupont",
    "status": "pending",
    "message": "Compte cree, en attente de validation par l'administrateur"
}
```
- Response 400: validation error (email invalide, mot de passe faible)
- Response 409: email deja utilise

#### POST /api/auth/login
- Auth: none
- Rate limiting: 5 req/min (protection brute force)
- Request body:
```json
{
    "email": "user@example.com",
    "password": "SecurePass123"
}
```
- Response 200:
```json
{
    "user_id": "01JCV8...",
    "email": "user@example.com",
    "display_name": "Jean Dupont",
    "role": "user",
    "session_id": "01JCV9..."
}
```
- Headers set: `Set-Cookie: user_session=<token>; HttpOnly; SameSite=Lax; Path=/; Max-Age=2592000`
- Response 401: "Email ou mot de passe incorrect" (message generique, pas de distinction email/password)
- Response 403: "Compte en attente de validation" (status=pending) ou "Compte desactive" (status=disabled)

#### POST /api/auth/logout
- Auth: cookie user_session
- Response 200: `{ "message": "Deconnecte" }`
- Headers set: suppression du cookie user_session

#### GET /api/auth/me
- Auth: cookie user_session
- Response 200:
```json
{
    "user_id": "01JCV8...",
    "email": "user@example.com",
    "display_name": "Jean Dupont",
    "role": "user",
    "status": "active"
}
```
- Response 401: non authentifie

### Devis utilisateur

#### GET /api/users/me/quotes
- Auth: cookie user_session (user actif)
- Query params: `?page=1&per_page=20`
- Response 200:
```json
{
    "quotes": [
        {
            "id": "01JCV8...",
            "session_id": "01JCV9...",
            "total_price": 15.50,
            "status": "generated",
            "model_count": 2,
            "created_at": "2026-03-14T10:30:00"
        }
    ],
    "total": 42,
    "page": 1,
    "per_page": 20
}
```

#### GET /api/users/me/quotes/{quote_id}
- Auth: cookie user_session (user actif, proprietaire du devis)
- Response 200: detail complet du devis (breakdown, modeles, materiaux)
- Response 404: devis non trouve ou pas proprietaire

### Administration des comptes

#### GET /api/admin/users
- Auth: cookie admin_token (admin)
- Query params: `?status=pending&page=1&per_page=20`
- Response 200:
```json
{
    "users": [
        {
            "id": "01JCV8...",
            "email": "user@example.com",
            "display_name": "Jean Dupont",
            "status": "pending",
            "created_at": "2026-03-14T09:00:00",
            "quote_count": 0
        }
    ],
    "total": 5,
    "page": 1,
    "per_page": 20
}
```

#### PATCH /api/admin/users/{user_id}
- Auth: cookie admin_token (admin)
- Request body:
```json
{
    "status": "active"   // "active" | "disabled" | "rejected"
}
```
- Response 200:
```json
{
    "id": "01JCV8...",
    "email": "user@example.com",
    "display_name": "Jean Dupont",
    "status": "active",
    "updated_at": "2026-03-14T10:00:00"
}
```
- Response 404: utilisateur non trouve

### Modifications aux endpoints existants

#### POST /api/sessions (modifie)
- Comportement change :
  - Si cookie `user_session` valide (user actif) : cree une session `authenticated` liee au user_id, expiration 30 jours
  - Si pas de cookie ou user non actif : cree une session `anonymous`, expiration 24h (comportement actuel)

#### POST /api/sessions/{session_id}/quote (modifie)
- Comportement change :
  - Si session `authenticated` : genere le devis normalement
  - Si session `anonymous` : retourne 403 "Inscrivez-vous pour generer un devis"

#### Pages SSR (modifie)

| Route | Modification |
|-------|-------------|
| `GET /` | Detecte l'auth user. Si connecte : affiche nom + bouton deconnexion + lien "Mes devis". Si visiteur : affiche boutons "Connexion" / "Inscription" |
| `GET /login` | Nouvelle page SSR : formulaire connexion |
| `GET /register` | Nouvelle page SSR : formulaire inscription |
| `GET /my-quotes` | Nouvelle page SSR : historique des devis de l'utilisateur connecte. Redirige vers /login si non connecte |
| `GET /admin` | Ajoute section "Gestion des comptes" apres authentification admin |

## Architecture des sessions utilisateur

### Strategie d'authentification

Cookie-based session auth (coherent avec le pattern admin existant). Pas de JWT.

**Justification** : L'application est SSR avec Tera. Les cookies HttpOnly sont le pattern naturel pour ce type d'app. JWT serait over-engineered pour du server-side rendering sans API publique tierce.

### Flow d'authentification

```
1. POST /api/auth/login
   -> Verifie email + password (Argon2 verify)
   -> Verifie status == "active"
   -> Genere un token de session (random 32 bytes, base64)
   -> Stocke en cookie HttpOnly (user_session)
   -> Cree une quote_session liee au user

2. Middleware user_auth (pour routes protegees)
   -> Lit cookie user_session
   -> Verifie token en base (table user_sessions ou in-memory)
   -> Injecte user_id dans la request via Extension

3. POST /api/auth/logout
   -> Supprime le cookie
   -> Invalide le token cote serveur
```

### Table de sessions utilisateur

```sql
-- Ajouter dans migration 007 ou 008
CREATE TABLE IF NOT EXISTS user_sessions (
    token TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id),
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires ON user_sessions(expires_at);
```

## Mode demo (visiteurs)

Le mode demo est l'evolution du comportement actuel :

1. Le visiteur arrive sur `/` sans cookie `user_session`
2. Une session anonymous est creee (comme avant)
3. Il peut uploader, voir la visualisation 3D, selectionner des materiaux
4. Le bouton "Generer le devis" est remplace par un CTA "Inscrivez-vous pour obtenir votre devis"
5. L'endpoint `POST /api/sessions/{id}/quote` refuse les sessions anonymous (403)

**Pas de breaking change** sur l'upload ou la visualisation. Le seul changement visible est le blocage de la generation de devis pour les visiteurs.

## Breaking Changes

**Non** -- Il n'y a pas de breaking change au sens strict :
- Les sessions existantes en base (sans user_id) continuent de fonctionner comme sessions anonymous
- La migration `ALTER TABLE ADD COLUMN` est non-destructive (colonne nullable)
- L'API existante reste fonctionnelle : les endpoints de session, upload, materiaux gardent le meme comportement
- Le seul changement de comportement est `POST /api/sessions/{id}/quote` qui refuse les sessions anonymous. C'est un changement fonctionnel voulu, pas un breaking change API (l'endpoint n'a pas de consommateurs externes).

**Point d'attention** : L'endpoint MCP `generate_quote` utilise des sessions. Il faudra s'assurer que les sessions creees via MCP sont de type `authenticated` (le MCP est deja protege par token). Alternative : exempter les sessions MCP de la restriction demo. Decision recommandee : les sessions MCP restent `anonymous` mais exemptees de la restriction de devis (le MCP est un canal machine-to-machine authentifie).

## Plan d'implementation

### Backend (ordonne)

| Etape | Description | Complexite |
|-------|-------------|------------|
| 1 | Ecrire les migrations SQL (007_users.sql, 008_sessions_user_id.sql) | Faible |
| 2 | Creer le modele `User` + `UserSession` dans `src/models/user.rs` | Faible |
| 3 | Creer la couche persistence `src/persistence/users.rs` (CRUD user, find_by_email, update_status) | Faible |
| 4 | Creer la couche persistence `src/persistence/user_sessions.rs` (create, find_by_token, delete) | Faible |
| 5 | Creer le service `src/business/auth.rs` (register, login, verify_session, logout, password hashing/verify) | Moyenne |
| 6 | Modifier `src/business/session.rs` : liaison user_id, expiration conditionnelle | Moyenne |
| 7 | Creer le middleware `user_auth` dans `src/api/middleware/auth.rs` | Moyenne |
| 8 | Creer les handlers `src/api/handlers/auth.rs` (register, login, logout, me) | Moyenne |
| 9 | Creer les handlers pour devis utilisateur dans `src/api/handlers/user_quotes.rs` | Faible |
| 10 | Ajouter les handlers admin comptes dans `src/api/handlers/admin.rs` | Faible |
| 11 | Modifier `src/api/handlers/upload.rs` : creation session liee au user si authentifie | Faible |
| 12 | Modifier `src/api/handlers/quote.rs` : bloquer generation devis pour sessions anonymous | Faible |
| 13 | Mettre a jour `src/api/routes.rs` : nouvelles routes | Faible |
| 14 | Modifier `src/api/handlers/ssr.rs` : nouvelles pages + contexte auth | Moyenne |
| 15 | Ajouter les variantes AppError (Unauthorized, Forbidden, Conflict, WeakPassword) | Faible |
| 16 | Modifier `src/models/quote.rs` : champs user_id et session_type sur QuoteSession | Faible |
| 17 | Modifier `src/persistence/sessions.rs` : queries avec user_id | Faible |

### Frontend (ordonne)

| Etape | Description | Complexite |
|-------|-------------|------------|
| 1 | Creer `templates/login.html` et `templates/register.html` | Faible |
| 2 | Creer `templates/my-quotes.html` | Faible |
| 3 | Modifier `templates/base.html` : header conditionnel (connecte/visiteur) | Faible |
| 4 | Modifier `templates/index.html` : CTA inscription au lieu du bouton devis pour visiteurs | Faible |
| 5 | Modifier `templates/admin.html` : section gestion des comptes | Moyenne |
| 6 | Creer `static/js/services/auth-manager.js` | Faible |
| 7 | Modifier `static/js/main.js` : detecter l'etat auth, conditionner le bouton devis | Faible |
| 8 | Creer `static/js/admin/accounts.js` : gestion des comptes admin | Moyenne |
| 9 | CSS pour les nouvelles pages (login, register, my-quotes, admin accounts) | Faible |

## Evolution V2 (prix personnalises par utilisateur)

En V2, chaque utilisateur pourra avoir ses propres prix. Le discriminant sera le `user_id` (pas de notion de tenant/organisation). L'evolution V2 consistera a :

1. Creer une table `user_materials (user_id, material_id, custom_price_per_cm3)`
2. Modifier la logique de pricing pour chercher d'abord le prix utilisateur, puis fallback sur le prix global
3. Ajouter une interface admin pour configurer les prix par utilisateur

Aucune preparation structurelle n'est necessaire en V1 : le `user_id` est deja present sur les sessions et les devis, ce qui suffit comme point d'ancrage pour la V2.

## Risks & Constraints

| Risque | Impact | Mitigation |
|--------|--------|------------|
| Securite des mots de passe | Critique | Argon2id avec parametres OWASP (memory=19456 KiB, iterations=2, parallelism=1) |
| Timing attacks sur login | Haute | Toujours executer le hash verify meme si l'email n'existe pas (constant-time) |
| Brute force login | Haute | Rate limiting 5 req/min sur /api/auth/login |
| Enumeration d'emails via register | Moyenne | Message generique "Un email de confirmation a ete envoye" (meme si email existe). Comme on n'a pas d'email en V1, alternative : rate limiting agressif sur register |
| Sessions MCP brisees | Haute | Exempter les sessions MCP de la restriction demo |
| Migration DB non reversible | Faible | ALTER TABLE ADD COLUMN est non-destructif, les donnees existantes ne sont pas touchees |

## Architecture Decision Records

(fichiers a creer ci-dessous)
- `docs/adr/0001-authentification-cookie-session.md`
- `docs/adr/0002-password-hashing-argon2.md`
- `docs/adr/0003-mode-demo-visiteurs.md`
