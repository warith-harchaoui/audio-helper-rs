---
name: audio-helper-rs
description: >-
  Local, ffmpeg-backed work on audio files from Rust: probe a file for validity and duration, convert its format, sample rate or channel count, extract a start-to-end chunk, concatenate several files, split one into fixed-duration pieces, generate silence, mix pink or brown room tone under a recording, and decode to in-memory f32 PCM. Also reads the audio track of video containers. Use when the user is writing Rust and says "convert this mp3 to wav", "make it mono 16 kHz", "how long is this file", "cut from 3s to 8s", "merge these clips", "split into 30-second chunks", "generate silence", "mask the cuts with room tone", "decode to f32 samples", or names cargo add audio-helper-rs or audio_helper_rs. Do NOT use to transcribe (vocal-helper), to download (youtube-helper-rs, podcast-helper-rs), to record from a microphone (capture-helper-rs), or to separate stems or score similarity -- the Python twin audio-helper covers those.
license: BSD-3-Clause
metadata:
  author: Warith Harchaoui
  contact: warithmetics@deraison.ai
  repository: https://github.com/warith-harchaoui/audio-helper-rs
  crate: https://crates.io/crates/audio-helper-rs
  language: rust
---

# audio-helper-rs

File-level audio work in Rust, on top of `ffmpeg`. Typed options structs, a
`thiserror` enum, and no stringly-typed errors reaching a caller.

## Before anything else: ffmpeg

Every operation but the in-memory PCM path shells out to `ffmpeg`. If it is not
on `PATH` the call fails with a typed error, not a panic — report that plainly,
because the fix is on the user's machine, not in their code.

## The seven operations

| Intent | CLI | Library |
|--------|-----|---------|
| Validity and duration | `probe` | `is_valid_audio_file`, `get_audio_duration` |
| Re-encode | `convert` | `convert`, `ConvertOptions` |
| Slice `[start, end]` | `chunk` | `extract_chunk` |
| Join files | `concat` | `concatenate` |
| Cut into fixed pieces | `split` | `split_regularly` |
| Generate silence | `silence` | `generate_silence` |
| Mix room tone | `roomtone` | `mix_room_tone`, `RoomToneOptions` |

Full flags and the output contract: `references/cli-reference.md`.

## Writing Rust against it

```rust
use audio_helper_rs::{convert, ConvertOptions, load, LoadOptions};

// Re-encode to 16 kHz mono, which is what most speech models want.
let options = ConvertOptions { sample_rate: 16_000, channels: 1, ..Default::default() };
convert(Path::new("in.mp3"), Path::new("out.wav"), &options)?;

// Or skip the file entirely and decode straight to f32 samples.
let pcm = load(Path::new("in.mp3"), &LoadOptions {
    target_sample_rate: Some(16_000), to_mono: true, ..Default::default()
})?;
```

`load` / `save` / `PcmBuffer` are **library-only**: a buffer of samples is not
something a command line can hand back. Reach for them when the next step is a
model or a DSP routine rather than a file.

## Two things that are easy to get wrong

**`probe` never fails on a bad file.** It answers the question it was asked, so
an unreadable path returns `{"valid": false, "duration": null}` and exit `0`.
Check the field; do not check the exit code.

**Room tone is a masking tool, not a denoiser.** It adds quiet ambient noise so
abrupt edits between takes stop being audible. It cannot remove noise.

## Choosing this crate over the Python twin

Pick this one for Rust calling code, a single binary, or typed PCM in memory.
Pick the Python `audio-helper` for Demucs stem separation, MFCC resemblance
scoring, the HTTP API, MCP, or the GUI.

Both install side by side: this binary is `audio-helper-rs`, not
`audio-helper`, so neither shadows the other on `PATH`.

## References

- `references/triggers.md` — the full catalogue, mirroring the root `TRIGGERS.md`.
- `references/cli-reference.md` — every subcommand, flag and exit code.
- `references/surfaces.md` — which surfaces exist here, and which do not.
