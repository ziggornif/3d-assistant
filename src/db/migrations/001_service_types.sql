-- Migration: Create service_types table
-- Version: 001
-- Description: Service types for extensible service offerings

CREATE TABLE IF NOT EXISTS service_types (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_service_types_active ON service_types(active);
