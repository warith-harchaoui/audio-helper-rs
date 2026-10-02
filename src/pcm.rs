//! `load_audio` from the Python original: decode any ffmpeg-readable file (or the
//! audio track of a video container) into in-memory `f32` PCM. Where the Python
//! returns a `torch.Tensor`/`np.ndarray`, this returns a plain `Vec<f32>` — no
//! tensor library dependency for a library crate.

use crate::convert::{convert, ConvertOptions};
use crate::error::{AudioHelperError, Result};
use crate::probe::native_format;
use std::path::Path;

#[derive(Debug, Clone)]
/// How to decode a file into memory: resampling, and how many channels to
/// come back with.
pub struct LoadOptions {
    /// Resample to this rate; `None` keeps the file's native sample rate.
    pub target_sample_rate: Option<u32>,
    /// Down-mix to mono (default `true`, matching the Python original).
    pub to_mono: bool,
    /// Force stereo output when `to_mono` is `false`. Ignored when `to_mono` is
    /// `true` (mono takes precedence, same order of precedence as the Python).
    pub two_channels: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            target_sample_rate: None,
            to_mono: true,
            two_channels: false,
        }
    }
}

#[derive(Debug, Clone)]
/// Decoded audio held in memory, interleaved when there is more than one
/// channel.
pub struct PcmBuffer {
    /// The samples, interleaved across channels, in `-1.0..=1.0`.
    pub samples: Vec<f32>,
    /// Sample rate of these samples, in hertz.
    pub sample_rate: u32,
    /// How many channels the samples are interleaved across.
    pub channels: u32,
}

/// Decodes `path` into memory as interleaved `f32` PCM.
pub fn load(path: &Path, options: &LoadOptions) -> Result<PcmBuffer> {
    let (native_sr, native_ch) = native_format(path)?;
    let sample_rate = options.target_sample_rate.unwrap_or(native_sr);
    let channels = if options.to_mono {
        1
    } else if options.two_channels {
        2
    } else {
        native_ch.max(1)
    };

    let samples = crate::ffmpeg::decode_to_pcm_f32(path, sample_rate, channels)?;
    Ok(PcmBuffer {
        samples,
        sample_rate,
        channels,
    })
}

/// Writes interleaved PCM back to a file — the inverse of [`load`].
///
/// A `.wav` output is written directly with [`hound`] as 16-bit signed PCM, the
/// one format every tool reads. Any other extension goes through a temporary WAV
/// and [`crate::convert`], so the container and codec follow the extension
/// exactly as they do everywhere else in this crate, and `ffmpeg` is only needed
/// for the non-WAV case.
///
/// Samples outside `-1.0..=1.0` are clamped, not wrapped: an overshooting sample
/// is a mixing mistake, and wrapping turns it into a loud click that is far
/// harder to trace back than clipping.
///
/// # Errors
/// - [`AudioHelperError::InvalidPcmLayout`] if the buffer does not describe whole
///   interleaved frames (see that variant).
/// - [`AudioHelperError::Wav`] if the WAV itself could not be written.
/// - whatever [`crate::convert`] returns, for a non-WAV output.
pub fn save(buffer: &PcmBuffer, path: &Path) -> Result<()> {
    let channels = u16::try_from(buffer.channels).ok().filter(|c| *c > 0);
    let Some(channels) = channels else {
        return Err(invalid_layout(buffer));
    };
    if buffer.sample_rate == 0 || !buffer.samples.len().is_multiple_of(channels as usize) {
        return Err(invalid_layout(buffer));
    }

    let is_wav_out = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("wav"));

    if is_wav_out {
        return write_wav(buffer, channels, path);
    }

    // Non-WAV: write the canonical WAV first, then let `convert` (and therefore
    // ffmpeg) decide the codec from the extension. The temporary file is deleted
    // when `scratch` drops, on the error path as well as the success path.
    let scratch = tempfile::Builder::new().suffix(".wav").tempfile()?;
    write_wav(buffer, channels, scratch.path())?;
    convert(
        scratch.path(),
        path,
        &ConvertOptions {
            sample_rate: buffer.sample_rate,
            channels: buffer.channels,
            // Ignored for a non-WAV container; ffmpeg's own default codec for the
            // target container is the right answer there.
            encoding: None,
            overwrite: true,
        },
    )
}

fn invalid_layout(buffer: &PcmBuffer) -> AudioHelperError {
    AudioHelperError::InvalidPcmLayout {
        samples: buffer.samples.len(),
        channels: buffer.channels,
        sample_rate: buffer.sample_rate,
    }
}

