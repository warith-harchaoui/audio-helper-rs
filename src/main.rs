//! `audio-helper-rs` — the command-line front end for [`audio_helper_rs`].
//!
//! Thin by design, like the rest of this suite's binaries: parse arguments,
//! call one library function, print one line, map the outcome to an exit code.
//! Every piece of real work stays in the crate.
//!
//! The binary is named after the crate rather than after the Python twin's
//! `audio-helper` command on purpose. The two are meant to coexist, and giving
//! both the same name would mean whichever directory comes first on `PATH`
//! silently wins, with nothing to tell the user which one just ran.

use std::path::PathBuf;
use std::process::ExitCode;

use audio_helper_rs::{
    concatenate, convert, extract_chunk, generate_silence, get_audio_duration, is_valid_audio_file,
    mix_room_tone, split_regularly, ConvertOptions, RoomToneOptions,
};
use clap::{Parser, Subcommand};

/// Local, ffmpeg-backed audio file work: probe, convert, cut, join, split.
#[derive(Parser, Debug)]
#[command(name = "audio-helper-rs", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Report whether a file is readable audio, and how long it is, as JSON.
    Probe {
        /// The audio file, or a video file whose audio track should be read.
        input: PathBuf,
    },
    /// Re-encode to another sample rate, channel count or encoding.
    Convert {
        input: PathBuf,
        output: PathBuf,
        /// Target sample rate in Hz.
        #[arg(long, default_value_t = 44_100)]
        sample_rate: u32,
        /// Target channel count: 1 is mono, 2 is stereo.
        #[arg(long, default_value_t = 1)]
        channels: u32,
        /// ffmpeg encoder name, for WAV output (e.g. `pcm_s16le`).
        #[arg(long)]
        encoding: Option<String>,
        /// Fail instead of overwriting an existing output file.
        #[arg(long)]
        no_overwrite: bool,
    },
    /// Extract the `[start, end]` slice of a file, in seconds.
    Chunk {
        input: PathBuf,
        output: PathBuf,
        /// Start offset in seconds.
        #[arg(long)]
        start: f64,
        /// End offset in seconds.
        #[arg(long)]
        end: f64,
    },
    /// Concatenate several files, in the order given, into one.
    Concat {
        /// Two or more input files.
        #[arg(required = true, num_args = 2..)]
        inputs: Vec<PathBuf>,
        /// Destination file. A flag rather than a positional: the inputs are
        /// variadic, so an output positional cannot sit last among them, and
        /// putting it first made `concat a.wav b.wav c.wav` -- the order every
        /// other subcommand here uses -- silently overwrite `a.wav`.
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Cut a file into fixed-duration pieces.
    Split {
        input: PathBuf,
        /// Directory to write the pieces into; created if missing.
        #[arg(short, long, default_value = ".")]
        out_dir: PathBuf,
        /// Length of each piece, in seconds.
        #[arg(long)]
        seconds: f64,
        /// Extension for the pieces, without the dot.
        #[arg(long, default_value = "wav")]
        ext: String,
        /// Text inserted between the stem and the index.
        #[arg(long, default_value = "_part")]
        suffix: String,
    },
    /// Generate a file of pure silence.
    Silence {
        output: PathBuf,
        /// Duration in seconds.
        #[arg(long)]
        seconds: f64,
        #[arg(long, default_value_t = 44_100)]
        sample_rate: u32,
    },
    /// Mix ambient room tone underneath a recording.
    Roomtone {
        input: PathBuf,
        output: PathBuf,
        /// Noise level in dBFS. Quieter is more negative.
        #[arg(long, default_value_t = -42.0)]
        noise_db: f64,
        /// Noise colour: `pink`, `brown`, `white`.
        #[arg(long, default_value = "pink")]
        color: String,
        #[arg(long, default_value_t = 44_100)]
        sample_rate: u32,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Each arm returns the single line printed on success. Errors are rendered
    // the same way everywhere, so the match yields a Result instead of
    // printing and exiting inside each branch.
    let outcome: Result<String, String> = match cli.command {
        Command::Probe { input } => {
            // Validity is a plain bool, so an unreadable file is reported as
            // `"valid": false` rather than as an error: asking the question is
            // not itself a failure.
            let valid = is_valid_audio_file(&input);
            let duration = if valid {
                get_audio_duration(&input).ok()
            } else {
                None
            };
            Ok(match duration {
                Some(seconds) => format!("{{\"valid\":{valid},\"duration\":{seconds}}}"),
                None => format!("{{\"valid\":{valid},\"duration\":null}}"),
            })
        }
        Command::Convert {
            input,
            output,
            sample_rate,
            channels,
            encoding,
            no_overwrite,
        } => {
            let options = ConvertOptions {
                sample_rate,
                channels,
                // None leaves the encoder to ffmpeg's own default for the
                // container, which is what callers want for anything but WAV.
                encoding: encoding.or_else(|| Some("pcm_s16le".to_string())),
                overwrite: !no_overwrite,
            };
            convert(&input, &output, &options)
                .map(|()| output.display().to_string())
                .map_err(|e| e.to_string())
        }
        Command::Chunk {
            input,
            output,
            start,
            end,
        } => extract_chunk(&input, start, end, &output)
            .map(|()| output.display().to_string())
            .map_err(|e| e.to_string()),
        Command::Concat { output, inputs } => concatenate(&inputs, &output)
            .map(|()| output.display().to_string())
            .map_err(|e| e.to_string()),
        Command::Split {
            input,
            out_dir,
            seconds,
            ext,
            suffix,
        } => split_regularly(&input, &out_dir, seconds, &ext, &suffix)
            .map(|paths| {
                // One line on stdout, still: the pieces are space-free by
                // construction only if the stem is, so they are newline-joined
                // and the count is what a script usually wants.
                paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .map_err(|e| e.to_string()),
        Command::Silence {
            output,
            seconds,
            sample_rate,
        } => generate_silence(seconds, &output, sample_rate)
            .map(|()| output.display().to_string())
            .map_err(|e| e.to_string()),
        Command::Roomtone {
            input,
            output,
            noise_db,
            color,
            sample_rate,
        } => {
            let options = RoomToneOptions {
                noise_db,
                color,
                sample_rate,
            };
            mix_room_tone(&input, &output, &options)
                .map(|()| output.display().to_string())
                .map_err(|e| e.to_string())
        }
    };

    match outcome {
        Ok(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("audio-helper-rs: {message}");
            ExitCode::FAILURE
        }
    }
}
