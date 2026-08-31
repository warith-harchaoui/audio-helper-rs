//! `mix_room_tone` from the Python original: mix a constant low-level ambient
//! noise bed under a speech track, so silent gaps between edited-together takes
//! don't sound unnaturally dead. Standard post-production trick — the noise sits
//! well below the conscious hearing threshold but keeps the room "alive".

use crate::error::{AudioHelperError, Result};
use crate::probe::{get_audio_duration, is_valid_audio_file};
use std::path::Path;

const VALID_COLORS: [&str; 7] = ["white", "pink", "brown", "red", "blue", "violet", "velvet"];

#[derive(Debug, Clone)]
pub struct RoomToneOptions {
    /// Noise level in dB (typical range -45..-38). Amplitude is `10 ** (db / 20)`.
    pub noise_db: f64,
    /// ffmpeg `anoisesrc` color: white/pink/brown/red/blue/violet/velvet.
    pub color: String,
    pub sample_rate: u32,
}

impl Default for RoomToneOptions {
    fn default() -> Self {
        Self {
            noise_db: -42.0,
            color: "pink".to_string(),
            sample_rate: 44_100,
        }
    }
}

/// Mixes ambient noise under `input`, writing the result to `output`.
pub fn mix_room_tone(input: &Path, output: &Path, options: &RoomToneOptions) -> Result<()> {
    if !input.is_file() {
        return Err(AudioHelperError::FileNotFound(input.to_path_buf()));
    }
    if !is_valid_audio_file(input) {
        return Err(AudioHelperError::InvalidAudioFile(input.to_path_buf()));
    }
    if !VALID_COLORS.contains(&options.color.as_str()) {
        return Err(AudioHelperError::UnsupportedNoiseColor(
            options.color.clone(),
        ));
    }

    let duration = get_audio_duration(input)?;
    let amplitude = 10f64.powf(options.noise_db / 20.0);
    // Small overshoot so `amix=duration=first` clamps back to the speech length
    // without truncating its tail by sub-sample rounding.
    let noise_duration = duration + 0.5;

    let noise_source = format!(
        "anoisesrc=color={}:amplitude={:.6}:duration={:.3}:sample_rate={}",
        options.color, amplitude, noise_duration, options.sample_rate
    );

    crate::ffmpeg::run([
        "-i".into(),
        input.to_string_lossy().into_owned(),
        "-f".into(),
        "lavfi".into(),
        "-i".into(),
        noise_source,
        "-filter_complex".into(),
        "[0:a][1:a]amix=inputs=2:duration=first:dropout_transition=0[out]".into(),
        "-map".into(),
        "[out]".into(),
        output.to_string_lossy().into_owned(),
    ])?;

    if !output.is_file() || !is_valid_audio_file(output) {
        return Err(AudioHelperError::Ffmpeg(format!(
            "room-tone mix produced no valid audio file at {}",
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
    fn mixes_and_preserves_roughly_the_input_duration() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("speech.wav");
        write_test_wav(&input, 16_000, 1.0);
        let output = dir.path().join("mixed.wav");

        mix_room_tone(&input, &output, &RoomToneOptions::default()).unwrap();

        let duration = get_audio_duration(&output).unwrap();
        assert!((duration - 1.0).abs() < 0.1, "got {duration}");
    }

    #[test]
    fn rejects_unsupported_color() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("speech.wav");
        write_test_wav(&input, 16_000, 1.0);
        let output = dir.path().join("mixed.wav");

        let options = RoomToneOptions {
            color: "chartreuse".to_string(),
            ..Default::default()
        };
        let err = mix_room_tone(&input, &output, &options).unwrap_err();
        assert!(matches!(err, AudioHelperError::UnsupportedNoiseColor(_)));
    }
}
