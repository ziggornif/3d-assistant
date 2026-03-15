# Product Spec -- Feature 0002 : Cycle de vie des devis et export

## Contexte

L'application est un outil de calcul de prix pour impression 3D, pas un logiciel de devis legal. Les devis generes n'ont actuellement aucun cycle de vie : ils sont crees avec le statut "generated" et restent indefiniment dans cet etat. L'utilisateur ne peut ni revenir sur un brouillon, ni supprimer un devis, ni exporter les donnees vers un outil de devis legal externe.

Le besoin est double :
1. **Cycle de vie simple** : brouillon (session en cours) -> genere -> supprime, avec soft delete et hard delete
2. **Export/interop** : permettre d'exporter les devis vers des outils externes (CSV, etc.) pour la facturation et le suivi legal

Ce n'est PAS un outil de devis legal. Les statuts "envoye", "accepte", "refuse" sont hors scope -- ils relevent d'un outil de devis dedie. L'interoperabilite via l'export est la strategie retenue pour le cadre legal.

## Personas

| Persona | Description | Besoins principaux |
|---------|-------------|-------------------|
| **Utilisateur actif** | Personne avec un compte valide | Voir ses brouillons et devis, les supprimer, les exporter |
| **Admin** | Gestionnaire de la plateforme | Voir les devis de tous les utilisateurs, hard delete si necessaire |

## User Stories

### Must Have (MVP)

