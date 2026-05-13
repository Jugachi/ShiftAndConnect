-- Add migration script here
ALTER TABLE games ADD COLUMN is_private BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE games ADD COLUMN password VARCHAR(10);