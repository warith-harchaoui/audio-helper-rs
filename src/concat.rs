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
    // The destination must not be one of the sources. This is the one operation
    // here that takes a variable number of inputs, so the output cannot sit last
    // among the positionals, and anyone carrying over the `<input> <output>`
    // order of `chunk`, `convert` and `roomtone` hands the first *input* to the
    // output slot. ffmpeg then happily truncates it and reports success.
    //
    // Compared canonically where the filesystem allows it: `a.wav` and `./a.wav`
    // are the same file, and a path that does not exist yet cannot collide with
    // an input that must exist, so a failure to canonicalise is not a collision.
    let resolved = output.canonicalize().ok();
    for input in inputs {
        let path = input.as_ref();
        if !path.is_file() {
            return Err(AudioHelperError::FileNotFound(path.to_path_buf()));
        }
        if !is_valid_audio_file(path) {
            return Err(AudioHelperError::InvalidAudioFile(path.to_path_buf()));
        }
        let same = match (&resolved, path.canonicalize().ok()) {
            (Some(out), Some(src)) => *out == src,
            _ => path == output,
        };
        if same {
            return Err(AudioHelperError::OutputIsInput(path.to_path_buf()));
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
    use crate::testutil::{require_ffmpeg, write_test_wav};

    #[test]
    fn concatenates_two_files_and_sums_duration() {
        require_ffmpeg!();
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
    fn refuses_to_write_over_one_of_its_inputs() {
        // `concat` is the only subcommand whose inputs are variadic, so its
        // output could not sit last among the positionals like `chunk`,
        // `convert` and `roomtone` put it. It used to sit *first*, and
        // `concat a.wav b.wav c.wav` -- the order the siblings teach -- fed
        // `a.wav` to the output slot. ffmpeg truncated it, wrote b+c into it,
        // and exited 0: a one-second file came back five seconds long holding
        // someone else's audio, with nothing printed to say so.
        //
        // The CLI now takes `-o`, so that particular slip is unreachable. This
        // guards the library call, which anyone can still get wrong, and the
        // `-o a.wav a.wav b.wav` spelling of the same mistake.
        require_ffmpeg!();
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.wav");
        let b = dir.path().join("b.wav");
        write_test_wav(&a, 16_000, 1.0);
        write_test_wav(&b, 16_000, 0.5);

        let err = concat(&[&a, &b], &a).unwrap_err();
        assert!(
            matches!(err, AudioHelperError::OutputIsInput(ref p) if *p == a),
            "got {err:?}"
        );
        // Still one second: refused before ffmpeg ran, not after.
        let duration = get_audio_duration(&a).unwrap();
        assert!(
            (duration - 1.0).abs() < 0.05,
            "input was modified: {duration}"
        );
    }

    #[test]
    fn spots_the_same_file_under_a_different_spelling() {
        // `./a.wav` and `a.wav` are one file. A plain path comparison misses it,
        // which would leave the guard easy to walk around by accident.
        require_ffmpeg!();
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.wav");
        let b = dir.path().join("b.wav");
        write_test_wav(&a, 16_000, 1.0);
        write_test_wav(&b, 16_000, 0.5);
        let indirect = dir.path().join(".").join("a.wav");

        let err = concat(&[&a, &b], &indirect).unwrap_err();
        assert!(
            matches!(err, AudioHelperError::OutputIsInput(_)),
            "got {err:?}"
        );
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
