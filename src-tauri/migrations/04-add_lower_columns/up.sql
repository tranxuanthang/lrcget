ALTER TABLE tracks ADD title_lower TEXT;
ALTER TABLE albums ADD name_lower TEXT;
ALTER TABLE artists ADD name_lower TEXT;
CREATE INDEX idx_tracks_title_lower ON tracks(title_lower);
CREATE INDEX idx_albums_name_lower ON albums(name_lower);
CREATE INDEX idx_artists_name_lower ON artists(name_lower);
