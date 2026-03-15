# Architecture Spec -- Feature 0002 : Cycle de vie des devis et export

## Overview

Ajout d'un cycle de vie simplifie (draft/generated/deleted) aux devis, avec soft delete, filtres, export CSV, et recalcul. Les "brouillons" ne sont pas un statut en base mais une vue derivee : sessions authentifiees ayant des modeles uploades sans devis genere. L'export CSV permet l'interoperabilite avec les outils de devis legaux externes.

## Breaking Changes

**Non** -- Ajout de colonne nullable, nouvelles routes, aucune modification des endpoints existants.

## Data Model

### Migration 009 : ajout deleted_at sur quotes

```sql
ALTER TABLE quotes ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP NULL;
CREATE INDEX IF NOT EXISTS idx_quotes_deleted_at ON quotes(deleted_at);
```

## API Contracts

### GET /api/users/me/quotes (modifie)
- Ajout query param : `?status=draft|generated|deleted&page=1&per_page=20`
- Par defaut (sans filtre status) : retourne brouillons + generes (exclut deleted)
- `status=draft` : sessions avec modeles sans quote
- `status=generated` : quotes au statut generated
- `status=deleted` : quotes au statut deleted (soft-deleted)
- Response inchangee (QuoteListResponse) mais les items "draft" ont `total_price: 0.0`, `status: "draft"`, `id` = session_id (pas de quote_id)

### PATCH /api/users/me/quotes/{quote_id} (nouveau)
- Auth: cookie user_session
- Request body: `{"status": "deleted"}`
- Response 200: quote mise a jour
- Response 404: quote non trouvee ou pas proprietaire
- Seule transition autorisee : generated -> deleted

### GET /api/users/me/quotes/{quote_id}/export (nouveau)
- Auth: cookie user_session
- Query param: `?format=csv` (seul format MVP, JSON en should-have)
- Response 200: fichier CSV en telechargement
  - Content-Type: `text/csv; charset=utf-8`
  - Content-Disposition: `attachment; filename="devis-{id_court}-{date}.csv"`
  - BOM UTF-8 en debut de fichier
  - Separateur: point-virgule
- Response 404: quote non trouvee
- Response 400: format non supporte
- Disponible uniquement pour status=generated (pas draft, pas deleted)

### POST /api/users/me/quotes/{quote_id}/recalculate (nouveau)
- Auth: cookie user_session
- Response 201: nouveau quote genere avec les prix actuels
- Response 404: quote non trouvee ou pas proprietaire
- L'ancien devis reste inchange
- Le nouveau devis est lie a la meme session

## Plan d'implementation

| Etape | Fichier | Description |
|-------|---------|-------------|
| 1 | `src/db/migrations/009_quotes_deleted_at.sql` | Migration |
| 2 | `src/persistence/quotes.rs` | Ajout: soft_delete, find_by_id_and_user |
| 3 | `src/api/handlers/user_quotes.rs` | Modification: filtres status+drafts dans list, ajout: soft_delete, export_csv, recalculate |
| 4 | `src/api/routes.rs` | Nouvelles routes PATCH, GET export, POST recalculate |
| 5 | `templates/my-quotes.html` | Filtres, boutons action, export |
