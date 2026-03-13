CREATE TABLE directories (
    id INTEGER PRIMARY KEY,
    path TEXT
);

CREATE TABLE library_data (
    id INTEGER PRIMARY KEY,
    init BOOLEAN
);

CREATE TABLE config_data (
    id INTEGER PRIMARY KEY,
    skip_not_needed_tracks BOOLEAN,
    try_embed_lyrics BOOLEAN
);

CREATE TABLE artists (
    id INTEGER PRIMARY KEY,
    name TEXT
);

CREATE TABLE albums (
    id INTEGER PRIMARY KEY,
    name TEXT,
    artist_id INTEGER,
    image_path TEXT,
    FOREIGN KEY(artist_id) REFERENCES artists(id)
);

CREATE TABLE tracks (
    id INTEGER PRIMARY KEY,
    file_path TEXT,
    file_name TEXT,
    title TEXT,
    album_id INTEGER,
    artist_id INTEGER,
    duration FLOAT,
    lrc_lyrics TEXT,
    FOREIGN KEY(artist_id) REFERENCES artists(id),
    FOREIGN KEY(album_id) REFERENCES albums(id)
);

INSERT INTO library_data (init) VALUES (0);
INSERT INTO config_data (skip_not_needed_tracks, try_embed_lyrics) VALUES (1, 0);
