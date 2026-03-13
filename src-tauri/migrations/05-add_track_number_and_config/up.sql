ALTER TABLE tracks ADD track_number INTEGER;
ALTER TABLE albums ADD album_artist_name TEXT;
ALTER TABLE albums ADD album_artist_name_lower TEXT;
ALTER TABLE config_data ADD theme_mode TEXT DEFAULT 'auto';
ALTER TABLE config_data ADD lrclib_instance TEXT DEFAULT 'https://lrclib.net';
CREATE INDEX idx_albums_album_artist_name_lower ON albums(album_artist_name_lower);
CREATE INDEX idx_tracks_track_number ON tracks(track_number);

DELETE FROM tracks WHERE 1;
DELETE FROM albums WHERE 1;
DELETE FROM artists WHERE 1;
UPDATE library_data SET init = 0 WHERE 1;
