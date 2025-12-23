# Intégration MCP (Model Context Protocol)

Ce document décrit l'intégration du serveur MCP dans le Service de Devis d'Impression 3D, qui permet aux modèles IA et aux outils d'automatisation de générer des devis de manière programmatique sans utiliser l'interface web.

## Vue d'ensemble

Le serveur MCP fournit un ensemble d'outils qui permettent de :
- Uploader des fichiers de modèles 3D (STL/3MF) via des données encodées en base64
- Lister les matériaux disponibles et leurs prix
- Configurer des modèles avec des matériaux et des quantités
- Générer des devis complets pour des sessions

## Architecture

L'implémentation MCP est construite en utilisant la bibliothèque [`rmcp`](https://crates.io/crates/rmcp) et se compose de :

- **`src/mcp/quote_tools.rs`** : Définitions des outils et logique métier
- **`src/mcp/server.rs`** : Configuration et setup du serveur MCP
- **`src/mcp/mod.rs`** : Exports du module

Le serveur MCP est intégré dans l'application principale et exposé sur :
```
POST /mcp
```

## Outils disponibles

### 1. `list_materials`

Liste tous les matériaux d'impression disponibles avec leurs prix.

**Paramètres :** Aucun

**Retourne :**
```json
[
  {
    "id": "pla_standard",
    "name": "PLA Standard",
    "base_price_per_cm3": 0.15,
    "active": true
  },
  {
    "id": "abs_standard",
    "name": "ABS Standard",
    "base_price_per_cm3": 0.18,
    "active": true
  }
]
```

### 2. `upload_model`

Upload un fichier de modèle 3D à partir de données encodées en base64.

**Paramètres :**
```json
{
  "session_id": "01JGXXX...",
  "filename": "cube.stl",
  "file_data": "contenu-du-fichier-encodé-en-base64"
}
```

**Retourne :**
```json
{
  "model_id": "01JGYYY...",
  "filename": "cube.stl",
  "file_format": "stl",
  "volume_cm3": 8.0,
  "dimensions_mm": {
    "x": 20.0,
    "y": 20.0,
    "z": 20.0
  },
  "triangle_count": 12
}
```

**Validation :**
- La taille du fichier ne doit pas dépasser le maximum configuré (50Mo par défaut)
- Le fichier doit être au format STL ou 3MF valide
- Le nom de fichier est automatiquement nettoyé (caractères dangereux supprimés)

**Traitement :**
- Le volume est calculé à partir du maillage de triangles
- Les dimensions (boîte englobante) sont extraites
- Le nombre de triangles est calculé
- Le fichier est sauvegardé sur disque dans un répertoire spécifique à la session

### 3. `configure_model`

Configure un modèle avec la sélection du matériau et de la quantité.

**Paramètres :**
```json
{
  "session_id": "01JGXXX...",
  "model_id": "01JGYYY...",
  "material_id": "pla_standard",
  "quantity": 5
}
```

**Retourne :**
```json
{
  "model_id": "01JGYYY...",
  "material_id": "pla_standard",
  "quantity": 5,
  "estimated_price": 12.50
}
```

**Validation :**
- Le modèle doit exister dans la session spécifiée
- Le matériau doit exister et être actif
- La quantité doit être positive

### 4. `generate_quote`

Génère un devis complet pour tous les modèles configurés dans une session.

**Paramètres :**
```json
{
  "session_id": "01JGXXX..."
}
```

**Retourne :**
```json
{
  "quote_id": "01JGZZZ...",
  "session_id": "01JGXXX...",
  "items": [
    {
      "model_id": "01JGYYY...",
      "filename": "cube.stl",
      "material_name": "PLA Standard",
      "quantity": 1,
      "unit_price": 2.50,
      "line_total": 2.50
    }
  ],
  "subtotal": 2.50,
  "total": 2.50
}
```

**Validation :**
- La session doit avoir au moins un modèle
- Tous les modèles doivent être configurés avec un matériau
- Le devis est persisté dans la base de données

## Exemple d'utilisation

Voici un workflow typique utilisant les outils MCP :

### Étape 1 : Créer une session
D'abord, créez une session via l'API REST :
```bash
curl -X POST http://localhost:3000/api/sessions
```

Réponse :
```json
{
  "session_id": "01JGXXX123",
  "expires_at": "2025-12-24T15:30:00Z"
}
```

### Étape 2 : Lister les matériaux disponibles (MCP)
```json
{
  "method": "tools/call",
  "params": {
    "name": "list_materials"
  }
}
```

### Étape 3 : Uploader un modèle (MCP)
```bash
# D'abord, encodez votre fichier STL en base64
base64 cube.stl > cube_base64.txt
```

```json
{
  "method": "tools/call",
  "params": {
    "name": "upload_model",
    "arguments": {
      "session_id": "01JGXXX123",
      "filename": "cube.stl",
      "file_data": "<contenu-base64>"
    }
  }
}
```

