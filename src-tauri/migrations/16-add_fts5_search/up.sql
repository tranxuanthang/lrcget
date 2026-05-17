CREATE VIRTUAL TABLE tracks_fts USING fts5(
    track_id UNINDEXED,
    title,
    artist_name,
    album_name
);

CREATE VIRTUAL TABLE albums_fts USING fts5(
    album_id UNINDEXED,
    album_name,
    album_artist_name
);

CREATE VIRTUAL TABLE artists_fts USING fts5(
    artist_id UNINDEXED,
    artist_name
);

INSERT INTO tracks_fts (track_id, title, artist_name, album_name)
SELECT
    t.id,
    t.title_lower,
    a.name_lower,
    al.name_lower
FROM tracks t
JOIN artists a ON t.artist_id = a.id
JOIN albums al ON t.album_id = al.id;

INSERT INTO albums_fts (album_id, album_name, album_artist_name)
SELECT
    id,
    name_lower,
    album_artist_name_lower
FROM albums;

INSERT INTO artists_fts (artist_id, artist_name)
SELECT
    id,
    name_lower
FROM artists;
