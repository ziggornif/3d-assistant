# ADR 0003 : Mode demo pour les visiteurs

## Statut
Accepte

## Date
2026-03-14

## Contexte
L'application actuelle permet a n'importe qui de creer une session anonyme, uploader des fichiers 3D, et generer un devis. Avec l'ajout de la gestion de comptes, il faut decider du comportement pour les visiteurs non inscrits.

### Options envisagees

**Option A : Suppression totale de l'acces anonyme**
- Les visiteurs doivent s'inscrire et etre valides avant de pouvoir utiliser l'app
- Simple a implementer mais tue le funnel de decouverte

**Option B : Mode demo complet (upload + visualisation, sans devis)**
- Les visiteurs gardent l'experience actuelle sauf la generation de devis
- Le bouton "Generer le devis" est remplace par un CTA d'inscription
- Les sessions demo expirent apres 24h (comportement actuel)
- Permet de tester le service avant de s'engager

**Option C : Mode demo restreint (navigation seule, pas d'upload)**
- Les visiteurs ne peuvent que voir les materiaux
- Trop restrictif pour convaincre un prospect

## Decision
**Option B : Mode demo complet sans generation de devis**

## Consequences
- Le funnel de conversion est preserve : le visiteur decouvre le service, teste l'upload, et est motive a s'inscrire pour obtenir son devis
- Les sessions anonymous continuent de fonctionner avec leur expiration 24h
- L'endpoint `POST /api/sessions/{id}/quote` verifie le type de session et refuse les sessions anonymous
- L'endpoint MCP est exempte de cette restriction (canal machine-to-machine authentifie par token)
- Les modeles uploades en mode demo ne sont PAS transferes vers un compte utilisateur si le visiteur s'inscrit ensuite (sessions jetables)
- Le frontend doit conditionner l'affichage du bouton devis selon l'etat d'authentification
