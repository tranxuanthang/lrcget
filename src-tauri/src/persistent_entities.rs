use serde::Serialize;

#[derive(Serialize)]
pub struct PersistentTrack {
    pub id: i64,
    pub file_path: String,
    pub file_name: String,
    pub title: String,
    pub album_name: String,
    pub album_artist_name: Option<String>,
    pub album_id: i64,
    pub artist_name: String,
    pub artist_id: i64,
    pub image_path: Option<String>,
    pub track_number: Option<i64>,
    pub txt_lyrics: Option<String>,
    pub lrc_lyrics: Option<String>,
    pub duration: f64,
    pub instrumental: bool,
}

#[derive(Serialize)]
pub struct PersistentAlbum {
    pub id: i64,
    pub name: String,
    pub image_path: Option<String>,
    pub artist_name: String,
    pub album_artist_name: Option<String>,
    pub tracks_count: i64,
}

#[derive(Serialize)]
pub struct PersistentArtist {
    pub id: i64,
    pub name: String,
    // pub albums_count: i64,
    pub tracks_count: i64,
}

#[derive(Serialize)]
pub struct PersistentConfig {
    pub skip_not_needed_tracks: bool,
    pub try_embed_lyrics: bool,
    pub theme_mode: String,
    pub lrclib_instance: String,
}
