CREATE TABLE lyricsfiles (
    id INTEGER PRIMARY KEY,
    track_id INTEGER UNIQUE,
    track_title TEXT,
    track_title_lower TEXT,
    track_album_name TEXT,
    track_album_name_lower TEXT,
    track_artist_name TEXT,
    track_artist_name_lower TEXT,
    track_duration FLOAT,
    lyricsfile TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(track_id) REFERENCES tracks(id) ON DELETE SET NULL
);

CREATE INDEX idx_lyricsfiles_track_title_lower ON lyricsfiles(track_title_lower);
CREATE INDEX idx_lyricsfiles_track_album_name_lower ON lyricsfiles(track_album_name_lower);
CREATE INDEX idx_lyricsfiles_track_artist_name_lower ON lyricsfiles(track_artist_name_lower);
