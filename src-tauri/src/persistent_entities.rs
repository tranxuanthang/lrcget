use serde::Serialize;

/// A track that can be played - either from database or from a file path.
/// This is used by the player and can represent both scanned library tracks
/// and arbitrary tracks selected via file picker.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayableTrack {
    pub id: Option<i64>, // None for file-based tracks not in database
    pub file_path: String,
    pub file_name: String,
    pub title: String,
    pub album_name: String,
    pub artist_name: String,
    pub album_artist_name: Option<String>,
    pub image_path: Option<String>,
    pub track_number: Option<i64>,
    pub duration: f64,
    pub instrumental: bool,
    pub lyricsfile: Option<String>,
    pub lyricsfile_id: Option<i64>, // ID from lyricsfiles table
}

impl From<PersistentTrack> for PlayableTrack {
    fn from(track: PersistentTrack) -> Self {
        Self {
            id: Some(track.id),
            file_path: track.file_path,
            file_name: track.file_name,
            title: track.title,
            album_name: track.album_name,
            artist_name: track.artist_name,
            album_artist_name: track.album_artist_name,
            image_path: track.image_path,
            track_number: track.track_number,
            duration: track.duration,
            instrumental: track.instrumental,
            lyricsfile: track.lyricsfile,
            lyricsfile_id: track.lyricsfile_id,
        }
    }
}

#[derive(Clone, Serialize)]
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
    pub lyricsfile: Option<String>,
    pub lyricsfile_id: Option<i64>, // ID from lyricsfiles table (null if no lyricsfile exists)
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
    pub skip_tracks_with_synced_lyrics: bool,
    pub skip_tracks_with_plain_lyrics: bool,
    pub show_line_count: bool,
    pub try_embed_lyrics: bool,
    pub theme_mode: String,
    pub lrclib_instance: String,
    pub volume: f64,
}
