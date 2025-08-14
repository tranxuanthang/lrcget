use anyhow::Result;
use kira::{
    AudioManager,
    AudioManagerSettings,
    Decibels,
    DefaultBackend,
    sound::{
        streaming::{StreamingSoundData, StreamingSoundHandle},
        FromFileError, PlaybackState,
    },
    Tween,
};

use crate::persistent_entities::PersistentTrack;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerStatus {
    Playing,
    Paused,
    Stopped,
}

#[derive(Serialize)]
pub struct Player {
    #[serde(skip)]
    manager: AudioManager,
    #[serde(skip)]
    sound_handle: Option<StreamingSoundHandle<FromFileError>>,
    #[serde(skip)]
    pub track: Option<PersistentTrack>,
    pub status: PlayerStatus,
    pub progress: f64,
    pub duration: f64,
    pub volume: f64,
}

impl Player {
    pub fn new() -> Result<Player> {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;

        Ok(Player {
            manager,
            sound_handle: None,
            track: None,
            status: PlayerStatus::Stopped,
            progress: 0.0,
            duration: 0.0,
            volume: 1.0,
        })
    }

    pub fn renew_state(&mut self) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            match sound_handle.state() {
                PlaybackState::Playing => self.status = PlayerStatus::Playing,
                PlaybackState::Pausing => self.status = PlayerStatus::Playing,
                PlaybackState::Stopping => self.status = PlayerStatus::Playing,
                PlaybackState::WaitingToResume => self.status = PlayerStatus::Playing,
                PlaybackState::Resuming => self.status = PlayerStatus::Playing,
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
            let sound_data = StreamingSoundData::from_file(&track.file_path)?;

            self.duration = sound_data.duration().as_secs_f64();
            self.sound_handle = Some(self.manager.play(sound_data)?);
            self.sound_handle
                .as_mut()
                .unwrap()
                .set_volume(Self::volume_as_decibels(self.volume), Tween::default());
        }

        Ok(())
    }

    pub fn resume(&mut self) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.resume(Tween::default());
        }
    }

    pub fn pause(&mut self) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.pause(Tween::default());
        }
    }

    pub fn seek(&mut self, position: f64) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            match sound_handle.state() {
                PlaybackState::Playing => sound_handle.seek_to(position),
                _ => {
                    sound_handle.seek_to(position);
                    sound_handle.resume(Tween::default());
                }
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.stop(Tween::default());
            self.sound_handle = None;
            self.track = None;
            self.duration = 0.0;
            self.progress = 0.0;
            self.status = PlayerStatus::Stopped;
        }
    }

    /// Kira doesn't provide a way to create Decibels from an amplitude.
    /// Invert the formula in Decibels::as_amplitude():
    /// original:                         amp = 10 ^ (db / 20)
    /// take log() of both sides:         log(amp) = log(10 ^ (db / 20))
    /// identity log(a^b) = b*log(a):     log(amp) = (db / 20) * log(10)
    /// divide both sides by log(10):     log(amp) / log(10) = db / 20
    /// divide by log(10) is log base 10: log10(amp) = db / 20
    /// multiple both sides by 20:        20 * log10(amp) = db
    pub(crate) fn volume_as_decibels(volume: f64) -> Decibels {
        if volume <= 0.0 {
            Decibels::SILENCE
        } else if volume == 1.0 {
            Decibels::IDENTITY
        } else {
            Decibels((20.0 * volume.log10()) as f32)
        }
    }

    pub fn set_volume(&mut self, volume: f64) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.set_volume(Self::volume_as_decibels(volume), Tween::default());
        }
        self.volume = volume;
    }
}

#[cfg(test)]
mod tests {
    use kira::Decibels;
    use super::Player;

    #[test]
    fn test_volume_as_decibels() {
        let decibels = [
            Decibels::IDENTITY,
            Decibels(3.0),
            Decibels(12.0),
            Decibels(-3.0),
            Decibels(-12.0),
            Decibels::SILENCE,
        ];
        for db_expected in decibels {
            let db_actual = Player::volume_as_decibels(db_expected.as_amplitude() as f64);
            assert!((db_expected.0 - db_actual.0) < 1e-5, "{} != {}", db_expected.0, db_actual.0);
        }
    }
}
