-- Migration: Create pricing_history table
-- Version: 006
-- Description: Audit trail for pricing changes

CREATE TABLE IF NOT EXISTS pricing_history (
    id TEXT PRIMARY KEY NOT NULL,
    material_id TEXT NOT NULL REFERENCES materials(id),
    old_price DOUBLE PRECISION,
    new_price DOUBLE PRECISION NOT NULL,
    changed_by TEXT,
    changed_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pricing_history_material ON pricing_history(material_id);
CREATE INDEX IF NOT EXISTS idx_pricing_history_changed_at ON pricing_history(changed_at);
