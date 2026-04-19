use crate::db;
use crate::persistent_entities::{PersistentAlbum, PersistentArtist, PersistentTrack};
use anyhow::Result;
use rusqlite::Connection;

pub fn uninitialize_library(conn: &Connection) -> Result<()> {
    db::clean_library(conn)?;
    db::set_init(false, conn)?;
    Ok(())
}

/// Full wipe of library data including associated lyricsfiles.
/// Clears tracks, albums, artists tables, deletes lyricsfiles with track associations,
/// and resets the init flag to trigger a full rescan.
pub fn full_wipe_library(conn: &Connection) -> Result<()> {
    db::clean_library(conn)?;
    db::delete_lyricsfiles_with_tracks(conn)?;
    db::set_init(false, conn)?;
    Ok(())
}

pub fn get_tracks(conn: &Connection) -> Result<Vec<PersistentTrack>> {
    db::get_tracks(conn)
}

pub fn get_track_ids(
    search_query: Option<String>,
    synced_lyrics: bool,
    plain_lyrics: bool,
    instrumental: bool,
    no_lyrics: bool,
    conn: &Connection,
) -> Result<Vec<i64>> {
    match search_query {
        Some(query) => db::get_search_track_ids(
            &query,
            synced_lyrics,
            plain_lyrics,
            instrumental,
            no_lyrics,
            conn,
        ),
        None => db::get_track_ids(synced_lyrics, plain_lyrics, instrumental, no_lyrics, conn),
    }
}

pub fn get_track(id: i64, conn: &Connection) -> Result<PersistentTrack> {
    db::get_track_by_id(id, conn)
}

pub fn get_albums(conn: &Connection) -> Result<Vec<PersistentAlbum>> {
    db::get_albums(conn)
}

pub fn get_album_ids(conn: &Connection) -> Result<Vec<i64>> {
    db::get_album_ids(conn)
}

pub fn get_album(id: i64, conn: &Connection) -> Result<PersistentAlbum> {
    db::get_album_by_id(id, conn)
}

pub fn get_artists(conn: &Connection) -> Result<Vec<PersistentArtist>> {
    db::get_artists(conn)
}

pub fn get_artist_ids(conn: &Connection) -> Result<Vec<i64>> {
    db::get_artist_ids(conn)
}

pub fn get_artist(id: i64, conn: &Connection) -> Result<PersistentArtist> {
    db::get_artist_by_id(id, conn)
}

pub fn get_album_tracks(album_id: i64, conn: &Connection) -> Result<Vec<PersistentTrack>> {
    db::get_album_tracks(album_id, conn)
}

pub fn get_artist_tracks(artist_id: i64, conn: &Connection) -> Result<Vec<PersistentTrack>> {
    db::get_artist_tracks(artist_id, conn)
}

pub fn get_album_track_ids(
    album_id: i64,
    without_plain_lyrics: bool,
    without_synced_lyrics: bool,
    conn: &Connection,
) -> Result<Vec<i64>> {
    db::get_album_track_ids(album_id, without_plain_lyrics, without_synced_lyrics, conn)
}

pub fn get_artist_track_ids(
    artist_id: i64,
    without_plain_lyrics: bool,
    without_synced_lyrics: bool,
    conn: &Connection,
) -> Result<Vec<i64>> {
    db::get_artist_track_ids(artist_id, without_plain_lyrics, without_synced_lyrics, conn)
}

pub fn get_init(conn: &Connection) -> Result<bool> {
    db::get_init(conn)
}
