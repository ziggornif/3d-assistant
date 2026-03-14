# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- **User account management** : Registration, login, logout with cookie-based session authentication
- **Password hashing** : Argon2id (OWASP recommended) with timing attack protection
- **Admin account management** : Validate, reject, disable, reactivate user accounts from the admin panel
- **User quote history** : Authenticated users can view their past quotes at `/my-quotes`
- **Demo mode** : Anonymous visitors can upload and preview 3D models but cannot generate quotes (CTA to register)
- **Authenticated sessions** : Logged-in users get 30-day sessions linked to their account (vs 24h anonymous sessions)
- **SSR pages** : `/login`, `/register`, `/my-quotes` with server-side rendering
- **Conditional header navigation** : Visitors see "Connexion" / "S'inscrire", authenticated users see "Mes devis" / name / "Deconnexion"
- **Admin UI** : Account management section with status filters (pending/active/disabled), inline actions, notification toasts
- **Quote summary enhancements** : Demo mode CTA banner for visitors, "Retrouvez ce devis dans Mes devis" reminder for authenticated users
- **New status badges CSS** : `.status-pending` (orange), `.status-disabled` (grey), `.status-rejected` (red)
- **Rate limiting** on `/api/auth/login` and `/api/auth/register`

### Changed

- `quote_sessions` table now includes `user_id` (nullable) and `session_type` (anonymous/authenticated)
- `POST /api/sessions` creates authenticated sessions when user is logged in
- `POST /api/sessions/{id}/quote` returns 403 for anonymous sessions (demo mode restriction)
- `QuoteSession` model extended with `user_id`, `session_type`, `new_authenticated()`, `is_anonymous()`, `is_authenticated()`
- `AppError` enum extended with `Unauthorized`, `Forbidden`, `Conflict`, `Validation` variants

### New API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/auth/register` | User registration (returns 201, status "pending") |
| POST | `/api/auth/login` | User login (sets HttpOnly cookie, rate limited) |
| POST | `/api/auth/logout` | User logout (clears cookie and server session) |
| GET | `/api/auth/me` | Get current authenticated user info |
| GET | `/api/users/me/quotes` | List user's quotes (paginated) |
| GET | `/api/users/me/quotes/{id}` | Get quote detail (ownership verified) |
| GET | `/api/admin/users` | List users with optional status filter (admin) |
| PATCH | `/api/admin/users/{id}` | Update user status (admin) |

### New Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `argon2` | `0.5` | Password hashing (Argon2id) |
| `rand` | `0.8` | Secure token generation |

### Database Migrations

- `007_users.sql` : Creates `users` and `user_sessions` tables
- `008_sessions_user_id.sql` : Adds `user_id` and `session_type` to `quote_sessions` (non-destructive)
