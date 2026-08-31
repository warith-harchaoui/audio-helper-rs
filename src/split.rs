//! `split_audio_regularly` from the Python original: cut an audio file into
//! fixed-duration chunks.

use crate::chunk::extract_chunk;
use crate::error::{AudioHelperError, Result};
use crate::probe::{get_audio_duration, is_valid_audio_file};
use std::path::{Path, PathBuf};

/// Splits `input` into `chunk_seconds`-long pieces (extension `output_ext`,
/// without the dot) inside `output_dir`, named `<stem>_<suffix>_<index>.<ext>`.
///
/// A trailing remainder shorter than 1 second is dropped rather than saved as its
/// own chunk, unless it is the very first (and only) chunk — a `input` shorter
/// than 1 second still yields one chunk covering it in full, instead of silently
/// returning an empty list. Matches the Python original's rule exactly.
pub fn split_regularly(
    input: &Path,
    output_dir: &Path,
    chunk_seconds: f64,
    output_ext: &str,
    suffix: &str,
) -> Result<Vec<PathBuf>> {
    if !input.is_file() {
        return Err(AudioHelperError::FileNotFound(input.to_path_buf()));
    }
    if !is_valid_audio_file(input) {
        return Err(AudioHelperError::InvalidAudioFile(input.to_path_buf()));
    }
    if chunk_seconds <= 0.0 {
        return Err(AudioHelperError::InvalidDuration(chunk_seconds));
    }

    std::fs::create_dir_all(output_dir)?;

    let duration = get_audio_duration(input)?;
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");
    let ext = output_ext.trim_start_matches('.');

    let mut outputs = Vec::new();
    let mut start = 0.0;
    let mut index = 0usize;
    while start < duration {
        let end = (start + chunk_seconds).min(duration);
        let remaining = end - start;
        if index > 0 && remaining < 1.0 {
            break;
        }
        let output_path = output_dir.join(format!("{stem}_{suffix}_{index}.{ext}"));
        extract_chunk(input, start, end, &output_path)?;
        outputs.push(output_path);
        start = end;
        index += 1;
    }

    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::write_test_wav;

    #[test]
    fn splits_into_regular_chunks() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.wav");
        write_test_wav(&input, 16_000, 3.5);
        let out_dir = dir.path().join("chunks");

        let chunks = split_regularly(&input, &out_dir, 1.0, "wav", "split").unwrap();

        // 3.5s / 1.0s chunks: 0-1, 1-2, 2-3, remainder 3-3.5 (0.5s < 1s) -> dropped.
        assert_eq!(chunks.len(), 3);
        for chunk in &chunks {
            assert!(chunk.is_file());
        }
    }

    #[test]
    fn a_file_shorter_than_one_second_still_yields_one_chunk() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("short.wav");
        write_test_wav(&input, 16_000, 0.3);
        let out_dir = dir.path().join("chunks");

        let chunks = split_regularly(&input, &out_dir, 5.0, "wav", "split").unwrap();
        assert_eq!(chunks.len(), 1);
    }
}
