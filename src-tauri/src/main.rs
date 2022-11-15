#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::error::Error;
use std::fmt;
use audiotags;
use globwalk::glob;
use id3::{TagLike};
use metaflac;
use reqwest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;
use window_shadows::set_shadow;
use mp3_duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MusicItem {
  file_path: String,
  file_name: String,
  track_name: String,
  album_name: String,
  artist_name: String,
  lyrics: String,
  duration: f64
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct LrcLibGetResponse {
  plain_lyrics: Option<String>,
  synced_lyrics: Option<String>,
  instrumental: bool,
  lang: Option<String>,
  isrc: Option<String>,
  spotify_id: Option<String>,
  name: Option<String>,
  album_name: Option<String>,
  artist_name: Option<String>,
  release_date: Option<String>,
  duration: Option<f64>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct LrcLibResponseError {
  status_code: Option<u16>,
  error: String,
  message: String
}
impl Error for LrcLibResponseError {}
impl fmt::Display for LrcLibResponseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "{}: {}", self.error, self.message)
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ApplyLyricsProgressPayload {
  progress: f32,
  successed_items_count: u32,
  failed_items_count: u32,
  processed_items_count: u32,
  total_items_count: u32,
  current_track: MusicItem,
  current_status: LrcLibStatus,
  current_message: Option<String>,
  lrclib_response: Option<LrcLibGetResponse>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum LrcLibStatus {
  Success,
  Failure
}

#[tauri::command]
async fn get_music_files(directory: String, app_handle: tauri::AppHandle) -> Option<Vec<MusicItem>> {
  let files = glob(format!("{}\\**\\*.{{mp3,m4a,flac}}", directory));
  match files {
    Ok(mut files) => {
      let mut result: Vec<MusicItem> = Vec::new();

      for file in &mut files {
        if let Ok(file) = file {
          let file_extension = file
            .path()
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

          let tag = audiotags::Tag::new().read_from_path(file.path());
          let mut lyrics = "".to_owned();

          /*
          if file_extension == "flac" {
            let flactag = metaflac::Tag::read_from_path(file.path()).unwrap();
            let synced_lyrics_list_result = flactag.get_vorbis("LYRICS");
            if let Some(synced_lyrics_list) = synced_lyrics_list_result {
              for synced_lyrics in synced_lyrics_list.take(1) {
                lyrics = synced_lyrics.to_owned();
              }
            }
          }

          if file_extension == "mp3" {
            let id3tag = id3::Tag::read_from_path(file.path()).unwrap();

            for synced_lyrics in id3tag.synchronised_lyrics() {
              let mut lyrics_builder = lrc::Lyrics::new();
              for timed_line in &synced_lyrics.content {
                lyrics_builder.add_timed_line(lrc::TimeTag::new(timed_line.0), &timed_line.1).unwrap();
              }
              lyrics = lyrics_builder.to_string();
            }
          }
          */

          let lrc_file_path = lrc_file_path(file.path().display().to_string());
          let lrc_content = std::fs::read_to_string(lrc_file_path);

          if let Ok(lrc_content) = lrc_content {
            lyrics = lrc_content;
          }

          if let Ok(tag) = tag {
            let track_name = tag.title().unwrap_or("").to_owned();
            let mut album_name = tag.album_title().unwrap_or("").to_owned();
            let mut artist_name = tag.artist().unwrap_or("").to_owned();
            // let mut artist_name = tag.artists().unwrap_or(Vec::new()).join(", ").to_owned();

            if file_extension == "mp3" {
              let id3tag = id3::Tag::read_from_path(file.path()).unwrap();

              if let Some(album) = id3tag.album() {
                album_name = album.to_owned();
              }

              if let Some(artist) = id3tag.artist() {
                artist_name = artist.to_owned();
              }
            }

            let duration = match tag.duration() {
              Some(duration) => duration,
              None => {
                if file_extension == "mp3" {
                  match mp3_duration::from_path(file.path()) {
                    Ok(duration) => duration.as_secs_f64(),
                    Err(_) => 0.0
                  }
                } else {
                  0.0
                }
              }
            };

            if !track_name.is_empty()
              && !album_name.is_empty()
              && !artist_name.is_empty()
            {
              result.push(MusicItem {
                file_path: file.path().display().to_string(),
                file_name: file
                  .path()
                  .file_name()
                  .unwrap()
                  .to_str()
                  .unwrap()
                  .to_owned(),
                track_name,
                album_name,
                artist_name,
                lyrics,
                duration
              });
            }
          }
        }
      }

      Some(result)
    }
    Err(_) => None,
  }
}

async fn get_lyrics_from_lrclib(
  music_item: &MusicItem,
) -> Result<LrcLibGetResponse, Box<dyn Error>> {
  let params = vec![
    ("artist_name", music_item.artist_name.to_owned()),
    ("track_name", music_item.track_name.to_owned()),
    ("album_name", music_item.album_name.to_owned()),
    ("duration", music_item.duration.round().to_string()),
  ];

  let url = reqwest::Url::parse_with_params("https://lrclib.net/api/get", &params)?;
  let res = reqwest::get(url).await?;

  match res.status() {
    reqwest::StatusCode::OK => {
      Ok(res.json::<LrcLibGetResponse>().await?)
    },

    reqwest::StatusCode::NOT_FOUND | reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::SERVICE_UNAVAILABLE | reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
      Err(res.json::<LrcLibResponseError>().await?.into())
    },

    _ => {
      Err(LrcLibResponseError {
        status_code: None,
        error: "UnknownError".to_string(),
        message: "Unknown error happened".to_string()
      }.into())
    }
  }
}

fn lrc_file_path(file_path: String) -> PathBuf {
  let path = PathBuf::from(file_path);
  let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
  let parent_path = path.parent().unwrap();
  let file_name_without_extension = std::path::Path::new(&file_name)
    .file_stem()
    .unwrap()
    .to_str()
    .unwrap()
    .to_owned();
  let mut lrc_file_name = file_name_without_extension.to_owned();
  lrc_file_name.push_str(".lrc");

  let lrc_file_path = parent_path.join(lrc_file_name);

  lrc_file_path
}

#[tauri::command]
async fn apply_lyrics(
  music_items: Vec<MusicItem>,
  is_create_lrc: bool,
  is_embed: bool,
  skip_tracks_have_existing_lyrics: bool,
  app_handle: tauri::AppHandle,
) {
  let mut processed_items_count = 0;
  let mut successed_items_count = 0;
  let mut failed_items_count = 0;
  let mut status: LrcLibStatus;
  let mut message: Option<String>;

  for music_item in &music_items {
    if !music_item.lyrics.is_empty() && skip_tracks_have_existing_lyrics == true {
      continue;
    }

    let data = get_lyrics_from_lrclib(&music_item).await;

    match data {
      Ok(data) => {
        let synced_lyrics = &data.synced_lyrics.clone().unwrap_or_default();
        if !synced_lyrics.is_empty() {
          let lrc_file_path = lrc_file_path((&music_item.file_path).to_owned());
          std::fs::write(lrc_file_path, synced_lyrics).expect("Unable to write file");
          successed_items_count = successed_items_count + 1;
          status = LrcLibStatus::Success;
          message = Some("Retrieved lyrics successfully".to_string());
        } else if *&data.instrumental {
          failed_items_count = failed_items_count + 1;
          status = LrcLibStatus::Failure;
          message = Some("Track is instrumental".to_string());
        } else {
          failed_items_count = failed_items_count + 1;
          status = LrcLibStatus::Failure;
          message = Some("Track has no synced lyrics submitted".to_string());
        }

        processed_items_count = processed_items_count + 1;

        report_progress(
          &music_items, music_item, &app_handle, successed_items_count,
          failed_items_count, processed_items_count, status, message,
          &Some(data)
        )
      },

      Err(dyn_err) => {
        failed_items_count = failed_items_count + 1;

        let error = dyn_err.downcast_ref::<LrcLibResponseError>();

        match error {
          Some(error) => {
            status = LrcLibStatus::Failure;
            message = Some((&error.message).to_owned());

            processed_items_count = processed_items_count + 1;

            report_progress(
              &music_items, music_item, &app_handle, successed_items_count,
              failed_items_count, processed_items_count, status, message,
              &None
            )
          },
          None => {
            status = LrcLibStatus::Failure;
            message = Some("Unknown Error".to_string());

            processed_items_count = processed_items_count + 1;

            report_progress(
              &music_items, music_item, &app_handle, successed_items_count,
              failed_items_count, processed_items_count, status, message,
              &None
            )
          }
        }
      }
    }
  }
}

fn report_progress(
  music_items: &Vec<MusicItem>, music_item: &MusicItem, app_handle: &tauri::AppHandle, successed_items_count: u32,
  failed_items_count: u32, processed_items_count: u32, current_status: LrcLibStatus, current_message: Option<String>,
  lrclib_response: &Option<LrcLibGetResponse>) {
    let music_items = music_items.clone();
    let music_item = music_item.clone();
    let app_handle = app_handle.clone();
    let lrclib_response = lrclib_response.clone();

    std::thread::spawn(move || {
      let total_items_count = music_items.len();
      app_handle
        .emit_all(
          "apply-lyrics-progress",
          ApplyLyricsProgressPayload {
            progress: (processed_items_count as f32) / (music_items.len() as f32),
            successed_items_count,
            failed_items_count,
            processed_items_count,
            total_items_count: total_items_count.try_into().unwrap(),
            current_track: music_item,
            current_status,
            current_message,
            lrclib_response
          },
        )
        .unwrap();
    });
}

#[tauri::command]
fn open_devtools(window: tauri::Window) {
  #[cfg(debug_assertions)]
  {
    window.open_devtools();
  }
}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let window = app.get_window("main").unwrap();

      #[cfg(target_os = "windows")]
      set_shadow(&window, true).expect("Unsupported platform!");

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      get_music_files,
      apply_lyrics,
      open_devtools
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
