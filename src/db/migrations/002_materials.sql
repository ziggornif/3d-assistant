-- Migration: Create materials table
-- Version: 002
-- Description: Materials available for printing with pricing

CREATE TABLE IF NOT EXISTS materials (
    id TEXT PRIMARY KEY NOT NULL,
    service_type_id TEXT NOT NULL REFERENCES service_types(id),
    name TEXT NOT NULL,
    description TEXT,
    price_per_cm3 DOUBLE PRECISION NOT NULL,
    color TEXT,
    properties TEXT, -- JSON string
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_materials_service_type ON materials(service_type_id);
CREATE INDEX IF NOT EXISTS idx_materials_active ON materials(active);
