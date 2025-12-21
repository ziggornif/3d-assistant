# ADR-005: Migration vers PostgreSQL uniquement

## Statut
**Accepté** - Novembre 2025

## Contexte

L'application utilisait initialement une approche dual-database avec SQLx supportant SQLite (développement) et PostgreSQL (production). Cette stratégie, documentée dans ADR-004, visait à simplifier le développement local tout en permettant une migration vers PostgreSQL pour la production.

### Problèmes rencontrés

1. **Incompatibilités SQL** : Différences de syntaxe entre SQLite et PostgreSQL
   - `datetime('now')` vs paramètres bindés
   - `?` placeholders vs `$1, $2, ...`
   - `INSERT OR IGNORE` vs `ON CONFLICT DO NOTHING`
   - `INTEGER` vs `BOOLEAN` pour les booléens
   - `REAL` vs `DOUBLE PRECISION` pour les flottants

2. **Complexité de maintenance** : Maintenir deux chemins de code et tester sur deux bases de données

3. **Risques de régression** : Comportements subtils différents entre les deux moteurs

4. **Overhead Docker** : Configuration plus complexe avec le driver "any"

## Décision

**Standardiser sur PostgreSQL uniquement** pour tous les environnements (développement, test, production).

### Changements techniques

- **SQLx** : Retrait du feature `any` et `sqlite`, utilisation directe de `postgres`
- **Migrations** : Types PostgreSQL natifs (`BOOLEAN`, `TIMESTAMP`, `BIGINT`, `DOUBLE PRECISION`)
- **Requêtes** : Placeholders PostgreSQL (`$1, $2, ...`) systématiquement
- **CI/CD** : Service PostgreSQL dans GitHub Actions pour les tests E2E
- **Docker** : Support PostgreSQL uniquement dans les images

## Conséquences

### Positives

- **Cohérence** : Même moteur en dev, test et prod
- **Types natifs** : Utilisation des types PostgreSQL optimaux (BOOLEAN, TIMESTAMP)
- **Performances** : Pas de couche d'abstraction "any"
- **Sécurité** : Une seule surface d'attaque à analyser
- **Maintenance** : Code simplifié, moins de conditionnels

### Négatives

- **Setup local** : Nécessite PostgreSQL ou Docker Compose
- **CI** : Service container PostgreSQL requis (léger overhead)
- **Barrier d'entrée** : Légèrement plus complexe pour les nouveaux contributeurs

### Atténuation

- Docker Compose fourni pour démarrage rapide
- CI pré-configuré avec PostgreSQL
- Documentation mise à jour avec instructions claires

## Alternatives considérées

1. **Maintenir dual-database** : Rejeté - complexité croissante, risques de bugs
2. **SQLite uniquement** : Rejeté - limitations pour la production (concurrence, scalabilité)
3. **ORM complet (Diesel, SeaORM)** : Rejeté - overhead, apprentissage, SQLx suffit

## Fichiers modifiés

- `Cargo.toml` : Features SQLx
- `src/db/mod.rs` : Pool PostgreSQL direct
- `src/db/migrations/*.sql` : Types PostgreSQL natifs
- `src/db/seed.sql` : Syntaxe PostgreSQL
- `.github/workflows/ci.yml` : Service PostgreSQL
- `docker-compose.yml` : Configuration PostgreSQL
- `README.md`, `.env.example` : Documentation

## Références

- [ADR-004: SQLite for Prototype](specs/001-3d-print-quoting/plan.md#adr-004) - Décision initiale
- [SQLx PostgreSQL](https://github.com/launchbadge/sqlx) - Driver utilisé
- [PostgreSQL Types](https://www.postgresql.org/docs/current/datatype.html)
