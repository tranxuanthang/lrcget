use crate::lyricsfile::{lyrics_presence_from_lyricsfile, LyricsPresence};
use crate::persistent_entities::{
    PersistentAlbum, PersistentArtist, PersistentConfig, PersistentTrack,
};
use crate::scanner::models::DbTrack;
use crate::utils::prepare_input;
use anyhow::Result;
use include_dir::{include_dir, Dir};
use indoc::indoc;
use rusqlite::{named_params, params, Connection, OptionalExtension};
use rusqlite_migration::Migrations;
use std::fs;
use tauri::{AppHandle, Manager};

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

/// Initializes the database connection, creating the .sqlite file if needed, and upgrading the
/// database if it's out of date.
pub fn initialize_database(app_handle: &AppHandle) -> Result<Connection, rusqlite::Error> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("The app data directory should exist.");
    fs::create_dir_all(&app_dir).expect("The app data directory should be created.");
    let sqlite_path = app_dir.join("db.sqlite3");

    println!("Database file path: {}", sqlite_path.display());

    let mut db = Connection::open(sqlite_path)?;

    db.pragma_update(None, "journal_mode", "WAL")?;

    let migrations = Migrations::from_directory(&MIGRATIONS_DIR)
        .expect("Failed to load migrations from directory");
    migrations
        .to_latest(&mut db)
        .expect("Failed to run database migrations");

    if let Err(error) = backfill_track_lyrics_presence(&db) {
        eprintln!("Failed to backfill track lyrics presence flags: {}", error);
    }

    Ok(db)
}

