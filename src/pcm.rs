//! `load_audio` from the Python original: decode any ffmpeg-readable file (or the
//! audio track of a video container) into in-memory `f32` PCM. Where the Python
//! returns a `torch.Tensor`/`np.ndarray`, this returns a plain `Vec<f32>` — no
//! tensor library dependency for a library crate.

use crate::error::Result;
use crate::probe::native_format;
use std::path::Path;

#[derive(Debug, Clone)]
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
pub struct PcmBuffer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::write_test_wav;

    #[test]
    fn loads_and_resamples_to_target_rate() {
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
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tone.wav");
        write_test_wav(&path, 22_050, 0.2);

        let pcm = load(&path, &LoadOptions::default()).unwrap();
        assert_eq!(pcm.sample_rate, 22_050);
    }
}
