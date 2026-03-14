-- Migration: Add deleted_at to quotes for soft delete
-- Version: 009
-- Description: Support soft delete on quotes (cycle de vie devis)

ALTER TABLE quotes ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP NULL;

CREATE INDEX IF NOT EXISTS idx_quotes_deleted_at ON quotes(deleted_at);
