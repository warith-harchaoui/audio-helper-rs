//! `extract_audio_chunk` from the Python original: cut a `[start, end)` slice out
//! of an audio file.

use crate::error::{AudioHelperError, Result};
use crate::probe::{get_audio_duration, is_valid_audio_file};
use std::path::Path;

/// Extracts the `[start, end)` slice of `input` (seconds) into `output`.
pub fn extract_chunk(input: &Path, start: f64, end: f64, output: &Path) -> Result<()> {
    if !input.is_file() {
        return Err(AudioHelperError::FileNotFound(input.to_path_buf()));
    }
    if !is_valid_audio_file(input) {
        return Err(AudioHelperError::InvalidAudioFile(input.to_path_buf()));
    }

    let duration = get_audio_duration(input)?;
    if !(start >= 0.0 && start < duration && end > start && end <= duration) {
        return Err(AudioHelperError::InvalidTimeRange {
            start,
            end,
            duration,
        });
    }

    // `-ss` before `-i` seeks the demuxer directly (fast, frame-accurate enough
    // for this crate's promise); `-t` is the slice *duration*, not an end time.
    crate::ffmpeg::run([
        "-ss".into(),
        start.to_string(),
        "-i".into(),
        input.to_string_lossy().into_owned(),
        "-t".into(),
        (end - start).to_string(),
        output.to_string_lossy().into_owned(),
    ])?;

    if !output.is_file() || !is_valid_audio_file(output) {
        return Err(AudioHelperError::Ffmpeg(format!(
            "chunk extraction produced no valid audio file at {}",
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
    fn extracts_a_valid_range() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.wav");
        write_test_wav(&input, 16_000, 2.0);
        let output = dir.path().join("chunk.wav");

        extract_chunk(&input, 0.5, 1.5, &output).unwrap();

        let chunk_duration = get_audio_duration(&output).unwrap();
        assert!((chunk_duration - 1.0).abs() < 0.05, "got {chunk_duration}");
    }

    #[test]
    fn rejects_out_of_range_end() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.wav");
        write_test_wav(&input, 16_000, 1.0);
        let output = dir.path().join("chunk.wav");

        let err = extract_chunk(&input, 0.0, 5.0, &output).unwrap_err();
        assert!(matches!(err, AudioHelperError::InvalidTimeRange { .. }));
    }

    #[test]
    fn rejects_end_before_start() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.wav");
        write_test_wav(&input, 16_000, 1.0);
        let output = dir.path().join("chunk.wav");

        let err = extract_chunk(&input, 0.5, 0.2, &output).unwrap_err();
        assert!(matches!(err, AudioHelperError::InvalidTimeRange { .. }));
    }
}
