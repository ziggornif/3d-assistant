# Product Spec -- Feature 0001 : Gestion de comptes multi-tenants

## Contexte

L'application de devis d'impression 3D fonctionne actuellement en mode mono-compte : n'importe qui peut creer une session anonyme (24h), uploader des fichiers 3D, et generer un devis. Il n'y a aucune notion d'utilisateur, aucun historique, et aucune isolation des donnees.

Le besoin est de passer a un systeme multi-tenants ou chaque utilisateur enregistre (et valide par l'admin) dispose de son propre espace avec ses devis. Cela permet :
- De fideliser les utilisateurs en leur offrant un historique de devis
- De controler qui accede au service de devis (validation admin)
- De preparer l'architecture pour une isolation forte future (prix personnalises par tenant)

## Personas

| Persona | Description | Besoins principaux |
|---------|-------------|-------------------|
| **Visiteur** | Personne non inscrite qui decouvre l'application | Voir les materiaux, tester l'upload en mode demo, comprendre le service |
| **Utilisateur inscrit (en attente)** | Personne ayant rempli le formulaire d'inscription, pas encore validee | Savoir que son compte est en attente, recevoir une confirmation quand il est active |
| **Utilisateur actif** | Personne avec un compte valide par l'admin | Creer des devis, voir ses devis passes, uploader des modeles 3D |
| **Admin** | Gestionnaire de la plateforme (role existant) | Valider/refuser les inscriptions, desactiver des comptes, gerer les materiaux (existant) |

## User Stories

### Must Have (MVP)

**US-001** : En tant que visiteur, je veux m'inscrire via un formulaire (email, mot de passe, nom) afin de creer un compte sur la plateforme.
- Criteres d'acceptation :
  - [ ] Le formulaire d'inscription contient : email, mot de passe, confirmation de mot de passe, nom (prenom + nom ou nom d'affichage)
  - [ ] L'email doit etre unique dans le systeme
  - [ ] Le mot de passe doit respecter une politique minimale (8 caracteres minimum, au moins une majuscule, un chiffre)
  - [ ] Apres soumission valide, l'utilisateur voit un message "Compte cree, en attente de validation par l'administrateur"
  - [ ] Le compte est cree avec le statut "pending"
  - [ ] En cas d'email deja utilise, un message d'erreur clair est affiche

**US-002** : En tant qu'utilisateur inscrit (en attente), je veux etre informe que mon compte est en attente de validation afin de savoir que je ne peux pas encore utiliser le service.
- Criteres d'acceptation :
  - [ ] Si un utilisateur "pending" tente de se connecter, il voit un message "Votre compte est en attente de validation par l'administrateur"
  - [ ] L'utilisateur ne peut acceder a aucune fonctionnalite de devis
  - [ ] L'utilisateur ne peut pas contourner cette restriction

**US-003** : En tant qu'utilisateur actif, je veux me connecter avec mon email et mon mot de passe afin d'acceder a mon espace personnel.
- Criteres d'acceptation :
  - [ ] Page de connexion avec email + mot de passe
  - [ ] En cas de succes, redirection vers la page d'accueil avec session authentifiee
  - [ ] En cas d'echec (mauvais identifiants), message d'erreur generique "Email ou mot de passe incorrect"
  - [ ] En cas de compte desactive, message "Votre compte a ete desactive. Contactez l'administrateur."
  - [ ] Rate limiting sur les tentatives de connexion (protection brute force)

**US-004** : En tant qu'utilisateur actif, je veux me deconnecter afin de securiser mon acces.
- Criteres d'acceptation :
  - [ ] Bouton de deconnexion visible dans l'interface
  - [ ] Apres deconnexion, redirection vers la page d'accueil (mode visiteur)
  - [ ] La session est invalidee cote serveur

**US-005** : En tant qu'utilisateur actif, je veux creer des devis qui sont lies a mon compte afin de les retrouver plus tard.
- Criteres d'acceptation :
  - [ ] Chaque devis cree est associe au compte de l'utilisateur connecte
  - [ ] L'utilisateur ne voit que SES devis (pas ceux des autres)
  - [ ] Le flux de creation de devis (upload, selection materiau, generation) reste identique a l'existant
  - [ ] Les sessions de devis ne sont plus anonymes : elles sont liees au user_id

