//! Rust rewrite of [`audio-helper`](https://github.com/warith-harchaoui/audio-helper)
//! (Python). Same promise, file-level audio utilities on top of `ffmpeg`/`ffprobe` —
//! not a line-by-line port, idiomatic Rust throughout. See the crate README for the
//! full scope and what is deliberately not ported (Demucs source separation, MFCC
//! similarity).
//!
//! ```
//! use audio_helper_rs::{probe::get_audio_duration, silence::generate_silence};
//!
//! let path = std::env::temp_dir().join("audio_helper_rs_doctest.wav");
//! generate_silence(0.1, &path, 16_000).unwrap();
//! assert!(get_audio_duration(&path).unwrap() > 0.0);
//! std::fs::remove_file(&path).ok();
//! ```

pub mod chunk;
pub mod concat;
pub mod convert;
pub mod error;
pub mod pcm;
pub mod probe;
pub mod roomtone;
pub mod silence;
pub mod split;

mod ffmpeg;
#[cfg(test)]
mod testutil;

pub use chunk::extract_chunk;
pub use concat::concat as concatenate;
pub use convert::{convert, ConvertOptions};
pub use error::{AudioHelperError, Result};
pub use pcm::{load, LoadOptions, PcmBuffer};
pub use probe::{get_audio_duration, is_valid_audio_file};
pub use roomtone::{mix_room_tone, RoomToneOptions};
pub use silence::generate_silence;
pub use split::split_regularly;