**US-001** : En tant qu'utilisateur actif, je veux voir mes sessions en cours (brouillons) dans "Mes devis" afin de pouvoir y revenir et finaliser mes devis.
- Criteres d'acceptation :
  - [ ] Les sessions authentifiees avec au moins un modele uploade mais sans devis genere apparaissent dans "Mes devis" avec le statut "Brouillon"
  - [ ] Le brouillon affiche : date de creation, nombre de modeles, pas de prix (puisque pas encore genere)
  - [ ] Cliquer sur un brouillon redirige vers la page principale avec la session rechargee (les modeles sont presents, l'utilisateur peut continuer)
  - [ ] Les sessions vides (aucun modele) n'apparaissent PAS comme brouillons

**US-002** : En tant qu'utilisateur actif, je veux supprimer (soft delete) un devis afin de nettoyer ma liste sans perdre definitivement les donnees.
- Criteres d'acceptation :
  - [ ] Un bouton "Supprimer" est disponible sur chaque devis (genere ou brouillon) dans "Mes devis"
  - [ ] La suppression masque le devis de la liste par defaut
  - [ ] Le devis supprime reste en base avec le statut "deleted" et un champ `deleted_at` renseigne
  - [ ] Une confirmation est demandee avant la suppression ("Etes-vous sur de vouloir supprimer ce devis ?")
  - [ ] L'utilisateur ne peut PAS restaurer un devis supprime (irreversible cote utilisateur)

**US-003** : En tant qu'utilisateur actif, je veux exporter un devis au format CSV afin de l'importer dans mon outil de devis/facturation.
- Criteres d'acceptation :
  - [ ] Un bouton "Exporter CSV" est disponible sur la page de detail d'un devis genere
  - [ ] Le CSV contient : reference du devis, date, pour chaque ligne : nom du modele, materiau, volume (cm3), prix unitaire, prix ligne ; puis sous-total, frais de service, total
  - [ ] Le separateur est le point-virgule (convention francaise pour compatibilite Excel)
  - [ ] L'encodage est UTF-8 avec BOM (pour compatibilite Excel FR)
  - [ ] Le fichier est nomme `devis-{id_court}-{date}.csv` (ex: `devis-01JCV8E3-2026-03-14.csv`)
  - [ ] L'export n'est disponible QUE pour les devis au statut "generated" (pas les brouillons, pas les supprimes)

**US-004** : En tant qu'utilisateur actif, je veux filtrer mes devis par statut dans "Mes devis" afin de retrouver rapidement ce que je cherche.
- Criteres d'acceptation :
  - [ ] Des filtres par statut sont disponibles : "Tous" | "Brouillons" | "Generes"
  - [ ] Le filtre par defaut est "Tous" (brouillons + generes, les supprimes sont exclus)
  - [ ] Un filtre supplementaire "Supprimes" est disponible pour voir les devis supprimes (lecture seule, pas d'action possible)
  - [ ] Le nombre de devis par filtre est affiche (ex: "Brouillons (3)")

**US-005** : En tant qu'utilisateur actif, je veux regenerer un devis existant afin de mettre a jour les prix si les tarifs ont change.
- Criteres d'acceptation :
  - [ ] Un bouton "Recalculer" est disponible sur la page de detail d'un devis genere
  - [ ] Le recalcul utilise les prix actuels des materiaux
  - [ ] L'ancien devis est conserve (nouveau record en base, l'ancien reste avec son prix d'origine)
  - [ ] L'utilisateur voit clairement que c'est un nouveau devis avec les prix mis a jour

### Should Have

**US-010** : En tant qu'admin, je veux pouvoir hard delete un devis (suppression definitive) afin de nettoyer la base de donnees.
- Criteres d'acceptation :
  - [ ] Un bouton "Supprimer definitivement" est disponible dans l'interface admin pour les devis soft-deleted
  - [ ] La suppression est irreversible : le devis et ses donnees associees sont supprimes de la base
  - [ ] Seuls les devis deja soft-deleted peuvent etre hard-deleted (pas de suppression directe d'un devis actif)

**US-011** : En tant qu'utilisateur actif, je veux exporter un devis au format JSON afin de l'integrer dans des outils d'automatisation ou des APIs tierces.
- Criteres d'acceptation :
  - [ ] Un bouton "Exporter JSON" est disponible a cote du bouton "Exporter CSV"
  - [ ] Le JSON contient la structure complete du devis (memes donnees que l'API `GET /api/users/me/quotes/{id}`)
  - [ ] Le fichier est nomme `devis-{id_court}-{date}.json`

### Could Have

**US-020** : En tant qu'utilisateur actif, je veux dupliquer un devis existant afin de creer une variante sans tout refaire.
- Criteres d'acceptation :
  - [ ] Un bouton "Dupliquer" cree une nouvelle session avec les memes modeles et materiaux
  - [ ] Le nouveau devis est au statut "brouillon" (l'utilisateur doit le regenerer)

### Won't Have (this time)

- **Statuts "envoye", "accepte", "refuse"** : relevent d'un outil de devis legal, hors scope de cette application
- **Expiration automatique des devis** : pas de duree de validite automatique
- **Historique des versions d'un devis** : un devis recalcule est un nouveau devis, pas une version
- **Export PDF** : hors scope (le CSV et JSON couvrent l'interop)
- **Partage de devis par lien public** : hors scope

## Flux principaux

### Flux 1 : Brouillon visible dans "Mes devis"

1. L'utilisateur connecte cree une session (POST /api/sessions)
2. Il uploade un ou plusieurs modeles 3D
3. Il ne genere PAS le devis (il quitte la page ou revient plus tard)
4. Dans "Mes devis", la session apparait comme "Brouillon" avec le nombre de modeles
5. Il clique sur le brouillon -> il est redirige vers la page principale avec sa session rechargee
6. Il peut continuer : selectionner des materiaux, generer le devis
- Cas limite : session avec 0 modele -> n'apparait PAS dans "Mes devis"
- Cas limite : session expiree (30 jours pour les authentifiees) -> brouillon n'apparait plus

### Flux 2 : Suppression soft d'un devis

1. L'utilisateur va dans "Mes devis"
2. Il clique sur "Supprimer" sur un devis
3. Une modale de confirmation apparait : "Etes-vous sur de vouloir supprimer ce devis ? Cette action est irreversible."
4. Il confirme
5. Le devis disparait de la liste (statut passe a "deleted", `deleted_at` renseigne)
6. Le devis reste visible dans le filtre "Supprimes" (lecture seule)
- Cas d'erreur : devis deja supprime -> message "Ce devis a deja ete supprime"

### Flux 3 : Export CSV

1. L'utilisateur va dans le detail d'un devis genere
2. Il clique sur "Exporter CSV"
3. Le navigateur telecharge le fichier CSV
4. L'utilisateur importe le CSV dans son outil de devis/facturation (Excel, Google Sheets, logiciel comptable)
- Cas limite : devis sans modeles (ne devrait pas arriver) -> CSV avec uniquement les totaux
- Cas d'erreur : erreur serveur -> message d'erreur, pas de telechargement

### Flux 4 : Recalcul d'un devis

1. L'utilisateur va dans le detail d'un devis genere
2. Il voit le prix calcule avec les anciens tarifs
3. Il clique sur "Recalculer avec les prix actuels"
4. Un nouveau devis est genere avec les prix actuels
5. L'ancien devis reste dans l'historique (non supprime)
6. L'utilisateur est redirige vers le detail du nouveau devis
- Cas limite : les prix n'ont pas change -> le nouveau devis a le meme total (c'est normal)

## Contraintes business

- **Pas de devis legal** : l'application calcule des prix. L'export CSV/JSON est le pont vers les outils legaux.
- **Tracabilite** : un devis supprime (soft delete) reste en base. Seul l'admin peut hard delete.
- **Performance** : la liste "Mes devis" inclut maintenant les brouillons (sessions avec modeles). La requete doit rester performante.
- **Retrocompatibilite** : les devis existants (statut "generated") ne sont pas impactes. Le nouveau champ `deleted_at` est nullable.

## Questions ouvertes

- **Format CSV exact** : le separateur point-virgule est-il confirme ? (convention FR pour Excel). Alternative : detecter la locale ou proposer les deux.
- **Limite d'export** : peut-on exporter plusieurs devis en un seul CSV (export bulk) ? Non prevu pour V1, un devis = un fichier.
