-- Migration: Create uploaded_models table
-- Version: 004
-- Description: 3D models uploaded by users

CREATE TABLE IF NOT EXISTS uploaded_models (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES quote_sessions(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    file_format TEXT NOT NULL CHECK (file_format IN ('stl', '3mf')),
    file_size_bytes BIGINT NOT NULL,
    volume_cm3 DOUBLE PRECISION,
    dimensions_mm TEXT, -- JSON: {x: float, y: float, z: float}
    triangle_count BIGINT,
    material_id TEXT REFERENCES materials(id),
    file_path TEXT NOT NULL,
    preview_url TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    support_analysis TEXT -- JSON: {needs_support: bool, overhang_percentage: float, estimated_support_material_percentage: float}
);

CREATE INDEX IF NOT EXISTS idx_uploaded_models_session ON uploaded_models(session_id);
CREATE INDEX IF NOT EXISTS idx_uploaded_models_material ON uploaded_models(material_id);
