//! `sound_converter` from the Python original: re-encode an audio file at a given
//! sample rate / channel count / codec.

use crate::error::{AudioHelperError, Result};
use crate::probe::is_valid_audio_file;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ConvertOptions {
    pub sample_rate: u32,
    pub channels: u32,
    /// PCM codec applied when the *output* container is WAV (e.g. `"pcm_s16le"`,
    /// `"pcm_f32le"`). Ignored for compressed containers (mp3, ...), where ffmpeg's
    /// own default codec for that container is used instead — forcing a raw PCM
    /// codec onto a compressed container is a user error, not something to paper
    /// over silently.
    pub encoding: Option<String>,
    /// If `false` and a valid output file already exists, skip re-converting and
    /// return immediately (mirrors `_overwrite_audio_file` in the Python original).
    pub overwrite: bool,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            sample_rate: 44_100,
            channels: 1,
            encoding: Some("pcm_s16le".to_string()),
            overwrite: true,
        }
    }
}

/// Converts `input` to `output`, re-encoding at `options.sample_rate` /
/// `options.channels` (and `options.encoding` for a WAV output).
pub fn convert(input: &Path, output: &Path, options: &ConvertOptions) -> Result<()> {
    if !input.is_file() {
        return Err(AudioHelperError::FileNotFound(input.to_path_buf()));
    }
    if !is_valid_audio_file(input) {
        return Err(AudioHelperError::InvalidAudioFile(input.to_path_buf()));
    }
    if !options.overwrite && output.is_file() && is_valid_audio_file(output) {
        return Ok(());
    }

    let is_wav_out = output
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("wav"))
        .unwrap_or(false);

    let mut args: Vec<std::ffi::OsString> = vec!["-i".into(), input.into()];
    args.push("-ar".into());
    args.push(options.sample_rate.to_string().into());
    args.push("-ac".into());
    args.push(options.channels.to_string().into());
    if is_wav_out {
        if let Some(codec) = &options.encoding {
            args.push("-acodec".into());
            args.push(codec.into());
        }
    }
    args.push(output.into());

    crate::ffmpeg::run(args)?;

    if !output.is_file() || !is_valid_audio_file(output) {
        return Err(AudioHelperError::Ffmpeg(format!(
            "conversion produced no valid audio file at {}",
            output.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::write_test_wav;

    #[test]
    fn converts_sample_rate_and_channels() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.wav");
        write_test_wav(&input, 44_100, 1.0);

        let output = dir.path().join("out.wav");
        let options = ConvertOptions {
            sample_rate: 16_000,
            channels: 1,
            ..Default::default()
        };
        convert(&input, &output, &options).unwrap();

        assert!(output.is_file());
        let (sr, _ch) = crate::probe::native_format(&output).unwrap();
        assert_eq!(sr, 16_000);
    }

    #[test]
    fn skips_conversion_when_overwrite_is_false_and_output_already_valid() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.wav");
        write_test_wav(&input, 44_100, 1.0);
        let output = dir.path().join("out.wav");
        write_test_wav(&output, 8_000, 1.0); // pre-existing, different rate

        let options = ConvertOptions {
            sample_rate: 16_000,
            overwrite: false,
            ..Default::default()
        };
        convert(&input, &output, &options).unwrap();

        // Untouched: still 8kHz, not re-converted to 16kHz.
        let (sr, _) = crate::probe::native_format(&output).unwrap();
        assert_eq!(sr, 8_000);
    }

    #[test]
    fn missing_input_errors() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("missing.wav");
        let output = dir.path().join("out.wav");
        let err = convert(&input, &output, &ConvertOptions::default()).unwrap_err();
        assert!(matches!(err, AudioHelperError::FileNotFound(_)));
    }
}