### Étape 4 : Configurer le modèle (MCP)
```json
{
  "method": "tools/call",
  "params": {
    "name": "configure_model",
    "arguments": {
      "session_id": "01JGXXX123",
      "model_id": "01JGYYY456",
      "material_id": "pla_standard",
      "quantity": 5
    }
  }
}
```

### Étape 5 : Générer le devis (MCP)
```json
{
  "method": "tools/call",
  "params": {
    "name": "generate_quote",
    "arguments": {
      "session_id": "01JGXXX123"
    }
  }
}
```

## Tests

Les tests d'intégration sont disponibles dans `tests/mcp_integration_test.rs` :

```bash
# Lancer les tests d'intégration MCP
cargo test --test mcp_integration_test

# Lancer tous les tests
cargo test
```

Les tests couvrent :
- Le listage des matériaux
- L'upload de fichiers STL avec calcul du volume
- La configuration des modèles avec des matériaux
- La génération de devis
- La gestion des erreurs (session invalide, modèles non configurés, etc.)

## Configuration

Le serveur MCP utilise la même configuration que l'application principale :

```env
# .env
DATABASE_URL=postgres://user:password@localhost:5432/quotes
UPLOAD_DIR=./uploads
MAX_FILE_SIZE_MB=50
```

## Considérations de sécurité

1. **Limites de taille de fichier** : La taille d'upload est limitée (50Mo par défaut) pour prévenir les attaques DoS
2. **Validation des fichiers** : Tous les fichiers uploadés sont validés pour le format correct (STL/3MF)
3. **Nettoyage des noms de fichiers** : Les caractères dangereux et tentatives de traversée de chemin sont supprimés
4. **Isolation des sessions** : Les modèles sont isolés par session dans des répertoires séparés
5. **Validation des matériaux** : Seuls les matériaux actifs peuvent être utilisés pour la configuration

## Détails d'implémentation

### Stack technologique
- **rmcp** : Implémentation Rust de MCP avec des macros déclaratives
- **sqlx** : Opérations de base de données avec vérification des requêtes à la compilation
- **base64** : Encodage/décodage de fichiers
- **serde** : Sérialisation/désérialisation JSON
- **schemars** : Génération de schémas JSON pour les paramètres des outils

### Macros utilisées
- `#[tool_router]` : Définit le routeur d'outils pour la structure QuoteTools
- `#[tool]` : Marque les méthodes comme outils MCP avec descriptions
- `#[tool_handler]` : Implémente le trait ServerHandler pour les outils

### Flux de données
```
Client (Modèle IA/Outil)
  ↓ Requête MCP
POST /mcp
  ↓
rmcp StreamableHttpService
  ↓
QuoteTools (exécution de l'outil)
  ↓ Opérations de base de données
PostgreSQL
  ↓ Système de fichiers
./uploads/{session_id}/
```

## Gestion des erreurs

Tous les outils retournent `Result<String, String>` où :
- **Ok(String)** : Résultat de succès sérialisé en JSON
- **Err(String)** : Message d'erreur lisible par l'humain

Erreurs courantes :
- `"Session not found"` : ID de session invalide ou expiré
- `"Invalid base64 encoding"` : Données de fichier mal formées
- `"File size exceeds maximum"` : Fichier trop volumineux
- `"File validation failed"` : Format STL/3MF invalide
- `"Model not found in session"` : ID de modèle invalide
- `"Material not found"` : ID de matériau invalide
- `"Material is not active"` : Matériau inactif sélectionné
- `"No models found in session"` : Session vide pour la génération de devis
- `"X model(s) missing material configuration"` : Modèles non configurés

## Limitations

1. **Pas d'authentification** : Actuellement pas d'authentification sur l'endpoint MCP (à ajouter en production)
2. **Quantité unique** : Le paramètre de quantité est accepté mais pas entièrement implémenté dans le calcul du devis
3. **Pas d'URLs de prévisualisation** : Les uploads MCP ne génèrent pas d'images de prévisualisation 3D
4. **Traitement synchrone** : Le traitement des fichiers est synchrone (peut timeout sur de très gros fichiers)

## Améliorations futures

- [ ] Ajouter l'authentification/autorisation pour l'endpoint MCP
- [ ] Support du streaming pour les uploads de gros fichiers
- [ ] Générer des images de prévisualisation 3D pour les uploads MCP
- [ ] Ajouter un outil pour lister les modèles d'une session
- [ ] Ajouter un outil pour supprimer des modèles
- [ ] Support des opérations par lot (upload de plusieurs modèles)
- [ ] Ajouter le support des métadonnées de modèle (notes, infos client)
- [ ] Notifications webhook lors de la génération de devis

## Références

- [Spécification Model Context Protocol](https://modelcontextprotocol.io/)
- [Implémentation Rust rmcp](https://github.com/EmilLindfors/rmcp)
- [Tests d'intégration MCP](../../../tests/mcp_integration_test.rs)
- [Code source Quote Tools](../../../src/mcp/quote_tools.rs)
