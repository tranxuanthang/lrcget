ALTER TABLE config_data ADD skip_tracks_with_synced_lyrics BOOLEAN DEFAULT 0;
ALTER TABLE config_data ADD skip_tracks_with_plain_lyrics BOOLEAN DEFAULT 0;
UPDATE config_data SET skip_tracks_with_synced_lyrics = skip_not_needed_tracks;
ALTER TABLE config_data DROP COLUMN skip_not_needed_tracks;
