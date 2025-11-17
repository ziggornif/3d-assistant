-- Migration: Create pricing_history table
-- Version: 006
-- Description: Audit trail for pricing changes

CREATE TABLE IF NOT EXISTS pricing_history (
    id TEXT PRIMARY KEY NOT NULL,
    material_id TEXT NOT NULL REFERENCES materials(id),
    old_price REAL,
    new_price REAL NOT NULL,
    changed_by TEXT,
    changed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_pricing_history_material ON pricing_history(material_id);
CREATE INDEX IF NOT EXISTS idx_pricing_history_changed_at ON pricing_history(changed_at);
