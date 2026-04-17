-- Add LRCLIB columns to lyricsfiles table for tracking remote lyrics sources
ALTER TABLE lyricsfiles ADD lrclib_instance TEXT;
ALTER TABLE lyricsfiles ADD lrclib_id INTEGER;

-- Create indexes for LRCLIB lookups
CREATE INDEX idx_lyricsfiles_lrclib_instance ON lyricsfiles(lrclib_instance);
CREATE INDEX idx_lyricsfiles_lrclib_id ON lyricsfiles(lrclib_id);

-- Create unique index for composite key (for UPSERT operations)
CREATE UNIQUE INDEX idx_lyricsfiles_lrclib_composite ON lyricsfiles(lrclib_instance, lrclib_id);
