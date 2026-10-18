# TRIGGERS: audio-helper-rs

The exhaustive catalogue of what `audio-helper-rs` does and what should reach
for it. The Python twin, [`audio-helper`](https://github.com/warith-harchaoui/audio-helper),
has its own catalogue; the two overlap, and the section on choosing between
them is the part that matters.

`audio-helper-rs` transforms audio **files** — and the audio track of video
files — by driving `ffmpeg`. It is local-first. It does **not** transcribe,
synthesize, download, or separate stems.

## The seven operations → how to invoke

| Intent | CLI | Library |
|--------|-----|---------|
| Is this readable audio, and how long? | `audio-helper-rs probe` | `is_valid_audio_file`, `get_audio_duration` |
| Re-encode format / sample rate / channels | `audio-helper-rs convert` | `convert`, `ConvertOptions` |
| Extract a `[start, end]` slice | `audio-helper-rs chunk` | `extract_chunk` |
| Concatenate files | `audio-helper-rs concat` | `concatenate` |
| Cut into fixed-duration pieces | `audio-helper-rs split` | `split_regularly` |
| Generate silence | `audio-helper-rs silence` | `generate_silence` |
| Mix room-tone ambient noise | `audio-helper-rs roomtone` | `mix_room_tone`, `RoomToneOptions` |

Decoding to in-memory PCM (`load`, `save`, `PcmBuffer`, `LoadOptions`) is a
library-only surface: a buffer of `f32` samples is not something a command line
can hand back.

## Natural-language phrasings that should fire

- **Probe**: "how long is this", "is this a valid audio file", "get the
  duration in Rust".
- **Convert**: "convert this mp3 to wav", "make it mono 16 kHz", "re-encode /
  transcode", "resample to 48000", "extract the audio from this mp4".
- **Chunk**: "cut from 3s to 8s", "keep the first minute", "trim / crop".
- **Split**: "split into 30-second chunks", "chop this into pieces".
- **Concat**: "merge / join / stitch these files", "combine into one".
- **Silence**: "generate 5 seconds of silence", "make a blank spacer".
- **Room tone**: "mask the cuts with room tone", "add pink / brown ambient
  hiss", "the edits between takes are too abrupt".
- **PCM**: "decode this to f32 samples", "load the waveform into memory",
  "resample and downmix before feeding a model".
- **The crate itself**: `cargo add audio-helper-rs`, `cargo install
  audio-helper-rs`, "the Rust audio helper", "audio_helper_rs".

## File types it accepts

Whatever `ffmpeg` decodes: `.mp3 .wav .m4a .flac .ogg .opus .aac .aiff` and the
rest, plus video containers (`.mp4 .mkv .mov .webm …`) whose audio track is
decoded on the way in.

## Choosing between the Rust crate and the Python twin

Reach for **`audio-helper-rs`** when the calling code is already Rust, when a
single binary with no Python runtime matters, or when you want PCM in memory as
typed `f32` samples.

Reach for the **Python `audio-helper`** when you need what this crate does not
have: stem separation with Demucs, MFCC resemblance scoring, the HTTP API, the
MCP tool set, or the browser GUI.

Both install side by side. The binary here is deliberately named
`audio-helper-rs` rather than `audio-helper`, so neither shadows the other on
`PATH`.

## When NOT to use audio-helper-rs (SKIP)

- Transcription, captions, subtitles or speech-to-text → `vocal-helper`.
- Text-to-speech, voice cloning, synthesis, music generation.
- Downloading audio from a URL → `youtube-helper-rs` or `podcast-helper-rs`.
- Separating vocals from music, karaoke, acapella → the Python `audio-helper`
  with its Demucs extra.
- Comparing two takes for similarity → the Python `audio-helper`.
- DAW-style editing: EQ, compression, reverb, de-noise, fades, mastering,
  loudness normalization.
- Pure signal-processing maths with no file in play.
- Capturing from a microphone → `capture-helper-rs`.

## See also

- [`README.md`](README.md) / [`LISEZMOI.md`](LISEZMOI.md): features and install.
- [`CHANGELOG.md`](CHANGELOG.md): what changed, version by version.
