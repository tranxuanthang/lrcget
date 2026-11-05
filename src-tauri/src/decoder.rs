use serde::Serialize;
use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSlice {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

/// Decodes `[start_ms, end_ms)` of the file at `path` to mono f32 samples, averaging channels.
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

    let track = format.default_track().ok_or(DecoderError::NoDefaultTrack)?;
    let track_id = track.id;

    let params = &track.codec_params;
    let sample_rate = params.sample_rate.ok_or(DecoderError::MissingSampleRate)?;
    let time_base = params.time_base;

    let mut decoder = symphonia::default::get_codecs().make(params, &DecoderOptions::default())?;

    let seeked = format.seek(
        SeekMode::Accurate,
        SeekTo::Time {
            time: Time::from(start_ms as f64 / 1000.0),
            track_id: Some(track_id),
        },
    )?;

    // Track timestamps count time-base ticks, or frames when the track has no time base.
    let ts_to_frame = |ts: u64| match time_base {
        Some(tb) => {
            let scaled = u128::from(ts) * u128::from(tb.numer) * u128::from(sample_rate);
            (scaled / u128::from(tb.denom)) as u64
        }
        None => ts,
    };

    let start_frame = ts_to_frame(seeked.required_ts);
    let frame_count =
        ((end_ms - start_ms) as f64 * f64::from(sample_rate) / 1000.0).round() as usize;
    let end_frame = start_frame + frame_count as u64;

    let mut samples: Vec<f32> = Vec::with_capacity(frame_count);
    let mut sample_buf: Option<SampleBuffer<f32>> = None;

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

        let frames = decoded.frames();
        if frames == 0 {
            continue;
        }

        let packet_start = ts_to_frame(packet.ts());
        let packet_end = packet_start + frames as u64;

        if packet_end <= start_frame {
            continue;
        }
        if packet_start >= end_frame {
            break;
        }

        let skip_frames = start_frame.saturating_sub(packet_start) as usize;
        let usable_frames_after_skip = frames.saturating_sub(skip_frames);
        let remaining_needed = frame_count.saturating_sub(samples.len());
        let frames_to_take = usable_frames_after_skip.min(remaining_needed);

        if frames_to_take == 0 {
            if samples.len() >= frame_count {
                break;
            }
            continue;
        }

        let spec = *decoded.spec();
        let channels = spec.channels.count();
        // SampleBuffer capacity counts samples; the decoded buffer's capacity counts frames.
        let buf = match &mut sample_buf {
            Some(buf) if buf.capacity() >= frames * channels => buf,
            _ => sample_buf.insert(SampleBuffer::new(decoded.capacity() as u64, spec)),
        };
        buf.copy_interleaved_ref(decoded);
        let interleaved = buf.samples();

        let inv_channels = 1.0_f32 / channels as f32;
        for frame_idx in skip_frames..(skip_frames + frames_to_take) {
            let base = frame_idx * channels;
            let mut sum = 0.0_f32;
            for ch in 0..channels {
                sum += interleaved[base + ch];
            }
            samples.push(sum * inv_channels);
        }

        if samples.len() >= frame_count {
            break;
        }
    }

    Ok(AudioSlice {
        samples,
        sample_rate,
    })
}
