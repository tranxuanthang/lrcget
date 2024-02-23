use anyhow::Result;
use kira::{
  manager::{
    AudioManager, AudioManagerSettings,
    backend::DefaultBackend,
    MainPlaybackState
  },
  sound::{streaming::{StreamingSoundData, StreamingSoundSettings, StreamingSoundHandle}, FromFileError, PlaybackState},
  tween::Tween
};

use serde::Serialize;
use crate::persistent_entities::PersistentTrack;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerStatus {
  Playing,
  Paused,
  Stopped
}

#[derive(Serialize)]
pub struct Player {
  #[serde(skip)]
  manager: AudioManager,
  #[serde(skip)]
  sound_handle: Option<StreamingSoundHandle<FromFileError>>,
  pub track: Option<PersistentTrack>,
  pub status: PlayerStatus,
  pub progress: f64,
  pub duration: f64
}

impl Player {
  pub fn new() -> Player {
    let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

    Player {
      manager,
      sound_handle: None,
      track: None,
      status: PlayerStatus::Stopped,
      progress: 0.0,
      duration: 0.0
    }
  }

  pub fn renew_state(&mut self) {
    if let Some(ref mut sound_handle) = self.sound_handle {
      match sound_handle.state() {
        PlaybackState::Playing => self.status = PlayerStatus::Playing,
        PlaybackState::Pausing => self.status = PlayerStatus::Playing,
        PlaybackState::Stopping => self.status = PlayerStatus::Playing,
        PlaybackState::Paused => self.status = PlayerStatus::Paused,
        PlaybackState::Stopped => self.status = PlayerStatus::Stopped,
      }
    } else {
      self.status = PlayerStatus::Stopped
    }

    match self.sound_handle {
      Some(ref mut sound_handle) => {
        self.progress = sound_handle.position();
      }
      None => {}
    }
  }

  pub fn play(&mut self, track: PersistentTrack) -> Result<()> {
    let _ = self.stop();
    self.track = Some(track);

    if let Some(ref mut track) = self.track {
      let sound_data = StreamingSoundData::from_file(
        &track.file_path,
        StreamingSoundSettings::default()
      )?;

      self.duration = sound_data.duration().as_secs_f64();
      self.sound_handle = Some(self.manager.play(sound_data)?);
    }

    Ok(())
  }

  pub fn resume(&mut self) -> Result<()> {
    if let Some(ref mut sound_handle) = self.sound_handle {
      sound_handle.resume(Tween::default())?;
    }

    Ok(())
  }

  pub fn pause(&mut self) -> Result<()> {
    if let Some(ref mut sound_handle) = self.sound_handle {
      sound_handle.pause(Tween::default())?;
    }

    Ok(())
  }

  pub fn seek(&mut self, position: f64) -> Result<()> {
    if let Some(ref mut sound_handle) = self.sound_handle {
      sound_handle.pause(Tween::default())?;
      sound_handle.seek_to(position)?;
      sound_handle.resume(Tween::default())?;
    }

    Ok(())
  }

  pub fn stop(&mut self) -> Result<()> {
    if let Some(ref mut sound_handle) = self.sound_handle {
      sound_handle.stop(Tween::default())?;
      self.sound_handle = None;
      self.track = None;
      self.duration = 0.0;
      self.progress = 0.0;
      self.status = PlayerStatus::Stopped;
    }

    Ok(())
  }
}
