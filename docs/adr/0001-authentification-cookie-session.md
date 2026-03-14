# ADR 0001 : Authentification par cookie de session

## Statut
Accepte

## Date
2026-03-14

## Contexte
L'application de devis 3D doit passer d'un mode anonyme a un mode multi-utilisateurs. Il faut choisir un mecanisme d'authentification pour les utilisateurs.

L'application existante est une app SSR (Server-Side Rendering) avec Tera comme moteur de templates. L'interface admin utilise deja des cookies (`admin_token`) pour l'authentification. Il n'y a pas d'API publique consommee par des clients tiers.

### Options envisagees

**Option A : Cookie-based session auth**
- Token de session genere cote serveur, stocke dans un cookie HttpOnly/SameSite
- Table `user_sessions` en base PostgreSQL
- Coherent avec le pattern admin existant
- Simple a implementer, securise par defaut (HttpOnly empeche XSS, SameSite empeche CSRF)

**Option B : JWT (JSON Web Tokens)**
- Token signe cote serveur, stocke cote client (cookie ou localStorage)
- Stateless (pas besoin de table de sessions)
- Plus complexe (gestion de refresh tokens, invalidation, signature key rotation)
- Utile quand il y a plusieurs services ou une API publique

**Option C : OAuth2 / OpenID Connect**
- Delegation a un provider externe (Google, GitHub)
- Hors scope V1 explicitement

## Decision
**Option A : Cookie-based session auth**

## Consequences
- Coherence avec le pattern d'authentification admin existant dans le codebase
- Simplicite d'implementation (pas de gestion de refresh tokens, pas de key rotation)
- Les sessions sont revocables instantanement (suppression en base)
- Necessite une table `user_sessions` en base (leger surcharge sur chaque requete authentifiee)
- Si l'application evolue vers une API publique ou des clients mobiles, il faudra reconsiderer (mais c'est hors scope V1 et V2)
