# Security Audit Report -- Feature 0001 : Gestion de comptes multi-tenants

## Scope

Fichiers analyses :
- `src/business/auth.rs` -- Service d'authentification, hashing, tokens
- `src/api/handlers/auth.rs` -- Handlers register, login, logout, me
- `src/api/handlers/user_quotes.rs` -- Handlers devis utilisateur
- `src/api/handlers/admin.rs` -- Handlers admin (list_users, update_user_status)
- `src/api/handlers/upload.rs` -- Creation de session (modifie)
- `src/api/handlers/quote.rs` -- Generation de devis (modifie : blocage anonymous)
- `src/api/handlers/ssr.rs` -- Pages SSR (login, register, my-quotes)
- `src/api/routes.rs` -- Configuration des routes
- `src/api/middleware/auth.rs` -- Middleware admin_auth, mcp_auth
- `src/api/middleware/error.rs` -- Gestion d'erreurs
- `src/api/middleware/security_headers.rs` -- Headers de securite
- `src/models/user.rs` -- Modele User, validations
- `src/persistence/users.rs` -- Requetes SQL users
- `src/persistence/user_sessions.rs` -- Requetes SQL sessions
- `templates/login.html`, `templates/register.html`, `templates/admin.html` -- Templates SSR

## Summary

- **BLOCKER** : 0
- **WARNING** : 3
- **INFO** : 4

## Findings

### [SEC-001] CORS reste ouvert (`allow_origin(Any)`) -- WARNING

- **Categorie** : OWASP A05 Security Misconfiguration
- **Localisation** : `src/api/routes.rs:37-40`
- **Description** : Le CORS est configure avec `allow_origin(Any)`, `allow_methods(Any)`, `allow_headers(Any)`. C'etait deja le cas avant cette feature (note dans le `plan.md` comme issue existante), mais avec l'ajout d'endpoints d'authentification, le risque augmente : un site malveillant pourrait effectuer des requetes cross-origin vers `/api/auth/login` avec les cookies de l'utilisateur.
- **Impact** : Un attaquant pourrait potentiellement effectuer des requetes authentifiees cross-origin. Cependant, les cookies `user_session` sont configures avec `SameSite=Lax`, ce qui bloque les requetes POST cross-origin. Le risque reel est donc attenue.
- **Recommandation** : Configurer le CORS avec une whitelist d'origines autorisees via variable d'environnement. Ceci etait deja prevu dans le `plan.md` (phase 1.2). A traiter en priorite maintenant que des endpoints auth existent.

### [SEC-002] Pas de protection CSRF sur les formulaires SSR -- WARNING

- **Categorie** : OWASP A01 Broken Access Control
- **Localisation** : `templates/login.html`, `templates/register.html`
- **Description** : Les formulaires login et register utilisent `fetch()` en JavaScript (pas de `<form method="POST">` classique), ce qui les rend moins vulnerables au CSRF classique. De plus, les cookies sont `SameSite=Lax`, ce qui bloque les POST cross-origin. Cependant, le formulaire admin login (`templates/admin.html` ligne 38) utilise toujours `<form method="POST" action="/admin/login">` sans token CSRF. C'est un point preexistant, mais l'ajout de la feature le rappelle.
- **Impact** : Le risque est attenue par `SameSite=Strict` sur le cookie admin_token. Un CSRF classique (redirect POST depuis un autre site) serait bloque. Le risque residuel est faible.
- **Recommandation** : Pour robustesse en profondeur, envisager d'ajouter un token CSRF sur les formulaires POST SSR du admin. Non bloquant pour cette feature.

### [SEC-003] Rate limiting sur `/api/auth/register` absent -- WARNING

- **Categorie** : OWASP A07 Identification and Authentication Failures
- **Localisation** : `src/api/routes.rs:55`
- **Description** : L'endpoint `/api/auth/register` n'a pas de rate limiting specifique. Seul le rate limiter global (20 req/s) s'applique. Un attaquant pourrait tenter de creer de nombreux comptes (spam) meme si la validation admin empeche leur activation.
- **Impact** : Pollution de la base de donnees avec des comptes "pending" spam. L'admin devrait refuser manuellement chaque compte spam.
- **Recommandation** : Ajouter `.layer(create_login_rate_limiter())` sur la route register, comme c'est deja fait pour login. Le rate limiter existant (1 req/s) suffirait.
- **Exemple** :
  ```rust
  // Avant
  .route("/api/auth/register", post(auth::register))
  // Apres
  .route("/api/auth/register", post(auth::register).layer(create_login_rate_limiter()))
  ```

