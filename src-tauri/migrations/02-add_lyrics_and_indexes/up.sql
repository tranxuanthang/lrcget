ALTER TABLE tracks ADD txt_lyrics TEXT;
CREATE INDEX idx_tracks_title ON tracks(title);
CREATE INDEX idx_albums_name ON albums(name);
CREATE INDEX idx_artists_name ON artists(name);
