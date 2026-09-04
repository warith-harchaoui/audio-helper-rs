//! Shared `ffmpeg` subprocess plumbing used by every transform in this crate.

use crate::error::{AudioHelperError, Result};
use std::ffi::OsStr;
use std::process::Command;

/// Runs `ffmpeg -y -hide_banner -loglevel error <args>`, returning `Err` with
/// ffmpeg's stderr on a non-zero exit.
pub(crate) fn run<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("ffmpeg")
        .args(["-y", "-hide_banner", "-loglevel", "error"])
        .args(args)
        .output()
        .map_err(|e| AudioHelperError::MissingBinary {
            binary: "ffmpeg",
            source: e,
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(AudioHelperError::Ffmpeg(stderr));
    }
    Ok(())
}

/// Decodes `source` to mono/stereo `f32` PCM at `sample_rate`, read straight off
/// ffmpeg's stdout — no intermediate file. Shared by `pcm::load` and
/// `silence`'s "confirm truly silent" self-check.
pub(crate) fn decode_to_pcm_f32(
    source: &std::path::Path,
    sample_rate: u32,
    channels: u32,
) -> Result<Vec<f32>> {
    use std::process::Stdio;

    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(source)
        .args([
            "-f",
            "f32le",
            "-ac",
            &channels.to_string(),
            "-ar",
            &sample_rate.to_string(),
            "pipe:1",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| AudioHelperError::MissingBinary {
            binary: "ffmpeg",
            source: e,
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(AudioHelperError::Ffmpeg(stderr));
    }

    Ok(output
        .stdout
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect())
}
