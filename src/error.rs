use std::path::PathBuf;
use thiserror::Error;

/// Every fallible operation in this crate returns this type.
#[derive(Debug, Error)]
pub enum AudioHelperError {
    #[error("audio file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("not a valid audio file (no audio stream found): {0}")]
    InvalidAudioFile(PathBuf),

    #[error("ffmpeg failed: {0}")]
    Ffmpeg(String),

    #[error("ffprobe failed: {0}")]
    Ffprobe(String),

    #[error("ffprobe output could not be parsed: {0}")]
    FfprobeParse(String),

    #[error("failed to spawn `{binary}` — is it installed and on PATH? ({source})")]
    MissingBinary {
        binary: &'static str,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid time range: start={start}, end={end}, duration={duration}")]
    InvalidTimeRange { start: f64, end: f64, duration: f64 },

    #[error("duration must be strictly positive, got {0}")]
    InvalidDuration(f64),

    #[error("at least one audio file is required")]
    EmptyInputList,

    #[error(
        "unsupported noise color: {0:?} (expected one of white/pink/brown/red/blue/violet/velvet)"
    )]
    UnsupportedNoiseColor(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Wav(#[from] hound::Error),
}

pub type Result<T> = std::result::Result<T, AudioHelperError>;
