-- Migration 14: Remove deprecated lyrics columns from tracks table
-- Note: SQLite 3.35.0+ supports ALTER TABLE DROP COLUMN
-- Since we're forcing a database reset, we don't need to migrate data

-- Drop indexes on deprecated columns
DROP INDEX IF EXISTS idx_tracks_has_plain_lyrics;
DROP INDEX IF EXISTS idx_tracks_has_synced_lyrics;
DROP INDEX IF EXISTS idx_tracks_has_word_synced_lyrics;

-- Drop deprecated columns from tracks table (SQLite 3.35.0+)
ALTER TABLE tracks DROP COLUMN txt_lyrics;
ALTER TABLE tracks DROP COLUMN lrc_lyrics;
ALTER TABLE tracks DROP COLUMN instrumental;
ALTER TABLE tracks DROP COLUMN has_plain_lyrics;
ALTER TABLE tracks DROP COLUMN has_synced_lyrics;
ALTER TABLE tracks DROP COLUMN has_word_synced_lyrics;

/* Force reset the scanned tracks in the database since v7->v14 migrations have many major changes */
DELETE FROM tracks WHERE 1;
DELETE FROM albums WHERE 1;
DELETE FROM artists WHERE 1;
UPDATE library_data SET init = 0 WHERE 1;
