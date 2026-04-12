ALTER TABLE tracks ADD COLUMN has_plain_lyrics BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE tracks ADD COLUMN has_synced_lyrics BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE tracks ADD COLUMN has_word_synced_lyrics BOOLEAN NOT NULL DEFAULT 0;

CREATE INDEX idx_tracks_has_plain_lyrics ON tracks(has_plain_lyrics);
CREATE INDEX idx_tracks_has_synced_lyrics ON tracks(has_synced_lyrics);
CREATE INDEX idx_tracks_has_word_synced_lyrics ON tracks(has_word_synced_lyrics);

/* Force reset the scanned tracks in the database since v7->v10 migrations have many major changes */
DELETE FROM tracks WHERE 1;
DELETE FROM albums WHERE 1;
DELETE FROM artists WHERE 1;
UPDATE library_data SET init = 0 WHERE 1;
