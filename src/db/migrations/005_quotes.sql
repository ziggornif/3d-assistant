-- Migration: Create quotes table
-- Version: 005
-- Description: Generated price quotes

CREATE TABLE IF NOT EXISTS quotes (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES quote_sessions(id),
    total_price DOUBLE PRECISION NOT NULL,
    breakdown TEXT NOT NULL, -- JSON: itemized costs
    status TEXT NOT NULL DEFAULT 'generated', -- Quote status: generated, accepted, rejected
    created_at TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_quotes_session ON quotes(session_id);
