use crate::fs_track;
use crate::persistent_entities::{
    PersistentAlbum, PersistentArtist, PersistentConfig, PersistentTrack,
};
use crate::scanner::models::DbTrack;
use crate::utils::prepare_input;
use anyhow::Result;
use indoc::indoc;
use regex::Regex;
use rusqlite::{named_params, params, Connection, OptionalExtension};
use std::fs;
use tauri::{AppHandle, Manager};

const CURRENT_DB_VERSION: u32 = 8;

/// Initializes the database connection, creating the .sqlite file if needed, and upgrading the database
/// if it's out of date.
pub fn initialize_database(app_handle: &AppHandle) -> Result<Connection, rusqlite::Error> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("The app data directory should exist.");
    fs::create_dir_all(&app_dir).expect("The app data directory should be created.");
    let sqlite_path = app_dir.join("db.sqlite3");

    println!("Database file path: {}", sqlite_path.display());

    let mut db = Connection::open(sqlite_path)?;

    let mut user_pragma = db.prepare("PRAGMA user_version")?;
    let existing_user_version: u32 = user_pragma.query_row([], |row| Ok(row.get(0)?))?;
    drop(user_pragma);

    upgrade_database_if_needed(&mut db, existing_user_version)?;

    Ok(db)
}

/// Upgrades the database to the current version.
pub fn upgrade_database_if_needed(
    db: &mut Connection,
    existing_version: u32,
) -> Result<(), rusqlite::Error> {
    println!("Existing database version: {}", existing_version);

    if existing_version < CURRENT_DB_VERSION {
        if existing_version <= 0 {
            println!("Migrate database version 1...");
            db.pragma_update(None, "journal_mode", "WAL")?;

            let tx = db.transaction()?;

            tx.pragma_update(None, "user_version", 1)?;

            tx.execute_batch(indoc! {"
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
            "})?;

            tx.commit()?;
        }

        if existing_version <= 1 {
            println!("Migrate database version 2...");
            db.pragma_update(None, "journal_mode", "WAL")?;

            let tx = db.transaction()?;

            tx.pragma_update(None, "user_version", 2)?;

            tx.execute_batch(indoc! {"
            ALTER TABLE tracks ADD txt_lyrics TEXT;
            CREATE INDEX idx_tracks_title ON tracks(title);
            CREATE INDEX idx_albums_name ON albums(name);
            CREATE INDEX idx_artists_name ON artists(name);
            "})?;
            tx.commit()?;
        }

        if existing_version <= 2 {
            println!("Migrate database version 3...");
            let tx = db.transaction()?;

            tx.pragma_update(None, "user_version", 3)?;

            tx.execute_batch(indoc! {"
            ALTER TABLE tracks ADD instrumental BOOLEAN;
            "})?;
            tx.commit()?;
        }

        if existing_version <= 3 {
            println!("Migrate database version 4...");
            let tx = db.transaction()?;

            tx.pragma_update(None, "user_version", 4)?;

            tx.execute_batch(indoc! {"
            ALTER TABLE tracks ADD title_lower TEXT;
            ALTER TABLE albums ADD name_lower TEXT;
            ALTER TABLE artists ADD name_lower TEXT;
            CREATE INDEX idx_tracks_title_lower ON tracks(title_lower);
            CREATE INDEX idx_albums_name_lower ON albums(name_lower);
            CREATE INDEX idx_artists_name_lower ON artists(name_lower);
            "})?;

            tx.commit()?;
        }

        if existing_version <= 4 {
            println!("Migrate database version 5...");
            let tx = db.transaction()?;

            tx.pragma_update(None, "user_version", 5)?;

            tx.execute_batch(indoc! {"
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
            "})?;

            tx.commit()?;
        }

        if existing_version <= 5 {
            println!("Migrate database version 6...");
            let tx = db.transaction()?;

            tx.pragma_update(None, "user_version", 6)?;

            tx.execute_batch(indoc! {"
            ALTER TABLE config_data ADD skip_tracks_with_synced_lyrics BOOLEAN DEFAULT 0;
            ALTER TABLE config_data ADD skip_tracks_with_plain_lyrics BOOLEAN DEFAULT 0;
            UPDATE config_data SET skip_tracks_with_synced_lyrics = skip_not_needed_tracks;
            ALTER TABLE config_data DROP COLUMN skip_not_needed_tracks;
            "})?;

            tx.commit()?;
        }

        if existing_version <= 6 {
            println!("Migrate database version 7...");
            let tx = db.transaction()?;

            tx.pragma_update(None, "user_version", 7)?;

            tx.execute_batch(indoc! {"
            ALTER TABLE config_data ADD show_line_count BOOLEAN DEFAULT 1;
            "})?;

            tx.commit()?;
        }

        if existing_version <= 7 {
            println!("Migrate database version 8...");
            let tx = db.transaction()?;

            tx.pragma_update(None, "user_version", 8)?;

            tx.execute_batch(indoc! {"
            ALTER TABLE tracks ADD COLUMN file_size INTEGER;
            ALTER TABLE tracks ADD COLUMN modified_time INTEGER;
            ALTER TABLE tracks ADD COLUMN content_hash TEXT;
            ALTER TABLE tracks ADD COLUMN scan_status INTEGER DEFAULT 1;

            CREATE INDEX idx_tracks_file_path ON tracks(file_path);
            CREATE INDEX idx_tracks_content_hash ON tracks(content_hash);
            CREATE INDEX idx_tracks_scan_status ON tracks(scan_status);
            CREATE INDEX idx_tracks_fingerprint ON tracks(modified_time, file_size);
            "})?;

            tx.commit()?;
        }
    }

    Ok(())
}

pub fn get_directories(db: &Connection) -> Result<Vec<String>> {
    let mut statement = db.prepare("SELECT * FROM directories")?;
    let mut rows = statement.query([])?;
    let mut directories: Vec<String> = Vec::new();
    while let Some(row) = rows.next()? {
        let path: String = row.get("path")?;

        directories.push(path);
    }

    Ok(directories)
}

pub fn set_directories(directories: Vec<String>, db: &Connection) -> Result<()> {
    db.execute("DELETE FROM directories WHERE 1", ())?;
    let mut statement = db.prepare("INSERT INTO directories (path) VALUES (@path)")?;
    for directory in directories.iter() {
        statement.execute(named_params! { "@path": directory })?;
    }

    Ok(())
}

pub fn get_init(db: &Connection) -> Result<bool> {
    let mut statement = db.prepare("SELECT init FROM library_data LIMIT 1")?;
    let init: bool = statement.query_row([], |r| r.get(0))?;
    Ok(init)
}

pub fn set_init(init: bool, db: &Connection) -> Result<()> {
    let mut statement = db.prepare("UPDATE library_data SET init = ? WHERE 1")?;
    statement.execute([init])?;
    Ok(())
}

pub fn get_config(db: &Connection) -> Result<PersistentConfig> {
    let mut statement = db.prepare(indoc! {"
      SELECT
        skip_tracks_with_synced_lyrics,
        skip_tracks_with_plain_lyrics,
        show_line_count,
        try_embed_lyrics,
        theme_mode,
        lrclib_instance
      FROM config_data
      LIMIT 1
    "})?;
    let row = statement.query_row([], |r| {
        Ok(PersistentConfig {
            skip_tracks_with_synced_lyrics: r.get("skip_tracks_with_synced_lyrics")?,
            skip_tracks_with_plain_lyrics: r.get("skip_tracks_with_plain_lyrics")?,
            show_line_count: r.get("show_line_count")?,
            try_embed_lyrics: r.get("try_embed_lyrics")?,
            theme_mode: r.get("theme_mode")?,
            lrclib_instance: r.get("lrclib_instance")?,
        })
    })?;
    Ok(row)
}

pub fn set_config(
    skip_tracks_with_synced_lyrics: bool,
    skip_tracks_with_plain_lyrics: bool,
    show_line_count: bool,
    try_embed_lyrics: bool,
    theme_mode: &str,
    lrclib_instance: &str,
    db: &Connection,
) -> Result<()> {
    let mut statement = db.prepare(indoc! {"
      UPDATE config_data
      SET
        skip_tracks_with_synced_lyrics = ?,
        skip_tracks_with_plain_lyrics = ?,
        show_line_count = ?,
        try_embed_lyrics = ?,
        theme_mode = ?,
        lrclib_instance = ?
      WHERE 1
    "})?;
    statement.execute((
        skip_tracks_with_synced_lyrics,
        skip_tracks_with_plain_lyrics,
        show_line_count,
        try_embed_lyrics,
        theme_mode,
        lrclib_instance,
    ))?;
    Ok(())
}

pub fn find_artist(name: &str, db: &Connection) -> Result<i64> {
    let mut statement = db.prepare("SELECT id FROM artists WHERE name = ?")?;
    let id: i64 = statement.query_row([name], |r| r.get(0))?;
    Ok(id)
}

pub fn add_artist(name: &str, db: &Connection) -> Result<i64> {
    let mut statement = db.prepare("INSERT INTO artists (name, name_lower) VALUES (?, ?)")?;
    let row_id = statement.insert((name, prepare_input(name)))?;
    Ok(row_id)
}

pub fn find_album(name: &str, album_artist_name: &str, db: &Connection) -> Result<i64> {
    let mut statement =
        db.prepare("SELECT id FROM albums WHERE name = ? AND album_artist_name = ?")?;
    let id: i64 = statement.query_row((name, album_artist_name), |r| r.get(0))?;
    Ok(id)
}

pub fn add_album(name: &str, album_artist_name: &str, db: &Connection) -> Result<i64> {
    let mut statement = db.prepare("INSERT INTO albums (name, name_lower, album_artist_name, album_artist_name_lower) VALUES (?, ?, ?, ?)")?;
    let row_id = statement.insert((
        name,
        prepare_input(name),
        album_artist_name,
        prepare_input(album_artist_name),
    ))?;
    Ok(row_id)
}

pub fn get_track_by_id(id: i64, db: &Connection) -> Result<PersistentTrack> {
    let query = indoc! {"
    SELECT
      tracks.id,
      file_path,
      file_name,
      title,
      artists.name AS artist_name,
      tracks.artist_id,
      albums.name AS album_name,
      albums.album_artist_name,
      album_id,
      duration,
      track_number,
      albums.image_path,
      txt_lyrics,
      lrc_lyrics,
      instrumental
    FROM tracks
    JOIN albums ON tracks.album_id = albums.id
    JOIN artists ON tracks.artist_id = artists.id
    WHERE tracks.id = ?
    LIMIT 1
  "};

    let mut statement = db.prepare(query)?;
    let row = statement.query_row([id], |row| {
        let is_instrumental: Option<bool> = row.get("instrumental")?;

        Ok(PersistentTrack {
            id: row.get("id")?,
            file_path: row.get("file_path")?,
            file_name: row.get("file_name")?,
            title: row.get("title")?,
            artist_name: row.get("artist_name")?,
            artist_id: row.get("artist_id")?,
            album_name: row.get("album_name")?,
            album_artist_name: row.get("album_artist_name")?,
            album_id: row.get("album_id")?,
            duration: row.get("duration")?,
            track_number: row.get("track_number")?,
            txt_lyrics: row.get("txt_lyrics")?,
            lrc_lyrics: row.get("lrc_lyrics")?,
            image_path: row.get("image_path")?,
            instrumental: is_instrumental.unwrap_or(false),
        })
    })?;
    Ok(row)
}

pub fn update_track_synced_lyrics(
    id: i64,
    synced_lyrics: &str,
    plain_lyrics: &str,
    db: &Connection,
) -> Result<PersistentTrack> {
    let mut statement = db.prepare(
        "UPDATE tracks SET lrc_lyrics = ?, txt_lyrics = ?, instrumental = false WHERE id = ?",
    )?;
    statement.execute((synced_lyrics, plain_lyrics, id))?;

    Ok(get_track_by_id(id, db)?)
}

pub fn update_track_plain_lyrics(
    id: i64,
    plain_lyrics: &str,
    db: &Connection,
) -> Result<PersistentTrack> {
    let mut statement = db.prepare(
        "UPDATE tracks SET txt_lyrics = ?, lrc_lyrics = null, instrumental = false WHERE id = ?",
    )?;
    statement.execute((plain_lyrics, id))?;

    Ok(get_track_by_id(id, db)?)
}

pub fn update_track_null_lyrics(id: i64, db: &Connection) -> Result<PersistentTrack> {
    let mut statement = db.prepare(
        "UPDATE tracks SET txt_lyrics = null, lrc_lyrics = null, instrumental = false WHERE id = ?",
    )?;
    statement.execute([id])?;

    Ok(get_track_by_id(id, db)?)
}

pub fn update_track_instrumental(id: i64, db: &Connection) -> Result<PersistentTrack> {
    let mut statement = db.prepare(
        "UPDATE tracks SET txt_lyrics = null, lrc_lyrics = ?, instrumental = true WHERE id = ?",
    )?;
    statement.execute(params!["[au: instrumental]", id])?;

    Ok(get_track_by_id(id, db)?)
}

pub fn add_tracks(tracks: &Vec<fs_track::FsTrack>, db: &mut Connection) -> Result<()> {
    let tx = db.transaction()?;

    for track in tracks.iter() {
        add_track(track, &tx)?;
    }

    tx.commit()?;

    Ok(())
}

pub fn add_track(track: &fs_track::FsTrack, db: &Connection) -> Result<()> {
    let artist_result = find_artist(&track.artist(), db);
    let artist_id = match artist_result {
        Ok(artist_id) => artist_id,
        Err(_) => add_artist(&track.artist(), db)?,
    };

    let album_result = find_album(&track.album(), &track.album_artist(), db);
    let album_id = match album_result {
        Ok(album_id) => album_id,
        Err(_) => add_album(&track.album(), &track.album_artist(), db)?,
    };

    // Create a regex to match "[au: instrumental]" or "[au:instrumental]"
    let re = Regex::new(r"\[au:\s*instrumental\]").expect("Invalid regex");
    let is_instrumental = track
        .lrc_lyrics()
        .as_ref()
        .map_or(false, |lyrics| re.is_match(lyrics));

    let query = indoc! {"
    INSERT INTO tracks (
        file_path,
        file_name,
        title,
        title_lower,
        album_id,
        artist_id,
        duration,
        track_number,
        txt_lyrics,
        lrc_lyrics,
        instrumental
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  "};
    let mut statement = db.prepare(query)?;
    statement.execute((
        track.file_path(),
        track.file_name(),
        track.title(),
        prepare_input(&track.title()),
        album_id,
        artist_id,
        track.duration(),
        track.track_number(),
        track.txt_lyrics(),
        track.lrc_lyrics(),
        is_instrumental,
    ))?;

    Ok(())
}

pub fn get_tracks(db: &Connection) -> Result<Vec<PersistentTrack>> {
    let query = indoc! {"
      SELECT
          tracks.id, file_path, file_name, title,
          artists.name AS artist_name, tracks.artist_id,
          albums.name AS album_name, albums.album_artist_name, album_id, duration, track_number,
          albums.image_path, txt_lyrics, lrc_lyrics, instrumental
      FROM tracks
      JOIN albums ON tracks.album_id = albums.id
      JOIN artists ON tracks.artist_id = artists.id
      ORDER BY title_lower ASC
  "};
    let mut statement = db.prepare(query)?;
    let mut rows = statement.query([])?;
    let mut tracks: Vec<PersistentTrack> = Vec::new();

    while let Some(row) = rows.next()? {
        let is_instrumental: Option<bool> = row.get("instrumental")?;

        let track = PersistentTrack {
            id: row.get("id")?,
            file_path: row.get("file_path")?,
            file_name: row.get("file_name")?,
            title: row.get("title")?,
            artist_name: row.get("artist_name")?,
            artist_id: row.get("artist_id")?,
            album_name: row.get("album_name")?,
            album_artist_name: row.get("album_artist_name")?,
            album_id: row.get("album_id")?,
            duration: row.get("duration")?,
            track_number: row.get("track_number")?,
            txt_lyrics: row.get("txt_lyrics")?,
            lrc_lyrics: row.get("lrc_lyrics")?,
            image_path: row.get("image_path")?,
            instrumental: is_instrumental.unwrap_or(false),
        };

        tracks.push(track);
    }

    Ok(tracks)
}

pub fn get_track_ids(
    synced_lyrics: bool,
    plain_lyrics: bool,
    instrumental: bool,
    no_lyrics: bool,
    db: &Connection
) -> Result<Vec<i64>> {
    let base_query = "SELECT id FROM tracks";

    let mut conditions = Vec::new();

    if !synced_lyrics {
        conditions.push("(lrc_lyrics IS NULL OR lrc_lyrics = '[au: instrumental]')");
    }
    if !plain_lyrics {
        conditions.push("(txt_lyrics IS NULL OR lrc_lyrics IS NOT NULL)");
    }
    if !instrumental {
        conditions.push("instrumental = false");
    }
    if !no_lyrics {
        conditions.push("(txt_lyrics IS NOT NULL OR lrc_lyrics IS NOT NULL OR instrumental = true)");
    }

    let where_clause = if !conditions.is_empty() {
        format!(" WHERE {}", conditions.join(" AND "))
    } else {
        String::new()
    };

    let full_query = format!("{}{} ORDER BY title_lower ASC", base_query, where_clause);

    let mut statement = db.prepare(&full_query)?;
    let mut rows = statement.query([])?;
    let mut track_ids: Vec<i64> = Vec::new();

    while let Some(row) = rows.next()? {
        track_ids.push(row.get("id")?);
    }

    Ok(track_ids)
}

pub fn get_search_track_ids(
    query_str: &String,
    synced_lyrics: bool,
    plain_lyrics: bool,
    instrumental: bool,
    no_lyrics: bool,
    db: &Connection
) -> Result<Vec<i64>> {
    let base_query = indoc! {"
      SELECT tracks.id
      FROM tracks
      JOIN artists ON tracks.artist_id = artists.id
      JOIN albums ON tracks.album_id = albums.id
      WHERE (artists.name_lower LIKE ?
      OR albums.name_lower LIKE ?
      OR tracks.title_lower LIKE ?)
    "};

    let mut conditions = Vec::new();

    if !synced_lyrics {
        conditions.push("(lrc_lyrics IS NULL OR lrc_lyrics = '[au: instrumental]')");
    }
    if !plain_lyrics {
        conditions.push("(txt_lyrics IS NULL OR lrc_lyrics IS NOT NULL)");
    }
    if !instrumental {
        conditions.push("instrumental = false");
    }
    if !no_lyrics {
        conditions.push("(txt_lyrics IS NOT NULL OR lrc_lyrics IS NOT NULL OR instrumental = true)");
    }

    let where_clause = if !conditions.is_empty() {
        format!(" AND {}", conditions.join(" AND "))
    } else {
        String::new()
    };

    let full_query = format!("{}{} ORDER BY title_lower ASC", base_query, where_clause);

    let mut statement = db.prepare(&full_query)?;
    let formatted_query_str = format!("%{}%", prepare_input(query_str));
    let mut rows = statement.query(params![
        formatted_query_str,
        formatted_query_str,
        formatted_query_str
    ])?;
    let mut track_ids: Vec<i64> = Vec::new();

    while let Some(row) = rows.next()? {
        track_ids.push(row.get("id")?);
    }

    Ok(track_ids)
}

pub fn get_albums(db: &Connection) -> Result<Vec<PersistentAlbum>> {
    let mut statement = db.prepare(indoc! {"
      SELECT albums.id, albums.name, albums.album_artist_name AS album_artist_name, albums.album_artist_name,
          COUNT(tracks.id) AS tracks_count
      FROM albums
      JOIN tracks ON tracks.album_id = albums.id
      GROUP BY albums.id, albums.name, albums.album_artist_name
      ORDER BY albums.name_lower ASC
  "})?;
    let mut rows = statement.query([])?;
    let mut albums: Vec<PersistentAlbum> = Vec::new();

    while let Some(row) = rows.next()? {
        let album = PersistentAlbum {
            id: row.get("id")?,
            name: row.get("name")?,
            image_path: row.get("image_path")?,
            artist_name: row.get("album_artist_name")?,
            album_artist_name: row.get("album_artist_name")?,
            tracks_count: row.get("tracks_count")?,
        };

        albums.push(album);
    }

    Ok(albums)
}

pub fn get_album_by_id(id: i64, db: &Connection) -> Result<PersistentAlbum> {
    let mut statement = db.prepare(indoc! {"
    SELECT
      albums.id,
      albums.name,
      albums.album_artist_name,
      COUNT(tracks.id) AS tracks_count
    FROM albums
    JOIN tracks ON tracks.album_id = albums.id
    WHERE albums.id = ?
    GROUP BY
      albums.id,
      albums.name,
      albums.album_artist_name
    LIMIT 1
  "})?;
    let row = statement.query_row([id], |row| {
        Ok(PersistentAlbum {
            id: row.get("id")?,
            name: row.get("name")?,
            image_path: None,
            artist_name: row.get("album_artist_name")?,
            album_artist_name: row.get("album_artist_name")?,
            tracks_count: row.get("tracks_count")?,
        })
    })?;
    Ok(row)
}

pub fn get_album_ids(db: &Connection) -> Result<Vec<i64>> {
    let mut statement = db.prepare("SELECT id FROM albums ORDER BY name_lower ASC")?;
    let mut rows = statement.query([])?;
    let mut album_ids: Vec<i64> = Vec::new();

    while let Some(row) = rows.next()? {
        album_ids.push(row.get("id")?);
    }

    Ok(album_ids)
}

pub fn get_artists(db: &Connection) -> Result<Vec<PersistentArtist>> {
    let mut statement = db.prepare(indoc! {"
    SELECT artists.id, artists.name AS name, COUNT(tracks.id) AS tracks_count
    FROM artists
    JOIN tracks ON tracks.artist_id = artists.id
    GROUP BY artists.id, artists.name
    ORDER BY artists.name_lower ASC
  "})?;
    let mut rows = statement.query([])?;
    let mut artists: Vec<PersistentArtist> = Vec::new();

    while let Some(row) = rows.next()? {
        let artist = PersistentArtist {
            id: row.get("id")?,
            name: row.get("name")?,
            // albums_count: row.get("albums_count")?,
            tracks_count: row.get("tracks_count")?,
        };

        artists.push(artist);
    }

    Ok(artists)
}

pub fn get_artist_by_id(id: i64, db: &Connection) -> Result<PersistentArtist> {
    let mut statement = db.prepare(indoc! {"
    SELECT artists.id,
      artists.name AS name,
      COUNT(tracks.id) AS tracks_count
    FROM artists
    JOIN tracks ON tracks.artist_id = artists.id
    WHERE artists.id = ?
    GROUP BY artists.id, artists.name
    LIMIT 1
  "})?;
    let row = statement.query_row([id], |row| {
        Ok(PersistentArtist {
            id: row.get("id")?,
            name: row.get("name")?,
            // albums_count: row.get("albums_count")?,
            tracks_count: row.get("tracks_count")?,
        })
    })?;
    Ok(row)
}

pub fn get_artist_ids(db: &Connection) -> Result<Vec<i64>> {
    let mut statement = db.prepare("SELECT id FROM artists ORDER BY name_lower ASC")?;
    let mut rows = statement.query([])?;
    let mut artist_ids: Vec<i64> = Vec::new();

    while let Some(row) = rows.next()? {
        artist_ids.push(row.get("id")?);
    }

    Ok(artist_ids)
}

pub fn get_album_tracks(album_id: i64, db: &Connection) -> Result<Vec<PersistentTrack>> {
    let mut statement = db.prepare(indoc! {"
    SELECT
      tracks.id,
      file_path,
      file_name,
      title,
      artists.name AS artist_name,
      tracks.artist_id,
      albums.name AS album_name,
      albums.album_artist_name,
      album_id,
      duration,
      track_number,
      albums.image_path,
      txt_lyrics,
      lrc_lyrics,
      instrumental
    FROM tracks
    JOIN albums ON tracks.album_id = albums.id
    JOIN artists ON tracks.artist_id = artists.id
    WHERE tracks.album_id = ?
    ORDER BY track_number ASC
  "})?;
    let mut rows = statement.query([album_id])?;
    let mut tracks: Vec<PersistentTrack> = Vec::new();

    while let Some(row) = rows.next()? {
        let is_instrumental: Option<bool> = row.get("instrumental")?;

        let track = PersistentTrack {
            id: row.get("id")?,
            file_path: row.get("file_path")?,
            file_name: row.get("file_name")?,
            title: row.get("title")?,
            artist_name: row.get("artist_name")?,
            album_artist_name: row.get("album_artist_name")?,
            album_name: row.get("album_name")?,
            album_id: row.get("album_id")?,
            artist_id: row.get("artist_id")?,
            duration: row.get("duration")?,
            track_number: row.get("track_number")?,
            txt_lyrics: row.get("txt_lyrics")?,
            lrc_lyrics: row.get("lrc_lyrics")?,
            image_path: row.get("image_path")?,
            instrumental: is_instrumental.unwrap_or(false),
        };

        tracks.push(track);
    }

    Ok(tracks)
}

pub fn get_album_track_ids(album_id: i64, without_plain_lyrics: bool, without_synced_lyrics: bool, db: &Connection) -> Result<Vec<i64>> {
    let base_query = indoc! {"
      SELECT tracks.id
      FROM tracks
      JOIN albums ON tracks.album_id = albums.id
      WHERE tracks.album_id = ?"};

    let lyrics_conditions = match (without_plain_lyrics, without_synced_lyrics) {
        (true, true) => " AND txt_lyrics IS NULL AND lrc_lyrics IS NULL AND tracks.instrumental = false",
        (true, false) => " AND txt_lyrics IS NULL AND tracks.instrumental = false",
        (false, true) => " AND lrc_lyrics IS NULL AND tracks.instrumental = false",
        (false, false) => "",
    };

    let full_query = format!("{}{} ORDER BY tracks.track_number ASC",
        base_query, lyrics_conditions);

    let mut statement = db.prepare(&full_query)?;
    let mut rows = statement.query([album_id])?;
    let mut tracks: Vec<i64> = Vec::new();

    while let Some(row) = rows.next()? {
        tracks.push(row.get("id")?);
    }

    Ok(tracks)
}

pub fn get_artist_tracks(artist_id: i64, db: &Connection) -> Result<Vec<PersistentTrack>> {
    let mut statement = db.prepare(indoc! {"
      SELECT tracks.id, file_path, file_name, title, artists.name AS artist_name,
        tracks.artist_id, albums.name AS album_name, albums.album_artist_name, album_id, duration, track_number,
        albums.image_path, txt_lyrics, lrc_lyrics, instrumental
      FROM tracks
      JOIN albums ON tracks.album_id = albums.id
      JOIN artists ON tracks.artist_id = artists.id
      WHERE tracks.artist_id = ?
      ORDER BY album_name_lower ASC, track_number ASC
  "})?;
    let mut rows = statement.query([artist_id])?;
    let mut tracks: Vec<PersistentTrack> = Vec::new();

    while let Some(row) = rows.next()? {
        let is_instrumental: Option<bool> = row.get("instrumental")?;

        let track = PersistentTrack {
            id: row.get("id")?,
            file_path: row.get("file_path")?,
            file_name: row.get("file_name")?,
            title: row.get("title")?,
            artist_name: row.get("artist_name")?,
            artist_id: row.get("artist_id")?,
            album_name: row.get("album_name")?,
            album_artist_name: row.get("album_artist_name")?,
            album_id: row.get("album_id")?,
            duration: row.get("duration")?,
            track_number: row.get("track_number")?,
            txt_lyrics: row.get("txt_lyrics")?,
            lrc_lyrics: row.get("lrc_lyrics")?,
            image_path: row.get("image_path")?,
            instrumental: is_instrumental.unwrap_or(false),
        };

        tracks.push(track);
    }

    Ok(tracks)
}

pub fn get_artist_track_ids(artist_id: i64, without_plain_lyrics: bool, without_synced_lyrics: bool, db: &Connection) -> Result<Vec<i64>> {
    let base_query = indoc! {"
      SELECT tracks.id
      FROM tracks
      JOIN albums ON tracks.album_id = albums.id
      JOIN artists ON tracks.artist_id = artists.id
      WHERE tracks.artist_id = ?"};

    let lyrics_conditions = match (without_plain_lyrics, without_synced_lyrics) {
        (true, true) => " AND txt_lyrics IS NULL AND lrc_lyrics IS NULL AND tracks.instrumental = false",
        (true, false) => " AND txt_lyrics IS NULL AND tracks.instrumental = false",
        (false, true) => " AND lrc_lyrics IS NULL AND tracks.instrumental = false",
        (false, false) => "",
    };

    let full_query = format!("{}{} ORDER BY albums.name_lower ASC, tracks.track_number ASC",
        base_query, lyrics_conditions);

    let mut statement = db.prepare(&full_query)?;
    let mut rows = statement.query([artist_id])?;
    let mut tracks: Vec<i64> = Vec::new();

    while let Some(row) = rows.next()? {
        tracks.push(row.get("id")?);
    }

    Ok(tracks)
}

pub fn clean_library(db: &Connection) -> Result<()> {
    db.execute("DELETE FROM tracks WHERE 1", ())?;
    db.execute("DELETE FROM albums WHERE 1", ())?;
    db.execute("DELETE FROM artists WHERE 1", ())?;
    Ok(())
}

/// Get all tracks with their fingerprint data for comparison
pub fn get_tracks_with_fingerprints(db: &Connection) -> Result<Vec<DbTrack>> {
    let mut statement =
        db.prepare("SELECT id, file_path, file_size, modified_time, content_hash FROM tracks")?;
    let mut rows = statement.query([])?;
    let mut tracks = Vec::new();

    while let Some(row) = rows.next()? {
        tracks.push(DbTrack {
            id: row.get("id")?,
            file_path: row.get("file_path")?,
            file_size: row.get("file_size")?,
            modified_time: row.get("modified_time")?,
            content_hash: row.get("content_hash")?,
        });
    }

    Ok(tracks)
}

/// Delete tracks by their IDs (batch operation)
pub fn delete_tracks_by_ids(ids: &[i64], conn: &Connection) -> Result<()> {
    if ids.is_empty() {
        return Ok(());
    }

    // Create placeholders for the IN clause
    let placeholders: Vec<String> = (0..ids.len()).map(|i| format!("?{}", i + 1)).collect();
    let query = format!(
        "DELETE FROM tracks WHERE id IN ({})",
        placeholders.join(", ")
    );

    let mut statement = conn.prepare(&query)?;
    let params: Vec<rusqlite::types::Value> = ids.iter().map(|&id| id.into()).collect();
    statement.execute(rusqlite::params_from_iter(params.iter()))?;

    Ok(())
}

/// Update a track's file path (for move/rename detection)
pub fn update_track_path(
    track_id: i64,
    new_path: &std::path::Path,
    conn: &Connection,
) -> Result<()> {
    let new_path_str = new_path.to_string_lossy().to_string();
    let file_name = new_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    conn.execute(
        "UPDATE tracks SET file_path = ?, file_name = ? WHERE id = ?",
        (new_path_str, file_name, track_id),
    )?;

    Ok(())
}

// ============================================================================
// Scan-related database operations (transaction-based)
// ============================================================================

const SCAN_STATUS_PENDING: i32 = 0;
const SCAN_STATUS_PROCESSED: i32 = 1;

/// Track info for scan operations
#[derive(Debug)]
pub struct ScanTrackInfo {
    pub id: i64,
    pub file_path: String,
}

/// Find track by fingerprint (mtime + size) - for scan operations
pub fn find_track_by_fingerprint_tx(
    modified_time: i64,
    file_size: i64,
    tx: &rusqlite::Transaction,
) -> Result<Option<ScanTrackInfo>> {
    let mut stmt = tx.prepare(
        "SELECT id, file_path FROM tracks WHERE modified_time = ? AND file_size = ? LIMIT 1",
    )?;

    let result = stmt
        .query_row([modified_time, file_size], |row| {
            Ok(ScanTrackInfo {
                id: row.get(0)?,
                file_path: row.get(1)?,
            })
        })
        .optional()?;

    Ok(result)
}

/// Find track by content hash - for scan operations
pub fn find_track_by_hash_tx(
    hash: &str,
    tx: &rusqlite::Transaction,
) -> Result<Option<ScanTrackInfo>> {
    let mut stmt = tx.prepare("SELECT id, file_path FROM tracks WHERE content_hash = ? LIMIT 1")?;

    let result = stmt
        .query_row([hash], |row| {
            Ok(ScanTrackInfo {
                id: row.get(0)?,
                file_path: row.get(1)?,
            })
        })
        .optional()?;

    Ok(result)
}

/// Mark track as processed during scan
pub fn mark_track_processed_tx(track_id: i64, tx: &rusqlite::Transaction) -> Result<()> {
    tx.execute(
        "UPDATE tracks SET scan_status = ? WHERE id = ?",
        [SCAN_STATUS_PROCESSED, track_id as i32],
    )?;
    Ok(())
}

/// Update track path after move (fingerprint already matches)
pub fn update_track_path_tx(
    track_id: i64,
    new_path: &str,
    tx: &rusqlite::Transaction,
) -> Result<()> {
    let file_name = std::path::Path::new(new_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    tx.execute(
        "UPDATE tracks SET file_path = ?, file_name = ?, scan_status = ? WHERE id = ?",
        (new_path, file_name, SCAN_STATUS_PROCESSED, track_id),
    )?;

    Ok(())
}

/// Update track path and fingerprint after move
pub fn update_track_path_and_fingerprint_tx(
    track_id: i64,
    new_path: &str,
    file_size: i64,
    modified_time: i64,
    content_hash: &str,
    tx: &rusqlite::Transaction,
) -> Result<()> {
    let file_name = std::path::Path::new(new_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    tx.execute(
        "UPDATE tracks SET file_path = ?, file_name = ?, file_size = ?, modified_time = ?, content_hash = ?, scan_status = ? WHERE id = ?",
        (new_path, file_name, file_size, modified_time, content_hash, SCAN_STATUS_PROCESSED, track_id),
    )?;

    Ok(())
}

/// Insert a new track from metadata during scan
pub fn insert_track_from_metadata_tx(
    metadata: &crate::scanner::metadata::TrackMetadata,
    lyrics: &crate::scanner::metadata::LyricsInfo,
    file_size: i64,
    modified_time: i64,
    content_hash: &str,
    artist_id: i64,
    album_id: i64,
    tx: &rusqlite::Transaction,
) -> Result<()> {
    use crate::utils::prepare_input;

    // Check if instrumental
    let instrumental = lyrics
        .lrc_lyrics
        .as_ref()
        .map_or(false, |l| l.contains("[au:") && l.contains("instrumental"));

    tx.execute(
        "INSERT INTO tracks (file_path, file_name, title, title_lower, album_id, artist_id, duration, track_number, file_size, modified_time, content_hash, scan_status, txt_lyrics, lrc_lyrics, instrumental) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (
            &metadata.file_path,
            &metadata.file_name,
            &metadata.title,
            prepare_input(&metadata.title),
            album_id,
            artist_id,
            metadata.duration,
            metadata.track_number,
            file_size,
            modified_time,
            content_hash,
            SCAN_STATUS_PROCESSED,
            lyrics.txt_lyrics.as_ref(),
            lyrics.lrc_lyrics.as_ref(),
            instrumental,
        ),
    )?;

    Ok(())
}

/// Delete tracks that weren't processed during scan and clean up orphaned albums/artists
pub fn delete_unprocessed_tracks(conn: &mut Connection) -> Result<usize> {
    let tx = conn.transaction()?;

    // Delete tracks that weren't seen during this scan
    let deleted_count = tx.execute(
        "DELETE FROM tracks WHERE scan_status = ?",
        [SCAN_STATUS_PENDING],
    )?;

    // Clean up orphaned albums and artists
    tx.execute(
        "DELETE FROM albums WHERE id NOT IN (SELECT DISTINCT album_id FROM tracks)",
        [],
    )?;

    tx.execute(
        "DELETE FROM artists WHERE id NOT IN (SELECT DISTINCT artist_id FROM tracks)",
        [],
    )?;

    tx.commit()?;

    Ok(deleted_count)
}

/// Mark all tracks as pending before scan
pub fn mark_all_tracks_pending(conn: &mut Connection) -> Result<()> {
    conn.execute("UPDATE tracks SET scan_status = ?", [SCAN_STATUS_PENDING])?;
    Ok(())
}

// Transaction-based versions of artist/album functions for scan operations

/// Find artist by name (transaction version)
pub fn find_artist_tx(name: &str, tx: &rusqlite::Transaction) -> Result<i64> {
    let mut statement = tx.prepare("SELECT id FROM artists WHERE name = ?")?;
    let id: i64 = statement.query_row([name], |r| r.get(0))?;
    Ok(id)
}

/// Add new artist (transaction version)
pub fn add_artist_tx(name: &str, tx: &rusqlite::Transaction) -> Result<i64> {
    let mut statement = tx.prepare("INSERT INTO artists (name, name_lower) VALUES (?, ?)")?;
    let row_id = statement.insert((name, prepare_input(name)))?;
    Ok(row_id)
}

/// Find album by name and artist (transaction version)
pub fn find_album_tx(
    name: &str,
    album_artist_name: &str,
    tx: &rusqlite::Transaction,
) -> Result<i64> {
    let mut statement =
        tx.prepare("SELECT id FROM albums WHERE name = ? AND album_artist_name = ?")?;
    let id: i64 = statement.query_row((name, album_artist_name), |r| r.get(0))?;
    Ok(id)
}

/// Add new album (transaction version)
pub fn add_album_tx(
    name: &str,
    album_artist_name: &str,
    tx: &rusqlite::Transaction,
) -> Result<i64> {
    let mut statement = tx.prepare("INSERT INTO albums (name, name_lower, album_artist_name, album_artist_name_lower) VALUES (?, ?, ?, ?)")?;
    let row_id = statement.insert((
        name,
        prepare_input(name),
        album_artist_name,
        prepare_input(album_artist_name),
    ))?;
    Ok(row_id)
}
