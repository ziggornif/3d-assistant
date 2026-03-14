-- Migration: Add user_id and session_type to quote_sessions
-- Version: 008
-- Description: Link sessions to user accounts, distinguish anonymous from authenticated

ALTER TABLE quote_sessions ADD COLUMN IF NOT EXISTS user_id TEXT REFERENCES users(id);
ALTER TABLE quote_sessions ADD COLUMN IF NOT EXISTS session_type TEXT NOT NULL DEFAULT 'anonymous';

CREATE INDEX IF NOT EXISTS idx_quote_sessions_user_id ON quote_sessions(user_id);