**US-006** : En tant qu'utilisateur actif, je veux voir l'historique de mes devis passes afin de les consulter ou les reutiliser comme reference.
- Criteres d'acceptation :
  - [ ] Une page ou section "Mes devis" liste tous les devis de l'utilisateur
  - [ ] Chaque devis affiche : date, nombre de modeles, prix total, statut
  - [ ] L'utilisateur peut cliquer sur un devis pour voir le detail (breakdown)
  - [ ] Les devis sont tries par date (plus recents en premier)

**US-007** : En tant qu'admin, je veux voir la liste des comptes en attente de validation afin de les approuver ou les refuser.
- Criteres d'acceptation :
  - [ ] Dans l'interface admin, une nouvelle section "Gestion des comptes" est visible
  - [ ] La liste affiche les comptes "pending" avec : nom, email, date d'inscription
  - [ ] L'admin peut approuver un compte (statut passe a "active")
  - [ ] L'admin peut refuser un compte (statut passe a "rejected" ou le compte est supprime)
  - [ ] L'action est immediate, sans confirmation supplementaire (ou avec confirmation simple)

**US-008** : En tant qu'admin, je veux pouvoir desactiver un compte existant afin de revoquer l'acces d'un utilisateur.
- Criteres d'acceptation :
  - [ ] Dans la liste des comptes actifs, un bouton "Desactiver" est disponible
  - [ ] Un compte desactive ne peut plus se connecter
  - [ ] Les devis passes du compte desactive restent en base (pas de suppression)
  - [ ] L'admin peut reactiver un compte desactive

**US-009** : En tant que visiteur, je veux utiliser l'application en mode demo afin de decouvrir le service sans m'inscrire.
- Criteres d'acceptation :
  - [ ] Un visiteur non connecte peut naviguer sur la page d'accueil
  - [ ] Il peut voir la liste des materiaux disponibles
  - [ ] Il peut uploader un fichier 3D et voir la visualisation + les infos du modele
  - [ ] Il ne peut PAS generer de devis final (bouton desactive ou masque avec message "Inscrivez-vous pour obtenir un devis")
  - [ ] La session demo fonctionne comme les sessions actuelles (temporaire, 24h) mais sans generation de devis

### Should Have

**US-010** : En tant qu'utilisateur actif, je veux que mes sessions de devis n'expirent pas automatiquement apres 24h afin de ne pas perdre mon travail en cours.
- Criteres d'acceptation :
  - [ ] Les sessions liees a un compte utilisateur actif n'ont pas d'expiration automatique (ou expiration beaucoup plus longue, ex: 30 jours)
  - [ ] Les sessions demo (visiteurs) gardent l'expiration 24h actuelle

**US-011** : En tant qu'admin, je veux voir la liste de tous les comptes (actifs, en attente, desactives) afin d'avoir une vue d'ensemble.
- Criteres d'acceptation :
  - [ ] La liste des comptes est filtrable par statut (tous, pending, active, disabled)
  - [ ] Chaque ligne affiche : nom, email, statut, date d'inscription, nombre de devis

### Could Have

**US-020** : En tant qu'utilisateur actif, je veux modifier mon mot de passe afin de securiser mon compte.
- Criteres d'acceptation :
  - [ ] Formulaire de changement de mot de passe (ancien + nouveau + confirmation)
  - [ ] Validation de la politique de mot de passe sur le nouveau

**US-021** : En tant qu'utilisateur actif, je veux modifier mon profil (nom) afin de corriger mes informations.
- Criteres d'acceptation :
  - [ ] Formulaire de modification du nom
  - [ ] L'email n'est pas modifiable (identifiant unique)

### Won't Have (this time)

