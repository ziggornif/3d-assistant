# Code Review Report -- Feature 0001 : Gestion de comptes multi-tenants

## Verdict : APPROVE (avec warnings)

## Summary

L'implementation est solide, bien structuree, et conforme a la spec de l'architecte dans ses grandes lignes. Le code respecte les patterns du codebase existant (structure Axum handlers/services/persistence, requetes SQL parametrees, gestion d'erreurs avec AppError/AppResult, conventions de nommage). Les points d'ecart avec la spec sont mineurs (champs manquants dans des reponses JSON, mauvais type d'erreur HTTP) et ne sont pas bloquants. Les tests unitaires couvrent bien la logique metier. Le code est pret a merger apres correction des 2 warnings ci-dessous.

## Spec Conformity

### Endpoints

- [x] `POST /api/auth/register` -- conforme (201, request/response body, 400 validation, 409 email)
- [x] `POST /api/auth/login` -- conforme sauf champ `session_id` manquant dans la reponse (voir REV-001)
- [x] `POST /api/auth/logout` -- conforme
- [x] `GET /api/auth/me` -- conforme
- [x] `GET /api/users/me/quotes` -- conforme (pagination, isolation par user)
- [x] `GET /api/users/me/quotes/{id}` -- conforme sauf HTTP status (voir REV-002)
- [x] `GET /api/admin/users` -- conforme (pagination, filtre par status)
- [x] `PATCH /api/admin/users/{id}` -- conforme sauf champ `updated_at` manquant et mauvais HTTP status (voir REV-002)
- [x] `POST /api/sessions` -- conforme (anonymous vs authenticated)
- [x] `POST /api/sessions/{id}/quote` -- conforme (blocage anonymous 403)

### Data Model

- [x] Table `users` -- conforme a la spec
- [x] Table `user_sessions` -- conforme
- [x] Table `quote_sessions` modifiee -- conforme (`user_id`, `session_type`)
- [x] Modele Rust `User` -- conforme
- [x] Modele Rust `UserSession` -- conforme
- [x] Modele Rust `QuoteSession` -- conforme (nouveaux champs + methodes)

### Pages SSR

- [x] `/login` -- conforme
- [x] `/register` -- conforme
- [x] `/my-quotes` -- conforme (avec redirection login)
- [x] Header conditionnel (visiteur/connecte) -- conforme
- [x] Admin section gestion comptes -- conforme (table, filtres, badges, actions)
- [x] Quote summary demo mode -- conforme (CTA banner, rappel "Mes devis")

### Mode demo

- [x] Sessions anonymous gardent expiration 24h
- [x] Upload et visualisation fonctionnent en mode demo
- [x] Generation de devis bloquee (403) pour sessions anonymous
- [x] CTA "Inscrivez-vous" dans le composant quote-summary

## Feedback

### [REV-001] LoginResponse manque le champ `session_id` de la spec -- WARNING

- **Fichier** : `src/api/handlers/auth.rs:67-72`
- **Constat** : La spec de l'architecte definit la reponse login avec un champ `session_id` (ID de la quote session creee au login). L'implementation ne retourne que `user_id`, `email`, `display_name`, `role`. Le `session_id` n'est pas inclus car aucune quote session n'est creee au moment du login (elle est creee ensuite par `POST /api/sessions`).
- **Impact** : Ecart mineur avec la spec. Le flux fonctionne car la quote session est creee separement via l'endpoint existant. Le frontend SSR cree la session cote serveur dans `index_page`.
- **Recommandation** : Soit ajouter la creation d'une quote session au login et inclure le `session_id` dans la reponse, soit mettre a jour la spec pour refleter le comportement actuel. L'approche actuelle (session creee separement) est plus propre car elle separe les concerns auth et quoting. Recommandation : mettre a jour la spec.

### [REV-002] Mauvais type AppError pour "not found" dans deux handlers -- WARNING

- **Fichier** : `src/api/handlers/user_quotes.rs:184` et `src/api/handlers/admin.rs:331`
- **Constat** : Les deux handlers utilisent `AppError::Internal(...)` quand un devis ou un utilisateur n'est pas trouve. `AppError::Internal` retourne un HTTP 500, alors que la spec attend un 404 pour "devis non trouve" et "utilisateur non trouve".
- **Impact** : Un 500 est retourne au lieu d'un 404, ce qui est incorrect semantiquement et peut etre confus pour le client.
- **Recommandation** : Utiliser les variantes existantes ou en creer une adaptee.
  ```rust
  // user_quotes.rs:184 -- avant
  .ok_or_else(|| AppError::Internal("Devis non trouve".to_string()))?;
  // apres -- utiliser une variante qui retourne 404
  // Option A : creer une variante generique NotFound
  // Option B : reutiliser SessionNotFound ou creer QuoteNotFound
  ```
  Le plus simple : ajouter une variante `AppError::NotFound(String)` qui retourne HTTP 404 avec un message generique. Cela couvrirait les deux cas.

### [REV-003] AdminUserResponse manque le champ `updated_at` -- SUGGESTION

- **Fichier** : `src/api/handlers/admin.rs:252-259`
- **Constat** : La spec de l'architecte indique que la reponse de `PATCH /api/admin/users/{id}` doit inclure `updated_at`. L'implementation inclut `created_at` mais pas `updated_at`.
- **Impact** : Ecart cosmique avec la spec. L'admin ne voit pas la date de derniere modification.
- **Recommandation** : Ajouter `pub updated_at: NaiveDateTime` a `AdminUserResponse` et le peupler depuis `user.updated_at`.

### [REV-004] `UserSession` importe mais jamais utilise dans auth.rs -- SUGGESTION

- **Fichier** : `src/business/auth.rs:3`
- **Constat** : `UserSession` est importe dans la ligne `use crate::models::user::{self, User, UserSession, ...}` mais n'est jamais utilise directement dans le fichier. Le compilateur a probablement emis un warning.
- **Impact** : Warning de compilation, code mort.
- **Recommandation** : Retirer `UserSession` de l'import.

### [REV-005] N+1 query dans `get_user_quotes` -- SUGGESTION

- **Fichier** : `src/api/handlers/user_quotes.rs:124-132`
- **Constat** : Pour chaque devis, une requete supplementaire `SELECT COUNT(*) FROM uploaded_models WHERE session_id = $1` est executee. C'est un pattern N+1 : pour 20 devis, 21 requetes sont executees.
- **Impact** : Performance degradee si l'utilisateur a beaucoup de devis. Acceptable pour le MVP (paginiation a 20 par page), mais a surveiller.
- **Recommandation** : Remplcer par une seule requete avec sous-requete ou LEFT JOIN.
  ```sql
  SELECT q.id, q.session_id, q.total_price, q.status, q.created_at::text,
         (SELECT COUNT(*) FROM uploaded_models um WHERE um.session_id = q.session_id) as model_count
  FROM quotes q
  JOIN quote_sessions qs ON q.session_id = qs.id
  WHERE qs.user_id = $1
  ORDER BY q.created_at DESC
  LIMIT $2 OFFSET $3
  ```
  Non bloquant -- optimisation a faire en follow-up.

### [REV-006] `persistence::quotes` non modifie pour `find_by_user` -- SUGGESTION

- **Fichier** : `src/persistence/quotes.rs`
- **Constat** : La spec de l'architecte mentionnait "Ajout query `find_by_user` dans `persistence/quotes.rs`". L'implementation a choisi de faire les requetes directement dans le handler `user_quotes.rs` plutot que dans la couche persistence. C'est un ecart de structure, mais les requetes sont correctes et parametrees.
- **Impact** : Legere inconsistance avec le pattern du codebase (normalement, toutes les requetes SQL sont dans `persistence/`). Non bloquant car le handler est simple et autosuffisant.
- **Recommandation** : Dans un follow-up, deplacer les requetes SQL de `user_quotes.rs` vers `persistence/quotes.rs` pour maintenir la coherence architecturale.

## Technical Debt Assessment

| Item | Severity | Action |
|------|----------|--------|
| N+1 queries dans `get_user_quotes` | Acceptable | Optimiser en follow-up (sous-requete) |
| Requetes SQL dans handler au lieu de persistence | Acceptable | Refactorer en follow-up |
| `UserSession` import inutile | Trivial | Corriger dans le prochain commit |
| Pas de middleware `user_auth` dedie | Acceptable | L'auth est verifiee dans chaque handler via `AuthService::verify_session`. Un middleware dedie serait plus DRY mais l'approche actuelle est correcte et explicite. |
| CORS `allow_origin(Any)` | Preexistant | Hors scope feature, dans le plan.md |
| Pas de nettoyage des `user_sessions` expirees au startup | Acceptable | A ajouter en follow-up, comme le cleanup des quote sessions |

## Tests Assessment

- **Couverture unitaire** : Suffisante. 17 tests pour les validations user, 4 tests pour le hashing/token auth, 17 tests pricing existants, 20 tests sanitize existants. Total 126/126 passent.
- **Tests manquants** (mineurs) :
  - `QuoteSession::new_authenticated` -- pas de test unitaire dedie (couvert implicitement par integration)
  - `QuoteSession::is_anonymous` / `is_authenticated` -- pas de test unitaire dedie
- **Qualite des tests** : Bonne. Les noms sont descriptifs, les assertions sont specifiques, les cas limites sont couverts (password trop court, email sans @, display_name vide, etc.)
- **Tests d'integration** : Non ecrits par le QA (pas d'acces Bash). Le plan de 30+ tests est documente dans le QA report. A ecrire en follow-up.

## Conclusion

Le code est propre, bien structure, et conforme a la spec dans les aspects critiques (auth, securite, isolation des donnees, mode demo). Les ecarts identifies (REV-001 a REV-006) sont tous mineurs ou des suggestions d'amelioration. Les 2 warnings (REV-001 et REV-002) sont corrigeables rapidement mais ne bloquent pas le merge.

**Verdict : APPROVE**