### [SEC-004] Argon2id avec parametres par defaut -- INFO

- **Categorie** : OWASP A02 Cryptographic Failures
- **Localisation** : `src/business/auth.rs:186`
- **Description** : `Argon2::default()` utilise les parametres par defaut du crate `argon2` qui sont raisonnables (Argon2id, m_cost=19456 KiB, t_cost=2, p_cost=1). Ce sont les parametres recommandes par OWASP.
- **Impact** : Aucun -- les parametres par defaut sont securises.
- **Recommandation** : Documenter explicitement les parametres dans un commentaire pour reference. Optionnellement, les rendre configurables via variables d'environnement pour les ajuster si le hardware le permet.

### [SEC-005] Protection timing attacks implementee correctement -- INFO (positif)

- **Categorie** : OWASP A07
- **Localisation** : `src/business/auth.rs:111-122`
- **Description** : La fonction `login()` execute `hash_password()` (operation couteuse) meme quand l'email n'existe pas, empechant un attaquant de distinguer "email inexistant" de "mot de passe incorrect" via le temps de reponse.
- **Impact** : Protection efficace contre l'enumeration d'emails par timing.
- **Recommandation** : Aucune -- correctement implemente.

### [SEC-006] Password hash exclu de la serialisation JSON -- INFO (positif)

- **Categorie** : OWASP A02 Cryptographic Failures
- **Localisation** : `src/models/user.rs:21`
- **Description** : `#[serde(skip_serializing)]` sur `password_hash` empeche la fuite du hash dans les reponses JSON.
- **Impact** : Protection efficace contre l'exposition du hash de mot de passe.
- **Recommandation** : Aucune -- correctement implemente.

### [SEC-007] Messages d'erreur generiques sur login -- INFO (positif)

- **Categorie** : OWASP A07
- **Localisation** : `src/api/handlers/auth.rs:87`
- **Description** : Le message "Email ou mot de passe incorrect" est identique que l'email existe ou non. Pas de distinction "utilisateur inconnu" vs "mot de passe incorrect".
- **Impact** : Protection efficace contre l'enumeration d'emails via les messages d'erreur.
- **Recommandation** : Aucune -- correctement implemente. Les erreurs 403 pour pending/disabled revelent que l'email existe, mais c'est un compromis UX acceptable (l'utilisateur doit savoir que son compte est en attente).

## Checklist Summary

| Check | Status | Notes |
|-------|--------|-------|
| Injections SQL | PASS | Toutes les requetes utilisent des parametres SQLx (`$1`, `$2`...). Aucune concatenation. |
| XSS | PASS | Les templates Tera utilisent l'auto-escape par defaut. Le JS frontend utilise `textContent` (pas `innerHTML` avec du user input). `escapeHtml()` dans les templates admin. |
| Authentication | PASS | Argon2id, protection timing, messages generiques, sessions revocables. |
| Session management | PASS | Cookies HttpOnly + SameSite=Lax + Secure(prod). Expiration 30j. Revocation cote serveur au logout et a la desactivation. |
| Access control | PASS | Admin endpoints proteges par `admin_auth` middleware. User endpoints verifient le cookie `user_session`. Isolation devis par `user_id` (join SQL). |
| IDOR | PASS | `get_my_quote` verifie ownership via `WHERE q.id = $1 AND qs.user_id = $2`. `list_my_quotes` filtre par `user_id`. Impossible d'acceder aux devis d'un autre utilisateur. |
| Secrets | PASS | Aucun secret dans le code. `admin_token` via env var. Pas de token hardcode dans la feature auth. |
| Input validation | PASS | Email, password, display_name, status -- tous valides cote serveur avant persistence. |
| Error messages | PASS | Pas de stack trace expose. Messages generiques pour les erreurs d'auth. Codes HTTP corrects. |
| CORS | WARNING | `allow_origin(Any)` -- preexistant mais aggrave par les endpoints auth. |
| CSRF | WARNING | Attenue par SameSite cookies. Le formulaire admin POST preexistant n'a pas de token CSRF. |
| Rate limiting | WARNING | Absent sur `/api/auth/register`. |
| Headers de securite | PASS | HSTS, X-Frame-Options, X-Content-Type-Options, CSP, Permissions-Policy, Referrer-Policy -- tous presents. |
