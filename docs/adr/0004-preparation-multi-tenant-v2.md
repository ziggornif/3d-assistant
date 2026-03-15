# ADR 0004 : Preparation de l'isolation multi-tenant pour V2

## Statut
Annule

## Date
2026-03-14

## Contexte
Cette ADR proposait d'ajouter un champ `tenant_id` nullable sur la table `users` pour preparer une isolation forte par tenant en V2.

## Decision
**Annulee** -- Apres analyse, le concept de "tenant" (organisation regroupant plusieurs utilisateurs) n'est pas pertinent pour ce projet. Le discriminant pour l'isolation des donnees est le `user_id`, aussi bien en V1 (devis par utilisateur) qu'en V2 (prix personnalises par utilisateur). Le champ `tenant_id` serait une abstraction inutile (over-engineering).

En V2, les prix personnalises seront lies directement au `user_id` via une table `user_materials (user_id, material_id, custom_price_per_cm3)`. Aucune preparation structurelle n'est necessaire en V1.

## Consequences
- Pas de champ `tenant_id` dans la table `users`
- Schema plus simple et plus clair
- La V2 s'appuiera sur le `user_id` deja present dans les sessions et devis
