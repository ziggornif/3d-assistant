-- Migration: Create quote_sessions table
-- Version: 003
-- Description: User sessions for quote generation

CREATE TABLE IF NOT EXISTS quote_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    status TEXT NOT NULL DEFAULT 'active'
);

CREATE INDEX IF NOT EXISTS idx_quote_sessions_status ON quote_sessions(status);
CREATE INDEX IF NOT EXISTS idx_quote_sessions_expires ON quote_sessions(expires_at);
