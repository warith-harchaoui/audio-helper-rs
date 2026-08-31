//! Inspects an audio file via `ffprobe`, without decoding it — mirrors
//! `is_valid_audio_file` / `get_audio_duration` from the Python original.

use crate::error::{AudioHelperError, Result};
use std::path::Path;
use std::process::Command;

/// The parsed `codec_type: "audio"` stream ffprobe reports, when present.
struct AudioStream {
    sample_rate: u32,
    channels: u32,
    duration_seconds: Option<f64>,
}

fn ffprobe_json(path: &Path) -> Result<serde_json::Value> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .map_err(|e| AudioHelperError::MissingBinary {
            binary: "ffprobe",
            source: e,
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(AudioHelperError::Ffprobe(stderr));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|e| AudioHelperError::FfprobeParse(e.to_string()))
}

fn find_audio_stream(probe: &serde_json::Value) -> Option<AudioStream> {
    let streams = probe.get("streams")?.as_array()?;
    let stream = streams
        .iter()
        .find(|s| s.get("codec_type").and_then(|v| v.as_str()) == Some("audio"))?;

    let sample_rate: u32 = stream
        .get("sample_rate")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let channels: u32 = stream.get("channels").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
    // A stream's own `duration` is sometimes absent (e.g. some containers only
    // report it at the `format` level); fall back there.
    let duration_seconds = stream
        .get("duration")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .or_else(|| {
            probe
                .get("format")
                .and_then(|f| f.get("duration"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
        });

    Some(AudioStream {
        sample_rate,
        channels,
        duration_seconds,
    })
}

/// True if `path` exists and ffprobe finds at least one audio stream in it.
pub fn is_valid_audio_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    match ffprobe_json(path) {
        Ok(probe) => find_audio_stream(&probe).is_some(),
        Err(_) => false,
    }
}

/// Duration in seconds of `path`'s audio stream.
pub fn get_audio_duration(path: &Path) -> Result<f64> {
    if !path.is_file() {
        return Err(AudioHelperError::FileNotFound(path.to_path_buf()));
    }
    let probe = ffprobe_json(path)?;
    let stream = find_audio_stream(&probe)
        .ok_or_else(|| AudioHelperError::InvalidAudioFile(path.to_path_buf()))?;
    stream.duration_seconds.ok_or_else(|| {
        AudioHelperError::FfprobeParse(format!("no duration reported for {}", path.display()))
    })
}

/// Native sample rate and channel count of `path`'s audio stream, without decoding.
pub(crate) fn native_format(path: &Path) -> Result<(u32, u32)> {
    let probe = ffprobe_json(path)?;
    let stream = find_audio_stream(&probe)
        .ok_or_else(|| AudioHelperError::InvalidAudioFile(path.to_path_buf()))?;
    Ok((stream.sample_rate, stream.channels))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::write_test_wav;

    #[test]
    fn valid_wav_reports_duration_and_is_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tone.wav");
        write_test_wav(&path, 16_000, 1.0);

        assert!(is_valid_audio_file(&path));
        let duration = get_audio_duration(&path).unwrap();
        assert!((duration - 1.0).abs() < 0.01, "duration was {duration}");
    }

    #[test]
    fn missing_file_is_invalid_and_errors() {
        let path = Path::new("/nonexistent/does-not-exist.wav");
        assert!(!is_valid_audio_file(path));
        assert!(matches!(
            get_audio_duration(path),
            Err(AudioHelperError::FileNotFound(_))
        ));
    }

    #[test]
    fn non_audio_file_is_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("not-audio.txt");
        std::fs::write(&path, b"hello, not audio").unwrap();
        assert!(!is_valid_audio_file(&path));
    }
}
