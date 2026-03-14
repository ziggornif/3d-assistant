# UX Guidelines -- Feature 0001 : Gestion de comptes

## Design Overview

Les nouvelles pages (login, register, my-quotes, gestion comptes admin) s'integrent dans le design system existant sans introduire de nouveaux patterns. Toutes les pages reutilisent les composants et variables CSS deja en place : `section` cards avec `shadow-md`, formulaires avec `.form-group`, boutons `.btn-primary` / `.btn-secondary`, badges `.status-badge`, table pattern de l'admin. La navigation du header est enrichie pour refleter l'etat d'authentification. Le respect de WCAG 2.1 AA est maintenu (contrastes, focus visible, touch targets 44px, `prefers-reduced-motion`).

## Audit du design system existant

### Composants reutilisables pour cette feature

| Composant existant | Reutilisation |
|-------------------|---------------|
| `section` card (white bg, shadow-md, border-radius-lg, padding-xl) | Conteneur pour tous les formulaires et listes |
| `.form-group` (label + input vertical) | Formulaires login et register |
| `.form-row` (grid 2 colonnes) | Non utilise dans cette feature (formulaires simples) |
| `.btn-primary` / `.btn-secondary` | Boutons d'action dans toutes les pages |
| `.btn-small` | Actions par ligne dans les tables admin |
| `.error-message` (background rouge translucide) | Messages d'erreur des formulaires |
| `.success-message` | Message de confirmation post-inscription |
| `.status-badge` + `.status-active` / `.status-inactive` | Statuts des comptes (active/pending/disabled) |
| `.admin-card` (white bg, border, border-radius-lg) | Section gestion des comptes dans l'admin |
| Table pattern (`.materials-table`) | Liste des comptes admin |
| Modal pattern (`.modal` / `.modal-content`) | Non necessaire pour cette feature |
| Notification toast (`#notification-container`) | Feedback apres actions admin (valider/refuser compte) |
| `.skip-link` | Maintenu sur toutes les nouvelles pages |
| `.header-content` / `.header-nav` | Navigation enrichie |

### Nouveaux composants necessaires

**Aucun nouveau composant a creer.** Tout peut etre construit avec les patterns existants. Les seuls ajouts sont des variantes CSS mineures :
- `.status-pending` (badge pour statut "en attente") -- meme pattern que `.status-active` / `.status-inactive`
- `.status-rejected` (badge pour statut "refuse")
- `.status-disabled` (badge pour statut "desactive")
- `.auth-nav` -- variante de `.header-nav` pour les liens d'authentification
- `.cta-banner` -- banniere d'appel a action pour le mode demo (reutilise le pattern `.error-banner` mais avec la couleur primary)

## User Flows

### Flow 1 : Inscription

1. **Page d'accueil (/)** -- Le visiteur voit le header avec les liens "Connexion" et "S'inscrire" dans la navigation
2. **Clic sur "S'inscrire"** -- Navigation vers `/register`
3. **Page /register** -- Formulaire centre (max-width 400px, comme `#login-section` admin)
   - Champ "Nom d'affichage" (text, required)
   - Champ "Email" (email, required)
   - Champ "Mot de passe" (password, required) -- avec hint "8 caracteres minimum, 1 majuscule, 1 chiffre"
   - Champ "Confirmer le mot de passe" (password, required)
   - Bouton "Creer mon compte" (`.btn-primary`, full width)
   - Lien "Deja inscrit ? Se connecter" sous le bouton
4. **Soumission valide** -- Le formulaire est remplace par un message de succes :
   - Icone de succes (checkmark) + texte "Votre compte a ete cree avec succes."
   - Sous-texte : "Un administrateur doit valider votre compte avant que vous puissiez vous connecter."
   - Bouton "Retour a l'accueil" (`.btn-secondary`)
5. **Soumission avec erreur** -- Messages d'erreur inline sous chaque champ concerne (`.field-error` + `.field-error-message`)
   - Email deja utilise : "Un compte existe deja avec cet email"
   - Mot de passe trop faible : "Le mot de passe doit contenir au moins 8 caracteres, une majuscule et un chiffre"
   - Mots de passe differents : "Les mots de passe ne correspondent pas"
   - Etat loading : bouton desactive + texte "Creation en cours..."

### Flow 2 : Connexion

