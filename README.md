# Audio Helper (Rust)

[🇫🇷](https://github.com/warith-harchaoui/audio-helper-rs/blob/master/LISEZMOI.md) · [🇬🇧](https://github.com/warith-harchaoui/audio-helper-rs/blob/master/README.md)

[![crates.io](https://img.shields.io/crates/v/audio-helper-rs.svg)](https://crates.io/crates/audio-helper-rs) [![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](./LICENSE)

Rust rewrite of [`audio-helper`](https://github.com/warith-harchaoui/audio-helper). Same promise, file-level audio utilities on top of `ffmpeg`/`ffprobe` — not a line-by-line port of the Python code, idiomatic Rust throughout.

## v0.1 scope

| Function | What it does |
|---|---|
| `is_valid_audio_file(path) -> bool` | `ffprobe`-based check: does the file contain at least one audio stream? |
| `get_audio_duration(path) -> Result<f64>` | Duration in seconds, via `ffprobe`. |
| `convert(input, output, &ConvertOptions) -> Result<()>` | Re-encode at a given sample rate / channel count / codec. `overwrite: false` skips the work if a valid output already exists. |
| `load(path, &LoadOptions) -> Result<PcmBuffer>` | Decode into memory as `f32` PCM (`Vec<f32>` + sample rate + channels) — mono by default, optional resampling. Works on any `ffmpeg`-readable input, including the audio track of a video container. |
| `extract_chunk(input, start, end, output) -> Result<()>` | Cut a `[start, end)` slice (seconds). |
| `generate_silence(duration, output, sample_rate) -> Result<()>` | Write a silent file of a given duration; self-checks that the result really is silent. |
| `concatenate(inputs, output) -> Result<()>` | Join files head-to-tail via `ffmpeg`'s `concat` filter (works across mismatched codecs, unlike the `concat` demuxer). |
| `mix_room_tone(input, output, &RoomToneOptions) -> Result<()>` | Mix a constant low-level colored-noise bed under a speech track — standard post-production trick so edited-together silent gaps don't sound unnaturally dead. |
| `split_regularly(input, output_dir, chunk_seconds, ext, suffix) -> Result<Vec<PathBuf>>` | Cut a file into fixed-duration chunks. A trailing remainder under 1 second is dropped, unless it is the only chunk. |

```rust
use audio_helper_rs::{get_audio_duration, silence::generate_silence};

let path = std::env::temp_dir().join("example.wav");
generate_silence(1.0, &path, 16_000)?;
println!("{} seconds", get_audio_duration(&path)?);
```

### Not ported

- **`separate_sources`** (Demucs source separation): needs a real ML stack (`torch`/`torchaudio`), disproportionate to this crate's promise. Same call as `youtube-helper-rs` not reimplementing `yt-dlp` itself — a v0.1 that shells out to a Python/Demucs process, rather than a native Rust reimplementation, would be the natural next step if this is ever needed here.
- **`sound_resemblance`** (MFCC-based similarity score): specialized DSP (mel filter banks, MFCC) that belongs in its own crate rather than bolted onto file-level utilities.
- **`load`/`save` round-tripping through `torch.Tensor`/`np.ndarray`**: this crate returns plain `Vec<f32>` — no tensor library dependency for a library crate. Saving a `Vec<f32>` back to a file is just `convert` fed a WAV written via [`hound`](https://crates.io/crates/hound), already a direct dependency.

## Installation

Requires `ffmpeg` and `ffprobe` on `PATH` (macOS: `brew install ffmpeg`).

```toml
[dependencies]
audio-helper-rs = "0.1"
```

## Project status

- `cargo build`: clean, no warnings.
- `cargo test`: **20 unit tests + 1 doctest passing**, 0 failures — every test runs a real `ffmpeg`/`ffprobe` subprocess against a `hound`-generated WAV fixture, nothing mocked.
- `cargo clippy --all-targets`: clean, no warnings.
- **Measured code coverage** (`cargo llvm-cov`): **88.76% line coverage** (623 lines, 70 uncovered), 83.08% function coverage, 89.90% region coverage. Per-file breakdown:

  | File | Line coverage |
  |---|---|
  | `chunk.rs` | 89.47% |
  | `concat.rs` | 86.54% |
  | `convert.rs` | 93.98% |
  | `ffmpeg.rs` | 81.48% (the `MissingBinary` branch needs `ffmpeg` actually absent from `PATH` to exercise, not reproduced in a normal CI box) |
  | `pcm.rs` | 90.91% |
  | `probe.rs` | 82.88% (ffprobe JSON edge cases — a stream reporting no `duration` at all — aren't fixture-able without a corrupt file) |
  | `roomtone.rs` | 90.77% |
  | `silence.rs` | 87.80% |
  | `split.rs` | 94.83% |

  To reproduce:

  ```bash
  # once: LLVM tools (this project uses Homebrew LLVM, not rustup)
  export LLVM_COV=/opt/homebrew/opt/llvm/bin/llvm-cov
  export LLVM_PROFDATA=/opt/homebrew/opt/llvm/bin/llvm-profdata

  cargo llvm-cov --summary-only        # text summary
  cargo llvm-cov --html                # HTML report at target/llvm-cov/html/index.html
  ```

## Related

Part of the same author's local-first tooling as [`audio-helper`](https://github.com/warith-harchaoui/audio-helper) (Python), [`podcast-helper-rs`](https://github.com/warith-harchaoui/podcast-helper-rs), [`capture-helper-rs`](https://github.com/warith-harchaoui/capture-helper-rs), [`youtube-helper-rs`](https://github.com/warith-harchaoui/youtube-helper-rs), and the [AI Helpers](https://github.com/warith-harchaoui/ai-helpers) suite. Independent rewrite, not a binding.

## License

BSD-3-Clause, see [LICENSE](LICENSE).

## Author

[Warith HARCHAOUI](https://linkedin.com/in/warith-harchaoui)
