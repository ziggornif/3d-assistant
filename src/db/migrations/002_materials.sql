-- Migration: Create materials table
-- Version: 002
-- Description: Materials available for printing with pricing

CREATE TABLE IF NOT EXISTS materials (
    id TEXT PRIMARY KEY NOT NULL,
    service_type_id TEXT NOT NULL REFERENCES service_types(id),
    name TEXT NOT NULL,
    description TEXT,
    price_per_cm3 REAL NOT NULL,
    color TEXT,
    properties TEXT, -- JSON string
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_materials_service_type ON materials(service_type_id);
CREATE INDEX IF NOT EXISTS idx_materials_active ON materials(active);