// Backfill lyrics presence data from lyricsfile content
// This updates lyricsfiles records that may not have presence fields set
pub fn backfill_track_lyrics_presence(db: &Connection) -> Result<()> {
    let mut select_statement = db.prepare(indoc! {"
      SELECT
        lyricsfiles.id,
        lyricsfiles.lyricsfile
      FROM lyricsfiles
      WHERE lyricsfiles.has_plain_lyrics = 0
         AND lyricsfiles.has_synced_lyrics = 0
         AND lyricsfiles.has_word_synced_lyrics = 0
    "})?;

    let mut rows = select_statement.query([])?;
    let mut updates: Vec<(i64, LyricsPresence)> = Vec::new();

    while let Some(row) = rows.next()? {
        let lyricsfile_id: i64 = row.get("id")?;
        let lyricsfile_content: Option<String> = row.get("lyricsfile")?;

        let presence = match lyricsfile_content
            .as_deref()
            .map(str::trim)
            .filter(|content| !content.is_empty())
        {
            Some(content) => lyrics_presence_from_lyricsfile(content).unwrap_or_default(),
            None => LyricsPresence::default(),
        };

        updates.push((lyricsfile_id, presence));
    }

    // Update lyricsfiles records with calculated presence
    for (lyricsfile_id, presence) in updates {
        db.execute(
            "UPDATE lyricsfiles SET has_plain_lyrics = ?, has_synced_lyrics = ?, has_word_synced_lyrics = ?, instrumental = ? WHERE id = ?",
            (
                presence.has_plain_lyrics,
                presence.has_synced_lyrics,
                presence.has_word_synced_lyrics,
                presence.is_instrumental,
                lyricsfile_id,
            ),
        )?;
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
        lrclib_instance,
        volume
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
            volume: r.get("volume")?,
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
    volume: f64,
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
        lrclib_instance = ?,
        volume = ?
      WHERE 1
    "})?;
    statement.execute((
        skip_tracks_with_synced_lyrics,
        skip_tracks_with_plain_lyrics,
        show_line_count,
        try_embed_lyrics,
        theme_mode,
        lrclib_instance,
        volume,
    ))?;
    Ok(())
}

pub fn set_volume_config(volume: f64, db: &Connection) -> Result<()> {
    let mut statement = db.prepare("UPDATE config_data SET volume = ? WHERE 1")?;
    statement.execute([volume])?;
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
      tracks.file_path,
      tracks.file_name,
      tracks.title,
      artists.name AS artist_name,
      tracks.artist_id,
      albums.name AS album_name,
      albums.album_artist_name,
      tracks.album_id,
      tracks.duration,
      tracks.track_number,
      albums.image_path,
      lyricsfiles.id AS lyricsfile_id,
      lyricsfiles.lyricsfile,
      COALESCE(lyricsfiles.instrumental, 0) AS instrumental
    FROM tracks
    JOIN albums ON tracks.album_id = albums.id
    JOIN artists ON tracks.artist_id = artists.id
    LEFT JOIN lyricsfiles ON lyricsfiles.track_id = tracks.id
    WHERE tracks.id = ?
    LIMIT 1
  "};

    let mut statement = db.prepare(query)?;
    let row = statement.query_row([id], |row| {
        let is_instrumental: bool = row.get("instrumental")?;

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
            txt_lyrics: None,
            lrc_lyrics: None,
            lyricsfile: row.get("lyricsfile")?,
            lyricsfile_id: row.get("lyricsfile_id")?,
            image_path: row.get("image_path")?,
            instrumental: is_instrumental,
        })
    })?;
    Ok(row)
}

pub fn upsert_lyricsfile_for_track(
    track_id: i64,
    track_title: &str,
    track_album_name: &str,
    track_artist_name: &str,
    track_duration: f64,
    lyricsfile: &str,
    db: &Connection,
) -> Result<()> {
    let presence = lyrics_presence_from_lyricsfile(lyricsfile)?;

    db.execute(
        indoc! {"
        INSERT INTO lyricsfiles (
            track_id,
            track_title,
            track_title_lower,
            track_album_name,
            track_album_name_lower,
            track_artist_name,
            track_artist_name_lower,
            track_duration,
            lyricsfile,
            has_plain_lyrics,
            has_synced_lyrics,
            has_word_synced_lyrics,
            instrumental,
            created_at,
            updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT(track_id) DO UPDATE SET
            track_title = excluded.track_title,
            track_title_lower = excluded.track_title_lower,
            track_album_name = excluded.track_album_name,
            track_album_name_lower = excluded.track_album_name_lower,
            track_artist_name = excluded.track_artist_name,
            track_artist_name_lower = excluded.track_artist_name_lower,
            track_duration = excluded.track_duration,
            lyricsfile = excluded.lyricsfile,
            has_plain_lyrics = excluded.has_plain_lyrics,
            has_synced_lyrics = excluded.has_synced_lyrics,
            has_word_synced_lyrics = excluded.has_word_synced_lyrics,
            instrumental = excluded.instrumental,
            updated_at = CURRENT_TIMESTAMP
    "},
        (
            track_id,
            track_title,
            prepare_input(track_title),
            track_album_name,
            prepare_input(track_album_name),
            track_artist_name,
            prepare_input(track_artist_name),
            track_duration,
            lyricsfile,
            presence.has_plain_lyrics,
            presence.has_synced_lyrics,
            presence.has_word_synced_lyrics,
            presence.is_instrumental,
        ),
    )?;

    Ok(())
}

pub fn upsert_lyricsfile_for_track_tx(
    track_id: i64,
    track_title: &str,
    track_album_name: &str,
    track_artist_name: &str,
    track_duration: f64,
    lyricsfile: &str,
    tx: &rusqlite::Transaction,
) -> Result<()> {
    let presence = lyrics_presence_from_lyricsfile(lyricsfile)?;

    tx.execute(
        indoc! {"
        INSERT INTO lyricsfiles (
            track_id,
            track_title,
            track_title_lower,
            track_album_name,
            track_album_name_lower,
            track_artist_name,
            track_artist_name_lower,
            track_duration,
            lyricsfile,
            has_plain_lyrics,
            has_synced_lyrics,
            has_word_synced_lyrics,
            instrumental,
            created_at,
            updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT(track_id) DO UPDATE SET
            track_title = excluded.track_title,
            track_title_lower = excluded.track_title_lower,
            track_album_name = excluded.track_album_name,
            track_album_name_lower = excluded.track_album_name_lower,
            track_artist_name = excluded.track_artist_name,
            track_artist_name_lower = excluded.track_artist_name_lower,
            track_duration = excluded.track_duration,
            lyricsfile = excluded.lyricsfile,
            has_plain_lyrics = excluded.has_plain_lyrics,
            has_synced_lyrics = excluded.has_synced_lyrics,
            has_word_synced_lyrics = excluded.has_word_synced_lyrics,
            instrumental = excluded.instrumental,
            updated_at = CURRENT_TIMESTAMP
    "},
        (
            track_id,
            track_title,
            prepare_input(track_title),
            track_album_name,
            prepare_input(track_album_name),
            track_artist_name,
            prepare_input(track_artist_name),
            track_duration,
            lyricsfile,
            presence.has_plain_lyrics,
            presence.has_synced_lyrics,
            presence.has_word_synced_lyrics,
            presence.is_instrumental,
        ),
    )?;

    Ok(())
}

pub fn delete_lyricsfile_by_track_id(track_id: i64, db: &Connection) -> Result<()> {
    db.execute("DELETE FROM lyricsfiles WHERE track_id = ?", [track_id])?;
    Ok(())
}

/// Get lyricsfile by LRCLIB instance and ID
/// Returns (lyricsfile_id, lyricsfile_content) if found
pub fn get_lyricsfile_by_lrclib(
    lrclib_instance: &str,
    lrclib_id: i64,
    db: &Connection,
) -> Result<Option<(i64, String)>> {
    let result = db
        .query_row(
            "SELECT id, lyricsfile FROM lyricsfiles WHERE lrclib_instance = ? AND lrclib_id = ?",
            [lrclib_instance, &lrclib_id.to_string()],
            |row| {
                let id: i64 = row.get(0)?;
                let lyricsfile: String = row.get(1)?;
                Ok((id, lyricsfile))
            },
        )
        .optional()?;
    Ok(result)
}

/// Upsert lyricsfile for LRCLIB track (standalone, no track association)
/// Returns the lyricsfile_id
pub fn upsert_lyricsfile_for_lrclib(
    lrclib_instance: &str,
    lrclib_id: i64,
    track_title: &str,
    track_album_name: &str,
    track_artist_name: &str,
    track_duration: f64,
    lyricsfile: &str,
    db: &Connection,
) -> Result<i64> {
    // Calculate presence fields from lyricsfile content
    let presence = lyrics_presence_from_lyricsfile(lyricsfile)?;

    db.execute(
        indoc! {"
        INSERT INTO lyricsfiles (
            lrclib_instance,
            lrclib_id,
            track_title,
            track_title_lower,
            track_album_name,
            track_album_name_lower,
            track_artist_name,
            track_artist_name_lower,
            track_duration,
            lyricsfile,
            has_plain_lyrics,
            has_synced_lyrics,
            has_word_synced_lyrics,
            instrumental,
            created_at,
            updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT(lrclib_instance, lrclib_id) DO UPDATE SET
            track_title = excluded.track_title,
            track_title_lower = excluded.track_title_lower,
            track_album_name = excluded.track_album_name,
            track_album_name_lower = excluded.track_album_name_lower,
            track_artist_name = excluded.track_artist_name,
            track_artist_name_lower = excluded.track_artist_name_lower,
            track_duration = excluded.track_duration,
            lyricsfile = excluded.lyricsfile,
            has_plain_lyrics = excluded.has_plain_lyrics,
            has_synced_lyrics = excluded.has_synced_lyrics,
            has_word_synced_lyrics = excluded.has_word_synced_lyrics,
            instrumental = excluded.instrumental,
            updated_at = CURRENT_TIMESTAMP
    "},
        (
            lrclib_instance,
            lrclib_id,
            track_title,
            prepare_input(track_title),
            track_album_name,
            prepare_input(track_album_name),
            track_artist_name,
            prepare_input(track_artist_name),
            track_duration,
            lyricsfile,
            presence.has_plain_lyrics,
            presence.has_synced_lyrics,
            presence.has_word_synced_lyrics,
            presence.is_instrumental,
        ),
    )?;

    // Get the inserted/updated row ID
    let lyricsfile_id = db.query_row(
        "SELECT id FROM lyricsfiles WHERE lrclib_instance = ? AND lrclib_id = ?",
        [lrclib_instance, &lrclib_id.to_string()],
        |row| row.get(0),
    )?;

    Ok(lyricsfile_id)
}

/// Update lyricsfile content by ID (for standalone lyricsfiles without track association)
pub fn update_lyricsfile_by_id(
    lyricsfile_id: i64,
    lyricsfile: &str,
    db: &Connection,
) -> Result<()> {
    // Calculate presence fields from lyricsfile content
    let presence = lyrics_presence_from_lyricsfile(lyricsfile)?;

    db.execute(
        "UPDATE lyricsfiles SET lyricsfile = ?, has_plain_lyrics = ?, has_synced_lyrics = ?, has_word_synced_lyrics = ?, instrumental = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        (
            lyricsfile,
            presence.has_plain_lyrics,
            presence.has_synced_lyrics,
            presence.has_word_synced_lyrics,
            presence.is_instrumental,
            lyricsfile_id,
        ),
    )?;
    Ok(())
}

/// Get lyricsfile by ID
/// Returns (lyricsfile_id, track_id, lyricsfile_content) if found
pub fn get_lyricsfile_by_id(
    lyricsfile_id: i64,
    db: &Connection,
) -> Result<Option<(i64, Option<i64>, String)>> {
    let result = db
        .query_row(
            "SELECT id, track_id, lyricsfile FROM lyricsfiles WHERE id = ?",
            [lyricsfile_id],
            |row| {
                let id: i64 = row.get(0)?;
                let track_id: Option<i64> = row.get(1)?;
                let lyricsfile: String = row.get(2)?;
                Ok((id, track_id, lyricsfile))
            },
        )
        .optional()?;
    Ok(result)
}

pub fn get_tracks(db: &Connection) -> Result<Vec<PersistentTrack>> {
    let query = indoc! {"
      SELECT
          tracks.id, tracks.file_path, tracks.file_name, tracks.title,
          artists.name AS artist_name, tracks.artist_id,
          albums.name AS album_name, albums.album_artist_name, tracks.album_id, tracks.duration, tracks.track_number,
          albums.image_path, lyricsfiles.id AS lyricsfile_id, lyricsfiles.lyricsfile, COALESCE(lyricsfiles.instrumental, 0) AS instrumental
      FROM tracks
      JOIN albums ON tracks.album_id = albums.id
      JOIN artists ON tracks.artist_id = artists.id
      LEFT JOIN lyricsfiles ON lyricsfiles.track_id = tracks.id
      ORDER BY tracks.title_lower ASC
  "};
    let mut statement = db.prepare(query)?;
    let mut rows = statement.query([])?;
    let mut tracks: Vec<PersistentTrack> = Vec::new();

    while let Some(row) = rows.next()? {
        let is_instrumental: bool = row.get("instrumental")?;

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
            txt_lyrics: None,
            lrc_lyrics: None,
            lyricsfile: row.get("lyricsfile")?,
            lyricsfile_id: row.get("lyricsfile_id")?,
            image_path: row.get("image_path")?,
            instrumental: is_instrumental,
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
    db: &Connection,
) -> Result<Vec<i64>> {
    // Join with lyricsfiles table and use COALESCE to handle tracks without lyricsfiles
    let base_query = indoc! {"
        SELECT tracks.id 
        FROM tracks 
        LEFT JOIN lyricsfiles ON lyricsfiles.track_id = tracks.id
    "};

    let mut included_categories: Vec<String> = Vec::new();
    if synced_lyrics {
        included_categories.push(
            "(COALESCE(lyricsfiles.has_synced_lyrics, 0) = 1 AND COALESCE(lyricsfiles.instrumental, 0) = 0)".to_string(),
        );
    }
    if plain_lyrics {
        included_categories.push(
            "(COALESCE(lyricsfiles.has_plain_lyrics, 0) = 1 AND COALESCE(lyricsfiles.has_synced_lyrics, 0) = 0 AND COALESCE(lyricsfiles.instrumental, 0) = 0)".to_string(),
        );
    }
    if instrumental {
        included_categories.push("COALESCE(lyricsfiles.instrumental, 0) = 1".to_string());
    }
    if no_lyrics {
        included_categories.push(
            "(COALESCE(lyricsfiles.has_plain_lyrics, 0) = 0 AND COALESCE(lyricsfiles.has_synced_lyrics, 0) = 0 AND COALESCE(lyricsfiles.instrumental, 0) = 0)".to_string(),
        );
    }

    let where_clause = if included_categories.len() == 4 {
        String::new()
    } else if included_categories.is_empty() {
        " WHERE 0".to_string()
    } else {
        format!(" WHERE ({})", included_categories.join(" OR "))
    };

    let full_query = format!(
        "{}{} ORDER BY tracks.title_lower ASC",
        base_query, where_clause
    );

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
    db: &Connection,
) -> Result<Vec<i64>> {
    let base_query = indoc! {"
      SELECT tracks.id
      FROM tracks
      JOIN artists ON tracks.artist_id = artists.id
      JOIN albums ON tracks.album_id = albums.id
      LEFT JOIN lyricsfiles ON lyricsfiles.track_id = tracks.id
      WHERE (artists.name_lower LIKE ?
      OR albums.name_lower LIKE ?
      OR tracks.title_lower LIKE ?)
    "};

    let mut included_categories: Vec<String> = Vec::new();
    if synced_lyrics {
        included_categories.push(
            "(COALESCE(lyricsfiles.has_synced_lyrics, 0) = 1 AND COALESCE(lyricsfiles.instrumental, 0) = 0)".to_string(),
        );
    }
    if plain_lyrics {
        included_categories.push(
            "(COALESCE(lyricsfiles.has_plain_lyrics, 0) = 1 AND COALESCE(lyricsfiles.has_synced_lyrics, 0) = 0 AND COALESCE(lyricsfiles.instrumental, 0) = 0)".to_string(),
        );
    }
    if instrumental {
        included_categories.push("COALESCE(lyricsfiles.instrumental, 0) = 1".to_string());
    }
    if no_lyrics {
        included_categories.push(
            "(COALESCE(lyricsfiles.has_plain_lyrics, 0) = 0 AND COALESCE(lyricsfiles.has_synced_lyrics, 0) = 0 AND COALESCE(lyricsfiles.instrumental, 0) = 0)".to_string(),
        );
    }

    let where_clause = if included_categories.len() == 4 {
        String::new()
    } else if included_categories.is_empty() {
        " AND 0".to_string()
    } else {
        format!(" AND ({})", included_categories.join(" OR "))
    };

    let full_query = format!(
        "{}{} ORDER BY tracks.title_lower ASC",
        base_query, where_clause
    );

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
      tracks.file_path,
      tracks.file_name,
      tracks.title,
      artists.name AS artist_name,
      tracks.artist_id,
      albums.name AS album_name,
      albums.album_artist_name,
      tracks.album_id,
      tracks.duration,
      tracks.track_number,
      albums.image_path,
      lyricsfiles.id AS lyricsfile_id,
      lyricsfiles.lyricsfile,
      COALESCE(lyricsfiles.instrumental, 0) AS instrumental
    FROM tracks
    JOIN albums ON tracks.album_id = albums.id
    JOIN artists ON tracks.artist_id = artists.id
    LEFT JOIN lyricsfiles ON lyricsfiles.track_id = tracks.id
    WHERE tracks.album_id = ?
    ORDER BY tracks.track_number ASC
  "})?;
    let mut rows = statement.query([album_id])?;
    let mut tracks: Vec<PersistentTrack> = Vec::new();

    while let Some(row) = rows.next()? {
        let is_instrumental: bool = row.get("instrumental")?;

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
            txt_lyrics: None,
            lrc_lyrics: None,
            lyricsfile: row.get("lyricsfile")?,
            lyricsfile_id: row.get("lyricsfile_id")?,
            image_path: row.get("image_path")?,
            instrumental: is_instrumental,
        };

        tracks.push(track);
    }

    Ok(tracks)
}

pub fn get_album_track_ids(
    album_id: i64,
    without_plain_lyrics: bool,
    without_synced_lyrics: bool,
    db: &Connection,
) -> Result<Vec<i64>> {
    let base_query = indoc! {"
      SELECT tracks.id
      FROM tracks
      JOIN albums ON tracks.album_id = albums.id
      LEFT JOIN lyricsfiles ON lyricsfiles.track_id = tracks.id
      WHERE tracks.album_id = ?"};

    let lyrics_conditions = match (without_plain_lyrics, without_synced_lyrics) {
        (true, true) => {
            " AND COALESCE(lyricsfiles.has_plain_lyrics, 0) = 0 AND COALESCE(lyricsfiles.has_synced_lyrics, 0) = 0 AND COALESCE(lyricsfiles.instrumental, 0) = 0"
        }
        (true, false) => " AND COALESCE(lyricsfiles.has_plain_lyrics, 0) = 0 AND COALESCE(lyricsfiles.instrumental, 0) = 0",
        (false, true) => " AND COALESCE(lyricsfiles.has_synced_lyrics, 0) = 0 AND COALESCE(lyricsfiles.instrumental, 0) = 0",
        (false, false) => "",
    };

    let full_query = format!(
        "{}{} ORDER BY tracks.track_number ASC",
        base_query, lyrics_conditions
    );

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
      SELECT tracks.id, tracks.file_path, tracks.file_name, tracks.title, artists.name AS artist_name,
        tracks.artist_id, albums.name AS album_name, albums.album_artist_name, tracks.album_id, tracks.duration, tracks.track_number,
        albums.image_path, lyricsfiles.id AS lyricsfile_id, lyricsfiles.lyricsfile, COALESCE(lyricsfiles.instrumental, 0) AS instrumental
      FROM tracks
      JOIN albums ON tracks.album_id = albums.id
      JOIN artists ON tracks.artist_id = artists.id
      LEFT JOIN lyricsfiles ON lyricsfiles.track_id = tracks.id
      WHERE tracks.artist_id = ?
      ORDER BY albums.name_lower ASC, tracks.track_number ASC
  "})?;
    let mut rows = statement.query([artist_id])?;
    let mut tracks: Vec<PersistentTrack> = Vec::new();

    while let Some(row) = rows.next()? {
        let is_instrumental: bool = row.get("instrumental")?;

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
            txt_lyrics: None,
            lrc_lyrics: None,
            lyricsfile: row.get("lyricsfile")?,
            lyricsfile_id: row.get("lyricsfile_id")?,
            image_path: row.get("image_path")?,
            instrumental: is_instrumental,
        };

        tracks.push(track);
    }

    Ok(tracks)
}

pub fn get_artist_track_ids(
    artist_id: i64,
    without_plain_lyrics: bool,
    without_synced_lyrics: bool,
    db: &Connection,
) -> Result<Vec<i64>> {
    let base_query = indoc! {"
      SELECT tracks.id
      FROM tracks
      JOIN albums ON tracks.album_id = albums.id
      JOIN artists ON tracks.artist_id = artists.id
      LEFT JOIN lyricsfiles ON lyricsfiles.track_id = tracks.id
      WHERE tracks.artist_id = ?"};

    let lyrics_conditions = match (without_plain_lyrics, without_synced_lyrics) {
        (true, true) => {
            " AND COALESCE(lyricsfiles.has_plain_lyrics, 0) = 0 AND COALESCE(lyricsfiles.has_synced_lyrics, 0) = 0 AND COALESCE(lyricsfiles.instrumental, 0) = 0"
        }
        (true, false) => " AND COALESCE(lyricsfiles.has_plain_lyrics, 0) = 0 AND COALESCE(lyricsfiles.instrumental, 0) = 0",
        (false, true) => " AND COALESCE(lyricsfiles.has_synced_lyrics, 0) = 0 AND COALESCE(lyricsfiles.instrumental, 0) = 0",
        (false, false) => "",
    };

    let full_query = format!(
        "{}{} ORDER BY albums.name_lower ASC, tracks.track_number ASC",
        base_query, lyrics_conditions
    );

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
/// Note: Lyrics are stored separately in the lyricsfiles table via upsert_lyricsfile_for_track_tx
pub fn insert_track_from_metadata_tx(
    metadata: &crate::scanner::metadata::TrackMetadata,
    _lyrics: &crate::scanner::metadata::LyricsInfo,
    file_size: i64,
    modified_time: i64,
    content_hash: &str,
    artist_id: i64,
    album_id: i64,
    tx: &rusqlite::Transaction,
) -> Result<i64> {
    use crate::utils::prepare_input;

    tx.execute(
        "INSERT INTO tracks (file_path, file_name, title, title_lower, album_id, artist_id, duration, track_number, file_size, modified_time, content_hash, scan_status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
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
        ],
    )?;

    Ok(tx.last_insert_rowid())
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

/// Get track IDs that have lyrics (for mass export)
pub fn get_track_ids_with_lyrics(db: &Connection) -> Result<Vec<i64>> {
    let mut statement = db.prepare(indoc! {"
      SELECT tracks.id
      FROM tracks
      JOIN lyricsfiles ON lyricsfiles.track_id = tracks.id
      WHERE lyricsfiles.has_plain_lyrics = 1
         OR lyricsfiles.has_synced_lyrics = 1
      ORDER BY tracks.artist_id ASC, tracks.album_id ASC, tracks.track_number ASC NULLS LAST
    "})?;

    let mut rows = statement.query([])?;
    let mut track_ids: Vec<i64> = Vec::new();

    while let Some(row) = rows.next()? {
        track_ids.push(row.get("id")?);
    }

    Ok(track_ids)
}

/// Find orphaned lyricsfiles by track metadata (for reattachment during scan)
/// Returns the lyricsfile_id if found, None otherwise
/// Matches on normalized title, artist, album, and duration within ±2 seconds
pub fn find_orphaned_lyricsfile_tx(
    title: &str,
    artist_name: &str,
    album_name: &str,
    duration: f64,
    tx: &rusqlite::Transaction,
) -> Result<Option<i64>> {
    let result = tx
        .query_row(
            indoc! {"
                SELECT id FROM lyricsfiles
                WHERE track_id IS NULL
                  AND track_title_lower = ?
                  AND track_artist_name_lower = ?
                  AND track_album_name_lower = ?
                  AND ABS(track_duration - ?) <= 2.0
                LIMIT 1
            "},
            [
                prepare_input(title),
                prepare_input(artist_name),
                prepare_input(album_name),
                duration.to_string(),
            ],
            |row| {
                let id: i64 = row.get(0)?;
                Ok(id)
            },
        )
        .optional()?;
    Ok(result)
}

/// Reattach an orphaned lyricsfile to a track
pub fn reattach_lyricsfile_to_track_tx(
    lyricsfile_id: i64,
    track_id: i64,
    tx: &rusqlite::Transaction,
) -> Result<()> {
    tx.execute(
        "UPDATE lyricsfiles SET track_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        (track_id, lyricsfile_id),
    )?;
    Ok(())
}

/// Find tracks by metadata for matching against LRCLIB tracks
pub fn find_tracks_by_metadata(
    title: &str,
    artist_name: Option<&str>,
    album_name: Option<&str>,
    duration: Option<f64>,
    db: &Connection,
) -> Result<Vec<PersistentTrack>> {
    let normalized_title = prepare_input(title);
    let normalized_artist = artist_name.map(prepare_input);
    let normalized_album = album_name.map(prepare_input);

    // Build the query based on available metadata
    // We search using normalized (name_lower) fields for case-insensitive matching
    let mut conditions = vec!["tracks.title_lower = ?"];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(normalized_title.clone())];

    if let Some(ref artist) = normalized_artist {
        conditions.push("artists.name_lower = ?");
        params.push(Box::new(artist.clone()));
    }

    if let Some(ref album) = normalized_album {
        conditions.push("albums.name_lower = ?");
        params.push(Box::new(album.clone()));
    }

    if let Some(dur) = duration {
        conditions.push("ABS(tracks.duration - ?) <= 2.0");
        params.push(Box::new(dur));
    }

    let where_clause = conditions.join(" AND ");

    let query = format!(
        indoc! {r#"
        SELECT
            tracks.id,
            tracks.file_path,
            tracks.file_name,
            tracks.title,
            artists.name AS artist_name,
            tracks.artist_id,
            albums.name AS album_name,
            albums.album_artist_name,
            tracks.album_id,
            tracks.duration,
            tracks.track_number,
            albums.image_path,
            lyricsfiles.id AS lyricsfile_id,
            lyricsfiles.lyricsfile,
            COALESCE(lyricsfiles.instrumental, 0) AS instrumental
        FROM tracks
        JOIN albums ON tracks.album_id = albums.id
        JOIN artists ON tracks.artist_id = artists.id
        LEFT JOIN lyricsfiles ON lyricsfiles.track_id = tracks.id
        WHERE {}
        ORDER BY tracks.title_lower ASC
    "#},
        where_clause
    );

    let mut statement = db.prepare(&query)?;

    // Convert params to rusqlite params
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut rows = statement.query(param_refs.as_slice())?;
    let mut tracks: Vec<PersistentTrack> = Vec::new();

    while let Some(row) = rows.next()? {
        let is_instrumental: bool = row.get("instrumental")?;

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
            txt_lyrics: None,
            lrc_lyrics: None,
            lyricsfile: row.get("lyricsfile")?,
            lyricsfile_id: row.get("lyricsfile_id")?,
            image_path: row.get("image_path")?,
            instrumental: is_instrumental,
        };

        tracks.push(track);
    }

    Ok(tracks)
}