/// Writes `buffer` as 16-bit signed PCM WAV at `path`.
fn write_wav(buffer: &PcmBuffer, channels: u16, path: &Path) -> Result<()> {
    let spec = hound::WavSpec {
        channels,
        sample_rate: buffer.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for sample in &buffer.samples {
        // Scale by 32767, not 32768: +1.0 then lands exactly on i16::MAX and
        // -1.0 on -32767, one step short of i16::MIN. The asymmetry is what keeps
        // a full-scale sample from wrapping to the opposite sign.
        let scaled = sample.clamp(-1.0, 1.0) * i16::MAX as f32;
        writer.write_sample(scaled.round() as i16)?;
    }
    writer.finalize()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{require_ffmpeg, write_test_wav};

    /// `save` of a `.wav` needs no ffmpeg at all — the point of writing it with
    /// `hound` directly — so this one is deliberately unguarded.
    #[test]
    fn saves_a_wav_without_ffmpeg_and_reads_back_the_same_shape() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("saved.wav");
        let buffer = PcmBuffer {
            samples: vec![0.0, 0.5, -0.5, 1.0],
            sample_rate: 8_000,
            channels: 2,
        };
        save(&buffer, &out).unwrap();

        let reader = hound::WavReader::open(&out).unwrap();
        let spec = reader.spec();
        assert_eq!(spec.sample_rate, 8_000);
        assert_eq!(spec.channels, 2);
        assert_eq!(reader.len(), 4);
    }

    #[test]
    fn out_of_range_samples_clip_instead_of_wrapping() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("hot.wav");
        // +2.0 and -2.0 are twice full scale. Wrapped, they would come back with
        // their sign flipped — the loud click this clamp exists to prevent.
        let buffer = PcmBuffer {
            samples: vec![2.0, -2.0],
            sample_rate: 8_000,
            channels: 1,
        };
        save(&buffer, &out).unwrap();

        let mut reader = hound::WavReader::open(&out).unwrap();
        let written: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(written, vec![i16::MAX, -i16::MAX]);
    }

    #[test]
    fn a_buffer_that_is_not_whole_frames_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("ragged.wav");
        // Three samples cannot be two interleaved stereo frames.
        let buffer = PcmBuffer {
            samples: vec![0.0, 0.1, 0.2],
            sample_rate: 8_000,
            channels: 2,
        };
        let err = save(&buffer, &out).unwrap_err();
        assert!(matches!(err, AudioHelperError::InvalidPcmLayout { .. }));
        assert!(
            !out.exists(),
            "nothing should be written for a refused buffer"
        );
    }

    #[test]
    fn zero_channels_and_zero_sample_rate_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        for buffer in [
            PcmBuffer {
                samples: vec![0.0],
                sample_rate: 8_000,
                channels: 0,
            },
            PcmBuffer {
                samples: vec![0.0],
                sample_rate: 0,
                channels: 1,
            },
        ] {
            let err = save(&buffer, &dir.path().join("nope.wav")).unwrap_err();
            assert!(matches!(err, AudioHelperError::InvalidPcmLayout { .. }));
        }
    }

    #[test]
    fn load_then_save_round_trips_through_a_real_file() {
        require_ffmpeg!();
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("tone.wav");
        write_test_wav(&source, 16_000, 0.5);

        let loaded = load(&source, &LoadOptions::default()).unwrap();
        let out = dir.path().join("round-trip.wav");
        save(&loaded, &out).unwrap();

        let reloaded = load(&out, &LoadOptions::default()).unwrap();
        assert_eq!(reloaded.sample_rate, loaded.sample_rate);
        assert_eq!(reloaded.channels, loaded.channels);
        assert_eq!(reloaded.samples.len(), loaded.samples.len());
        // 16-bit quantisation is the only loss in the round trip, so the signal
        // must survive it: compare the peaks rather than sample-by-sample.
        let peak = |s: &[f32]| s.iter().fold(0.0_f32, |m, v| m.max(v.abs()));
        assert!((peak(&reloaded.samples) - peak(&loaded.samples)).abs() < 0.01);
    }

    #[test]
    fn a_non_wav_extension_is_encoded_through_ffmpeg() {
        require_ffmpeg!();
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("saved.mp3");
        let buffer = PcmBuffer {
            samples: (0..8_000)
                .map(|i| (i as f32 / 8_000.0 * std::f32::consts::TAU * 440.0).sin() * 0.2)
                .collect(),
            sample_rate: 8_000,
            channels: 1,
        };
        save(&buffer, &out).unwrap();
        assert!(crate::probe::is_valid_audio_file(&out));
    }

    #[test]
    fn loads_and_resamples_to_target_rate() {
        require_ffmpeg!();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tone.wav");
        write_test_wav(&path, 44_100, 0.5);

        let options = LoadOptions {
            target_sample_rate: Some(16_000),
            ..Default::default()
        };
        let pcm = load(&path, &options).unwrap();

        assert_eq!(pcm.sample_rate, 16_000);
        assert_eq!(pcm.channels, 1);
        // 0.5s at 16kHz mono, allow a little slack for ffmpeg's resampler.
        assert!(
            (pcm.samples.len() as i64 - 8_000).abs() < 200,
            "got {} samples",
            pcm.samples.len()
        );
    }

    #[test]
    fn defaults_to_native_sample_rate_when_unset() {
        require_ffmpeg!();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tone.wav");
        write_test_wav(&path, 22_050, 0.2);

        let pcm = load(&path, &LoadOptions::default()).unwrap();
        assert_eq!(pcm.sample_rate, 22_050);
    }
}
