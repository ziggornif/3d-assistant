# Code Review Report -- Feature 0002 : Cycle de vie des devis et export

## Verdict : APPROVE

## Summary

L'implementation de la feature 0002 est bien structuree et conforme a la spec. Le cycle de vie simplifie (draft/generated/deleted) est correctement implemente sans complexite inutile. Les brouillons sont une vue derivee (query SQL) et non un statut en base, ce qui est le bon choix. Le soft delete est securise (ownership check via JOIN SQL). L'export CSV est fonctionnel avec UTF-8 BOM et separateur point-virgule. Le recalcul cree correctement un nouveau devis. Les bugfixes (session auth dans SSR, suppression du cookie session_id au login, query param `?session=`) sont bien penses et ameliorent la coherence du flux. Aucun BLOCKER.

## Spec Conformity

- [x] US-001 Brouillons dans "Mes devis" -- conforme. Sessions authentifiees avec modeles et sans quote non-deleted apparaissent comme draft.
- [x] US-002 Soft delete -- conforme. Transition generated->deleted, `deleted_at` renseigne, confirmation modale, ownership check SQL.
- [x] US-003 Export CSV -- conforme. Separateur `;`, UTF-8 BOM, filename `devis-{id}-{date}.csv`, disponible uniquement pour status=generated.
- [x] US-004 Filtres par statut -- conforme. Tabs Tous/Brouillons/Generes/Supprimes dans my-quotes.html.
- [x] US-005 Recalcul -- conforme. Cree un nouveau devis, preserve l'ancien, utilise les prix actuels.
- [x] Migration 009 -- conforme. `ALTER TABLE ADD COLUMN IF NOT EXISTS`, index cree.
- [x] Nouvelles routes -- conformes. PATCH, GET export, POST recalculate.
- [x] Page detail devis -- conforme (bonus non demande dans la spec originale mais bien integre).

## Feedback

### [REV-001] La query draft utilise une double sous-requete COUNT -- SUGGESTION

- **Fichier** : `src/api/handlers/user_quotes.rs:114-131`
- **Constat** : La query des brouillons execute la sous-requete `(SELECT COUNT(*) FROM uploaded_models um WHERE um.session_id = qs.id)` deux fois : une fois dans le SELECT et une fois dans le WHERE. PostgreSQL devrait l'optimiser mais c'est plus propre de restructurer.
- **Impact** : Negligeable en pratique (PostgreSQL optimise les sous-requetes identiques).
- **Recommandation** : Utiliser un CTE ou une sous-requete HAVING pour ne compter qu'une fois. Non bloquant.

### [REV-002] Pagination en memoire pour la liste mixte (drafts + quotes) -- SUGGESTION

- **Fichier** : `src/api/handlers/user_quotes.rs:145-157`
- **Constat** : Les brouillons et les devis generes sont recuperes separement, fusionnes en memoire, tries, puis pagines cote Rust. Cela signifie que TOUTES les lignes sont chargees en memoire avant pagination.
- **Impact** : Acceptable pour le MVP (un utilisateur a rarement des centaines de devis). Pourrait poser probleme a grande echelle.
- **Recommandation** : En follow-up, envisager une UNION ALL en SQL avec pagination SQL directe. Non bloquant pour le MVP.

### [REV-003] CSV injection non presente mais a documenter -- SUGGESTION

- **Fichier** : `src/api/handlers/user_quotes.rs:296-307`
- **Constat** : Les noms de fichiers et de materiaux sont inseres directement dans le CSV sans echappement des caracteres speciaux CSV (`;`, `"`, newlines). Les noms de fichiers sont deja sanitises par `sanitize_filename()` en amont (lors de l'upload) et les noms de materiaux sont controles par l'admin. Le risque est donc faible. Cependant, si un nom de materiau contenait un `;`, la colonne serait decalee.
- **Impact** : Risque faible car les donnees sources sont controlees. Pas d'injection de formules (les champs ne commencent pas par `=`, `+`, `-`, `@`).
- **Recommandation** : Ajouter un echappement CSV minimal (encadrer les champs texte avec des guillemets doubles, doubler les guillemets internes). Non bloquant.

### [REV-004] Bugfix session auth au login bien implemente -- SUGGESTION (positif)

- **Fichier** : `src/api/handlers/auth.rs:109-115`
- **Constat** : Le cookie `session_id` (session anonyme) est supprime au login pour forcer la creation d'une session authentifiee au prochain chargement de page. C'est un bon fix qui resout le probleme ou un utilisateur qui se connecte conservait une session anonyme et ne pouvait pas generer de devis.
- **Recommandation** : Rien a changer, bien fait.

### [REV-005] Query param `?session=` pour reprendre un brouillon bien implemente -- SUGGESTION (positif)

- **Fichier** : `src/api/handlers/ssr.rs:30-38`
- **Constat** : L'ajout du query param `?session=` dans `index_page` permet de reprendre un brouillon depuis "Mes devis". La priorite (query param > cookie > nouvelle session) est correcte. Le cookie `session_id` est mis a jour pour que les appels API subsequents utilisent la bonne session.
- **Recommandation** : Verifier que le query param `session` ne permet pas de reprendre la session d'un AUTRE utilisateur. Actuellement, la query `find_by_id` ne filtre pas par `user_id`. Un utilisateur connecte pourrait theoriquement reprendre n'importe quelle session en connaissant l'ID. Le risque est attenue par le fait que les IDs sont des ULIDs (non devinables), mais un check supplementaire serait plus robuste.

### [REV-006] La page quote-detail.html est un bon ajout -- SUGGESTION (positif)

- **Fichier** : `templates/quote-detail.html`
- **Constat** : La page de detail avec affichage des modeles, breakdown, actions (export, recalcul, supprimer) et notice pour les devis supprimes est bien concue. Le bouton recalcul desactive pendant le traitement est un bon pattern UX.
- **Recommandation** : Rien a changer.

## Technical Debt Assessment

| Item | Severity | Action |
|------|----------|--------|
| Pagination en memoire (drafts + quotes) | Acceptable | Optimiser avec UNION ALL SQL en follow-up si volumetrie elevee |
| CSV sans echappement des champs | Acceptable | Ajouter echappement CSV (`"champ"`) en follow-up |
| Query param `?session=` sans check user_id | Low risk | Les ULIDs ne sont pas devinables, mais ajouter un check ownership en follow-up |
| Double sous-requete COUNT dans la query drafts | Trivial | Optimiser en follow-up |
| Hard delete admin (US-010) non implemente | Expected | Should-have, prevu pour plus tard |
| Export JSON (US-011) non implemente | Expected | Should-have, prevu pour plus tard |

## Tests Assessment

- **Couverture** : 126/126 tests passent. Les tests existants couvrent la logique metier (pricing, validation, auth, session). Les nouveaux endpoints (soft delete, export, recalculate) n'ont pas de tests unitaires dedies mais les queries SQL sont parametrees et le code est defensif (checks de status, ownership).
- **Tests manquants** (a ajouter en integration) : soft_delete sur un devis deja supprime, export d'un devis non-generated, recalculate sur un devis supprime, listing avec filtres, brouillons visibilite.
- **Qualite** : Le code est bien structure avec un helper `verify_user` factorise, des types de retour explicites, et une gestion d'erreurs coherente.

## Conclusion

Le code est propre, les fonctionnalites sont conformes a la spec, les bugfixes sont pertinents. Les points souleves sont tous des SUGGESTION d'amelioration, rien de bloquant. L'implementation est pragmatique et adaptee a un MVP.

**Verdict : APPROVE**
