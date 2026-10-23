use std::path::PathBuf;
use thiserror::Error;

/// Every fallible operation in this crate returns this type.
#[derive(Debug, Error)]
pub enum AudioHelperError {
    /// The path does not exist, or is not a file.
    #[error("audio file not found: {0}")]
    FileNotFound(PathBuf),

    /// The file exists but carries no audio stream: a video without sound, a
    /// text file with an audio extension, a truncated download.
    #[error("not a valid audio file (no audio stream found): {0}")]
    InvalidAudioFile(PathBuf),

    /// `ffmpeg` ran but exited non-zero. Carries its stderr verbatim, because
    /// that is where the actual reason lives.
    #[error("ffmpeg failed: {0}")]
    Ffmpeg(String),

    /// `ffprobe` ran but exited non-zero. Carries its stderr verbatim.
    #[error("ffprobe failed: {0}")]
    Ffprobe(String),

    /// `ffprobe` succeeded but its JSON did not hold what was expected. Kept
    /// distinct from [`AudioHelperError::Ffprobe`]: a parse failure points at
    /// this crate or at an ffprobe version change, not at the input file.
    #[error("ffprobe output could not be parsed: {0}")]
    FfprobeParse(String),

    /// `ffmpeg` or `ffprobe` could not be spawned at all. Almost always means
    /// the tool is not installed, or not on `PATH` for this process, which is
    /// a setup problem rather than an input problem.
    #[error("failed to spawn `{binary}` — is it installed and on PATH? ({source})")]
    MissingBinary {
        /// The executable that could not be spawned, `ffmpeg` or `ffprobe`.
        binary: &'static str,
        /// The spawn error itself, usually "no such file or directory".
        #[source]
        source: std::io::Error,
    },

    /// A requested range does not fit the file: negative, inverted, or past
    /// the end. All three bounds are reported so the caller can see which one
    /// is wrong without probing again.
    #[error("invalid time range: start={start}, end={end}, duration={duration}")]
    InvalidTimeRange {
        /// Requested start, in seconds.
        start: f64,
        /// Requested end, in seconds.
        end: f64,
        /// The file's actual duration, in seconds.
        duration: f64,
    },

    /// A duration of zero or less was asked for, which no operation here can
    /// honour.
    #[error("duration must be strictly positive, got {0}")]
    InvalidDuration(f64),

    /// Concatenation was called with no input at all.
    #[error("at least one audio file is required")]
    EmptyInputList,

    /// The destination is also one of the sources. ffmpeg would read and write
    /// the same path at once, and the input is gone either way. Refused rather
    /// than attempted: there is no reading of this command under which
    /// destroying one of the inputs is what the caller wanted.
    #[error("{0} is both an input and the output; refusing to overwrite a source file")]
    OutputIsInput(PathBuf),

    /// The buffer handed to [`crate::pcm::save`] cannot be interpreted as
    /// interleaved audio: zero channels, zero sample rate, more channels than a
    /// WAV header can express, or a sample count that is not a whole number of
    /// frames. Rejected rather than written, because every one of those produces
    /// a file that plays as noise instead of failing.
    #[error(
        "invalid PCM layout: {samples} samples across {channels} channel(s) at {sample_rate} Hz"
    )]
    InvalidPcmLayout {
        /// How many samples the buffer holds.
        samples: usize,
        /// How many channels it claims they are interleaved across.
        channels: u32,
        /// The sample rate it claims.
        sample_rate: u32,
    },

    #[error(
        "unsupported noise color: {0:?} (expected one of white/pink/brown/red/blue/violet/velvet)"
    )]
    /// Room tone was asked for a noise colour ffmpeg's `anoisesrc` does not
    /// know. Rejected up front rather than passed through, so the failure
    /// names the valid set instead of surfacing as an ffmpeg error.
    UnsupportedNoiseColor(String),

    /// An underlying I/O error, passed through unchanged.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// A WAV reading or writing error from `hound`, passed through unchanged.
    #[error(transparent)]
    Wav(#[from] hound::Error),
}

/// This crate's `Result`: one error type throughout, so `?` composes across
/// modules without per-call conversion.
pub type Result<T> = std::result::Result<T, AudioHelperError>;