- **Prix personnalises par utilisateur** : prevu en V2. En V1, tous les utilisateurs partagent les memes materiaux et tarifs. En V2, les prix personnalises seront lies au `user_id` (pas de notion de tenant/organisation).
- **Notifications email** : pas d'envoi d'email a l'inscription, a la validation, ou a la desactivation. L'utilisateur doit revenir verifier manuellement.
- **Reset de mot de passe / mot de passe oublie** : hors scope V1. L'admin peut eventuellement reinitialiser un compte manuellement.
- **OAuth / SSO / connexion tierce** : pas de login Google, GitHub, etc.
- **Roles multiples** : il n'y a que deux roles en V1 : utilisateur et admin. Pas de roles intermediaires.
- **Tableau de bord admin avec statistiques** : hors scope V1.
- **Suppression de compte par l'utilisateur lui-meme** : hors scope V1.

## Flux principaux

### Flux 1 : Inscription
1. Le visiteur clique sur "S'inscrire" depuis la page d'accueil
2. Il remplit le formulaire (nom, email, mot de passe, confirmation mot de passe)
3. Le systeme valide les champs (email unique, mot de passe conforme)
4. Le systeme cree le compte avec le statut "pending"
5. L'utilisateur voit un message de confirmation "Compte cree, en attente de validation"
- Cas d'erreur : email deja pris -> message "Un compte existe deja avec cet email"
- Cas d'erreur : mot de passe trop faible -> message avec les criteres manquants
- Cas limite : double soumission -> idempotent, pas de doublon

### Flux 2 : Validation admin
1. L'admin se connecte a l'interface admin (existant)
2. Il voit une notification ou un compteur "X comptes en attente"
3. Il accede a la section "Gestion des comptes"
4. Il voit la liste des comptes pending
5. Il approuve ou refuse chaque compte
6. Le statut du compte est mis a jour immediatement
- Cas d'erreur : le compte a deja ete traite par un autre admin -> message d'erreur
- Cas limite : aucun compte en attente -> section vide avec message "Aucun compte en attente"

### Flux 3 : Connexion et creation de devis
1. L'utilisateur accede a la page de connexion
2. Il saisit email + mot de passe
3. Le systeme verifie les identifiants et le statut du compte
4. Si OK, l'utilisateur est connecte et redirige vers la page d'accueil
5. Il peut uploader des modeles, selectionner des materiaux, et generer des devis
6. Les devis sont lies a son compte
7. Il peut consulter ses devis passes dans "Mes devis"
- Cas d'erreur : identifiants incorrects -> "Email ou mot de passe incorrect"
- Cas d'erreur : compte pending -> "Compte en attente de validation"
- Cas d'erreur : compte desactive -> "Compte desactive, contactez l'administrateur"

### Flux 4 : Mode demo (visiteur)
1. Le visiteur accede a la page d'accueil sans etre connecte
2. Il peut voir les materiaux et uploader un modele 3D
3. Il voit la visualisation 3D et les informations du modele (volume, dimensions)
4. Il ne peut pas generer de devis
5. Un message l'invite a s'inscrire pour obtenir un devis
- Cas limite : si le visiteur a uploade des modeles en demo puis s'inscrit, les modeles de la session demo ne sont PAS transferes vers le compte (session jetable)

## Contraintes business

- **Compatibilite ascendante** : le flux d'upload et de visualisation 3D ne doit pas etre degrade. Le mode demo doit offrir la meme experience que l'existant (sauf generation de devis).
- **Performance** : l'ajout de l'authentification ne doit pas ralentir les pages publiques.
- **Securite** : les mots de passe doivent etre hashes correctement (jamais en clair). Les sessions authentifiees doivent etre securisees (cookies HttpOnly, SameSite).
- **Preparation V2** : l'architecture doit prevoir un champ ou une table permettant l'isolation forte future (tenant_id, prix par tenant), mais sans l'implementer en V1.

## Questions ouvertes

- **Notification de validation** : en l'absence d'email, comment l'utilisateur sait-il que son compte a ete valide ? Reponse retenue pour V1 : il doit retenter de se connecter. Un message adapte lui indique le statut.
- **Migration des sessions existantes** : les sessions anonymes existantes en base ne seront pas migrees vers des comptes. Elles continueront a expirer normalement.
- **Limites par utilisateur** : y a-t-il un nombre maximum de devis ou de modeles par utilisateur ? Non defini pour V1, pas de limite.
