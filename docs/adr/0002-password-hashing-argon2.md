# ADR 0002 : Hashing des mots de passe avec Argon2id

## Statut
Accepte

## Date
2026-03-14

## Contexte
Les utilisateurs vont s'inscrire avec un mot de passe. Il faut choisir un algorithme de hashing securise pour stocker ces mots de passe en base.

### Options envisagees

**Option A : Argon2id (crate `argon2`)**
- Standard recommande par OWASP (2023+)
- Resistance aux attaques GPU et ASIC (memory-hard)
- Parametres configurables (memoire, iterations, parallelisme)
- Crate Rust mature et auditee

**Option B : bcrypt (crate `bcrypt`)**
- Standard historique, largement utilise
- Pas de resistance memoire (vulnerable aux attaques GPU modernes)
- Limite a 72 octets de mot de passe
- Plus ancien, moins recommande par OWASP pour les nouveaux projets

**Option C : scrypt**
- Memory-hard comme Argon2
- Moins configurable, moins largement adopte en Rust
- Pas le standard OWASP recommande

## Decision
**Option A : Argon2id** avec les parametres OWASP recommandes :
- Memory: 19456 KiB (19 MiB)
- Iterations: 2
- Parallelism: 1
- Salt: 16 bytes random (genere automatiquement par le crate)

## Consequences
- Securite maximale contre les attaques par dictionnaire et brute force
- Chaque hash prend environ 100-200ms (acceptable pour login, protection naturelle contre brute force)
- La dependance `argon2` est ajoutee au Cargo.toml
- La dependance `rand` est ajoutee pour la generation de tokens de session
- Les mots de passe ne sont jamais stockes en clair ni loggues
