-- Add lyrics presence fields to lyricsfiles table for unified lyrics management
ALTER TABLE lyricsfiles ADD COLUMN has_plain_lyrics BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE lyricsfiles ADD COLUMN has_synced_lyrics BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE lyricsfiles ADD COLUMN has_word_synced_lyrics BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE lyricsfiles ADD COLUMN instrumental BOOLEAN NOT NULL DEFAULT 0;

-- Create indexes for efficient querying
CREATE INDEX idx_lyricsfiles_has_plain_lyrics ON lyricsfiles(has_plain_lyrics);
CREATE INDEX idx_lyricsfiles_has_synced_lyrics ON lyricsfiles(has_synced_lyrics);
CREATE INDEX idx_lyricsfiles_has_word_synced_lyrics ON lyricsfiles(has_word_synced_lyrics);
CREATE INDEX idx_lyricsfiles_instrumental ON lyricsfiles(instrumental);

-- Note: We keep the fields in tracks table for backward compatibility during transition
-- Eventually, tracks table fields can be removed once all code uses lyricsfiles table
