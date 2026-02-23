pub mod hasher;
pub mod metadata;
pub mod models;
pub mod scan;

pub use metadata::{extract_track_info, is_instrumental_lyrics, LyricsInfo, TrackMetadata};
pub use scan::{scan_library, DetectionMethod};