1. **Clic sur "Connexion"** depuis le header -- Navigation vers `/login`
2. **Page /login** -- Formulaire centre (max-width 400px, meme layout que le login admin existant)
   - Champ "Email" (email, required)
   - Champ "Mot de passe" (password, required)
   - Bouton "Se connecter" (`.btn-primary`, full width)
   - Lien "Pas encore de compte ? S'inscrire" sous le bouton
3. **Connexion reussie** -- Redirection vers `/` avec le header mis a jour (nom utilisateur + liens)
4. **Erreur d'identifiants** -- Message d'erreur au-dessus du formulaire (`.error-message`)
   - "Email ou mot de passe incorrect"
5. **Compte en attente** -- Message d'information au-dessus du formulaire
   - Fond bleu translucide (pattern `.notification-info`)
   - "Votre compte est en attente de validation par l'administrateur."
6. **Compte desactive** -- Message d'erreur au-dessus du formulaire
   - "Votre compte a ete desactive. Contactez l'administrateur."
7. **Etat loading** -- Bouton desactive + texte "Connexion en cours..."

### Flow 3 : Navigation authentifiee (header)

Le header (`templates/base.html`) change selon l'etat d'authentification :

**Visiteur (non connecte)** :
```
[Logo/Titre]                    [Connexion] [S'inscrire]
```
- "Connexion" : lien texte (`.btn-secondary` style compact)
- "S'inscrire" : bouton (`.btn-primary` style compact, meme style que `.admin-link` actuel)

**Utilisateur connecte** :
```
[Logo/Titre]               [Mes devis] [Nom] [Deconnexion]
```
- "Mes devis" : lien texte dans la nav
- "Nom" : texte d'affichage (non cliquable, couleur `--color-text-light`)
- "Deconnexion" : lien texte (`.btn-secondary` style compact)

**Admin connecte (page /admin)** : inchange (le login admin reste distinct)

### Flow 4 : Mes devis (/my-quotes)

1. **Acces** -- Clic sur "Mes devis" dans le header (utilisateur connecte)
2. **Etat vide** -- Si aucun devis :
   - Message centre : "Vous n'avez pas encore de devis."
   - Bouton "Creer un devis" (`.btn-primary`) qui redirige vers `/`
3. **Liste des devis** -- Affichage en cards (meme pattern que `.model-card`)
   - Chaque card affiche :
     - Date du devis (format "14 mars 2026")
     - Nombre de modeles (ex: "3 modeles")
     - Prix total (format euro, vert `--color-success`, bold)
     - Statut (badge `.status-badge`)
   - Cards triees par date (plus recents en premier)
   - Clic sur une card : navigation vers le detail du devis
4. **Detail d'un devis** -- Reutilise exactement le meme layout que le composant `<quote-summary>` existant mais en lecture seule (pas de bouton "Finaliser")
   - Header bleu avec "Devis #XXXX"
   - Liste des items (modele, materiau, volume, prix)
   - Totaux (sous-total, frais, total)
   - Bouton "Retour a mes devis" (`.btn-secondary`)
5. **Etat loading** -- Skeleton loader ou texte "Chargement de vos devis..."
6. **Etat erreur** -- Banner d'erreur (`.error-banner`) + bouton "Reessayer"

### Flow 5 : Mode demo (visiteur)

Le comportement actuel est modifie pour les visiteurs non connectes :

