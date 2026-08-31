//! `generate_silent_audio` from the Python original: write a silent audio file of
//! a given duration.

use crate::convert::{convert, ConvertOptions};
use crate::error::{AudioHelperError, Result};
use crate::probe::is_valid_audio_file;
use std::path::Path;

/// Writes `duration` seconds of silence to `output`, mono, at `sample_rate`.
pub fn generate_silence(duration: f64, output: &Path, sample_rate: u32) -> Result<()> {
    if duration <= 0.0 {
        return Err(AudioHelperError::InvalidDuration(duration));
    }

    let is_wav = output
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("wav"))
        .unwrap_or(false);

    if is_wav {
        write_silent_wav(output, sample_rate, duration)?;
    } else {
        let tmp = tempfile::Builder::new().suffix(".wav").tempfile()?;
        write_silent_wav(tmp.path(), sample_rate, duration)?;
        convert(
            tmp.path(),
            output,
            &ConvertOptions {
                sample_rate,
                channels: 1,
                encoding: None,
                overwrite: true,
            },
        )?;
    }

    if !output.is_file() || !is_valid_audio_file(output) {
        return Err(AudioHelperError::Ffmpeg(format!(
            "silence generation produced no valid audio file at {}",
            output.display()
        )));
    }

    // Self-check, same assertion the Python original makes: the file really is
    // silent, not just present.
    let pcm = crate::pcm::load(
        output,
        &crate::pcm::LoadOptions {
            target_sample_rate: None,
            to_mono: true,
            two_channels: false,
        },
    )?;
    let sum_abs: f32 = pcm.samples.iter().map(|s| s.abs()).sum();
    if sum_abs != 0.0 {
        return Err(AudioHelperError::Ffmpeg(format!(
            "generated silence at {} is not actually silent (sum(|x|) = {sum_abs})",
            output.display()
        )));
    }

    Ok(())
}

fn write_silent_wav(path: &Path, sample_rate: u32, duration: f64) -> Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    let num_samples = (duration * sample_rate as f64).round() as usize;
    for _ in 0..num_samples {
        writer.write_sample(0i16)?;
    }
    writer.finalize()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_a_wav_of_the_requested_duration() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("silence.wav");

        generate_silence(0.5, &output, 16_000).unwrap();

        let duration = crate::probe::get_audio_duration(&output).unwrap();
        assert!((duration - 0.5).abs() < 0.01, "got {duration}");
    }

    #[test]
    fn generates_a_non_wav_container() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("silence.mp3");

        generate_silence(0.3, &output, 44_100).unwrap();
        assert!(is_valid_audio_file(&output));
    }

    #[test]
    fn rejects_non_positive_duration() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("silence.wav");
        let err = generate_silence(0.0, &output, 16_000).unwrap_err();
        assert!(matches!(err, AudioHelperError::InvalidDuration(_)));
    }
}
