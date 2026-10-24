use std::fs::File;
use std::path::Path;
use symphonia::core::audio::{SampleBuffer, SignalSpec};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("Failed to open file: {0}")]
    OpenFailed(#[from] std::io::Error),

    #[error("Symphonia error: {0}")]
    Symphonia(#[from] SymphoniaError),

    #[error("No default audio track found in file")]
    NoDefaultTrack,

    #[error("Audio track is missing sample rate metadata")]
    MissingSampleRate,

    #[error("Invalid time range: start_ms={start_ms}, end_ms={end_ms}")]
    InvalidRange { start_ms: i64, end_ms: i64 },
}

pub struct AudioSlice {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

/// Decode the audio file at `path` between `[start_ms, end_ms)` and return
/// mono f32 samples normalized to roughly [-1.0, 1.0]. Multi-channel sources
/// are downmixed by averaging.
pub fn decode_slice(path: &Path, start_ms: i64, end_ms: i64) -> Result<AudioSlice, DecoderError> {
    if end_ms <= start_ms || start_ms < 0 {
        return Err(DecoderError::InvalidRange { start_ms, end_ms });
    }

    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;

    let track = format
        .default_track()
        .ok_or(DecoderError::NoDefaultTrack)?;
    let track_id = track.id;

    let params = &track.codec_params;
    let sample_rate = params.sample_rate.ok_or(DecoderError::MissingSampleRate)?;
    let n_channels = params.channels.map(|c| c.count()).unwrap_or(1).max(1);

    let mut decoder = symphonia::default::get_codecs().make(params, &DecoderOptions::default())?;

    let start_seconds = start_ms as f64 / 1000.0;
    let _ = format.seek(
        SeekMode::Accurate,
        SeekTo::Time {
            time: Time::from(start_seconds),
            track_id: Some(track_id),
        },
    )?;

    let start_sample_pos = (start_seconds * sample_rate as f64).round() as u64;
    let end_sample_pos = ((end_ms as f64 / 1000.0) * sample_rate as f64).round() as u64;
    let target_samples = end_sample_pos.saturating_sub(start_sample_pos) as usize;

    let mut samples: Vec<f32> = Vec::with_capacity(target_samples);

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => return Err(DecoderError::Symphonia(e)),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(DecoderError::Symphonia(e)),
        };

        let spec: SignalSpec = *decoded.spec();
        let frames = decoded.frames();
        if frames == 0 {
            continue;
        }

        let packet_start = packet.ts;
        let packet_end = packet_start + frames as u64;

        if packet_end <= start_sample_pos {
            continue;
        }
        if packet_start >= end_sample_pos {
            break;
        }

        let skip_frames = if packet_start < start_sample_pos {
            (start_sample_pos - packet_start) as usize
        } else {
            0
        };

        let usable_frames_after_skip = frames.saturating_sub(skip_frames);
        let remaining_needed = target_samples.saturating_sub(samples.len());
        let frames_to_take = usable_frames_after_skip.min(remaining_needed);

        if frames_to_take == 0 {
            if samples.len() >= target_samples {
                break;
            }
            continue;
        }

        let mut sample_buf = SampleBuffer::<f32>::new(frames as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);
        let interleaved = sample_buf.samples();

        let channels = n_channels;
        let inv_channels = 1.0_f32 / channels as f32;
        for frame_idx in skip_frames..(skip_frames + frames_to_take) {
            let base = frame_idx * channels;
            let mut sum = 0.0_f32;
            for ch in 0..channels {
                sum += interleaved[base + ch];
            }
            samples.push(sum * inv_channels);
        }

        if samples.len() >= target_samples {
            break;
        }
    }

    Ok(AudioSlice {
        samples,
        sample_rate,
    })
}
