//! `audio_concatenation` from the Python original: join multiple audio files
//! head-to-tail via ffmpeg's `concat` filter (decodes each input first, so it
//! works across mismatched codecs/containers — unlike the `concat` demuxer, which
//! needs matching codecs).

use crate::error::{AudioHelperError, Result};
use crate::probe::is_valid_audio_file;
use std::path::Path;

/// Concatenates `inputs` in order into `output`.
pub fn concat(inputs: &[impl AsRef<Path>], output: &Path) -> Result<()> {
    if inputs.is_empty() {
        return Err(AudioHelperError::EmptyInputList);
    }
    for input in inputs {
        let path = input.as_ref();
        if !path.is_file() {
            return Err(AudioHelperError::FileNotFound(path.to_path_buf()));
        }
        if !is_valid_audio_file(path) {
            return Err(AudioHelperError::InvalidAudioFile(path.to_path_buf()));
        }
    }

    let mut args: Vec<std::ffi::OsString> = Vec::new();
    for input in inputs {
        args.push("-i".into());
        args.push(input.as_ref().into());
    }

    let refs: String = (0..inputs.len()).map(|i| format!("[{i}:a]")).collect();
    let filter = format!("{refs}concat=n={}:v=0:a=1[out]", inputs.len());
    args.push("-filter_complex".into());
    args.push(filter.into());
    args.push("-map".into());
    args.push("[out]".into());
    args.push(output.into());

    crate::ffmpeg::run(args)?;

    if !output.is_file() || !is_valid_audio_file(output) {
        return Err(AudioHelperError::Ffmpeg(format!(
            "concatenation produced no valid audio file at {}",
            output.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::get_audio_duration;
    use crate::testutil::write_test_wav;

    #[test]
    fn concatenates_two_files_and_sums_duration() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.wav");
        let b = dir.path().join("b.wav");
        write_test_wav(&a, 16_000, 1.0);
        write_test_wav(&b, 16_000, 0.5);
        let output = dir.path().join("concat.wav");

        concat(&[&a, &b], &output).unwrap();

        let duration = get_audio_duration(&output).unwrap();
        assert!((duration - 1.5).abs() < 0.05, "got {duration}");
    }

    #[test]
    fn rejects_empty_input_list() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("concat.wav");
        let empty: Vec<std::path::PathBuf> = vec![];
        let err = concat(&empty, &output).unwrap_err();
        assert!(matches!(err, AudioHelperError::EmptyInputList));
    }
}