1. L'upload, la visualisation 3D, et la selection de materiaux fonctionnent comme avant
2. Le composant `<quote-summary>` change son comportement :
   - Le bouton "Finaliser le devis" est remplace par un **CTA banner** :
     - Fond bleu translucide (`--color-primary` a 10% d'opacite)
     - Texte : "Inscrivez-vous pour obtenir votre devis"
     - Bouton "S'inscrire gratuitement" (`.btn-primary`)
     - Lien secondaire "Deja inscrit ? Se connecter"
3. Les calculs de prix restent visibles dans le recap (le visiteur voit les prix estimes) mais ne peut pas generer/sauvegarder le devis final

**Utilisateur connecte apres generation du devis** : Apres la generation reussie d'un devis (POST quote), un encart de rappel est affiche en bas du composant `<quote-summary>`, sous le numero de devis :
- Fond `--color-background` (#f8fafc), padding 1rem, texte centre, font-size-sm
- Texte : "Retrouvez ce devis et tous les autres dans"
- Lien "Mes devis" (couleur `--color-primary`, underline) pointant vers `/my-quotes`
- Cet encart n'est affiche QUE si l'utilisateur est connecte (attribut `authenticated` sur le composant) ET qu'un devis a ete genere (quote_id present)

### Flow 6 : Gestion des comptes (admin)

1. **Acces** -- L'admin se connecte via le login existant (token admin, inchange)
2. **Page /admin enrichie** -- Nouvelle section "Gestion des comptes" ajoutee AVANT la section "Gestion des Materiaux"
3. **Section "Gestion des comptes"** :
   - Meme pattern que `.admin-card`
   - Titre "Gestion des Comptes" avec compteur de comptes en attente : "(3 en attente)" en couleur `--color-warning`
   - Filtres par onglets : "En attente" | "Actifs" | "Desactives" | "Tous"
     - Onglets implementes comme des boutons inline (pas de tab component nouveau, utiliser des boutons avec active state via border-bottom `--color-primary`)
   - Table (meme pattern que `.materials-table`) :
     - Colonnes : Nom | Email | Date d'inscription | Statut | Actions
     - Le statut est affiche avec des badges :
       - `.status-pending` : fond orange translucide, texte orange (meme pattern que `.status-badge`)
       - `.status-active` : fond vert translucide, texte vert (existant)
       - `.status-disabled` : fond gris translucide, texte gris
       - `.status-rejected` : fond rouge translucide, texte rouge (meme que `.status-inactive`)
     - Actions par ligne :
       - Compte pending : boutons "Valider" (`.btn-primary` small) + "Refuser" (`.btn-secondary` small)
       - Compte actif : bouton "Desactiver" (`.btn-secondary` small)
       - Compte desactive : bouton "Reactiver" (`.btn-primary` small)
   - Etat vide (aucun compte dans le filtre actif) : "Aucun compte a afficher"
   - Feedback apres action : notification toast (pattern existant `#notification-container`)
     - "Compte valide avec succes" (notification-success)
     - "Compte refuse" (notification-info)
     - "Compte desactive" (notification-info)
     - "Compte reactive" (notification-success)

## Component Specifications

### Formulaire d'authentification (login / register)

- **Structure** : `section` card (white bg, shadow-md) centree, max-width 400px, meme layout que `#login-section` admin existant
- **Etats** :
  - Default : formulaire vide avec placeholders
  - Focus : border `--color-primary` + box-shadow bleu (existant)
  - Loading : bouton desactive, texte "En cours...", formulaire non editable (classe `.loading`)
  - Error : messages inline sous les champs (`.field-error` + `.field-error-message`), message global au-dessus du form (`.error-message`)
  - Success (register uniquement) : formulaire remplace par message de confirmation
- **Responsive** :
  - Mobile : padding reduit, full width
  - Desktop : centre avec max-width 400px
- **Accessibilite** :
  - Labels associes aux inputs via `for`/`id`
  - `aria-describedby` pour les hints (ex: criteres mot de passe)
  - `aria-invalid="true"` + `aria-errormessage` sur les champs en erreur
  - `role="alert"` sur les messages d'erreur globaux
  - `autocomplete` attributes : `email`, `new-password`, `current-password`, `name`
  - Focus trap : pas necessaire (pas de modal)

### Header conditionnel

- **Structure** : `.header-content` existant (flex, space-between), `.header-nav` (flex, gap)
- **Etats** :
  - Visiteur : liens "Connexion" (secondary) + "S'inscrire" (primary)
  - Utilisateur connecte : lien "Mes devis" + nom afiche + "Deconnexion" (secondary)
  - Admin : inchange (le header admin est sur `/admin`, page separee)
- **Responsive** :
  - Mobile : les liens de navigation passent en dessous du titre (flex-wrap)
  - Le nom d'utilisateur est tronque si trop long (text-overflow: ellipsis, max-width: 150px)
- **Accessibilite** :
  - Navigation dans un `<nav>` avec `aria-label="Navigation principale"`
  - Lien actif avec `aria-current="page"` (pattern deja dans accessibility.css)

### CTA banner mode demo

- **Structure** : div dans le composant `<quote-summary>`, a la place du bouton "Finaliser"
- **Style** : fond `rgba(37, 99, 235, 0.08)`, border `1px solid rgba(37, 99, 235, 0.2)`, border-radius, padding 1.5rem, texte centre
- **Contenu** : texte incitatif + bouton "S'inscrire gratuitement" (`.btn-primary`) + lien "Se connecter"
- **Accessibilite** : `role="complementary"`, texte explicite

### Liste des devis (my-quotes)

- **Structure** : page avec titre "Mes devis" + liste de cards
- **Chaque card** : meme pattern que `.model-card` (white bg, border, border-radius-lg, padding, shadow-sm)
  - Layout flex : infos a gauche, prix a droite
  - Date en `--color-text-light`, font-size-sm
  - Prix en `--color-success`, font-weight 700
  - Badge statut (`.status-badge`)
- **Etats** :
  - Default : liste de cards
  - Empty : message centre + CTA "Creer un devis"
  - Loading : texte "Chargement de vos devis..."
  - Error : `.error-banner` + bouton "Reessayer"
- **Responsive** :
  - Mobile : cards en full width
  - Desktop : cards en full width aussi (liste verticale simple)
- **Accessibilite** :
  - Cards cliquables : `role="link"` ou `<a>` wrapper
  - `aria-label` sur chaque card : "Devis du [date], [X] modeles, [prix]"

### Table des comptes (admin)

- **Structure** : reutilise le pattern `.materials-table` exact
- **Etats** :
  - Default : table avec donnees
  - Empty : "Aucun compte a afficher"
  - Loading : "Chargement des comptes..."
  - Apres action : notification toast + mise a jour inline de la ligne
- **Nouveaux badges de statut** (CSS a ajouter, meme pattern) :
  - `.status-pending` : `background: rgba(217, 119, 6, 0.1); color: #d97706;`
  - `.status-disabled` : `background: rgba(100, 116, 139, 0.1); color: #64748b;`
  - `.status-rejected` : meme style que `.status-inactive` existant
- **Accessibilite** :
  - Table avec `<caption>` : "Liste des comptes utilisateurs"
  - `<th scope="col">` pour les en-tetes
  - Boutons d'action avec `aria-label` explicite : "Valider le compte de [nom]"

## Design Annotations pour le frontend-dev

### Fichiers a creer

| Fichier | Description | Pattern de reference |
|---------|-------------|---------------------|
| `templates/login.html` | Page connexion | Copier le layout de `#login-section` dans `admin.html` |
| `templates/register.html` | Page inscription | Meme layout que login, plus de champs |
| `templates/my-quotes.html` | Historique devis | Reutiliser `.model-card` pour les cards devis |
| `static/css/auth.css` | Styles specifiques auth | Minimal -- principalement la confirmation post-register et le CTA demo |
| `static/js/admin/accounts.js` | Logique gestion comptes | Suivre les patterns de `static/js/admin/main.js` |

### Fichiers a modifier

| Fichier | Modification |
|---------|-------------|
| `templates/base.html` | Ajouter navigation conditionnelle dans le header (variable Tera `authenticated_user`) |
| `templates/index.html` | Ajouter `aria-label` et contexte auth au header. Le composant `<quote-summary>` doit recevoir un attribut `demo-mode` si visiteur |
| `templates/admin.html` | Ajouter section "Gestion des comptes" avant la section materiaux |
| `static/css/admin.css` | Ajouter les styles `.status-pending`, `.status-disabled`, `.status-rejected`, et les styles d'onglets de filtre |
| `static/js/components/quote-summary.js` | Ajouter la gestion de l'attribut `demo-mode` (CTA banner visiteur) et de l'attribut `authenticated` (lien "Mes devis" apres generation) |

### Conventions a respecter

1. **Pas de nouveau pattern de formulaire** -- reutiliser `.form-group`, `.form-row`, `.error-message`, `.success-message`
2. **Pas de nouveau composant Web** -- les nouvelles pages sont des templates Tera classiques, pas des Web Components
3. **Le mode demo modifie le composant `<quote-summary>` existant** -- pas de nouveau composant
4. **Les onglets de filtre admin sont des boutons simples** avec un active state (border-bottom primary) -- pas de composant tab complexe
5. **Toutes les nouvelles pages incluent** : skip-link, `<main id="main-content" role="main">`, footer, `lang="fr"`
6. **Les formulaires utilisent `method="POST"` avec des routes SSR** (comme le login admin existant), pas du fetch JS -- sauf pour les actions admin inline (valider/refuser) qui utilisent fetch + toast
