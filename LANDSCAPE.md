# Landscape

[🇫🇷 PAYSAGE.md](https://github.com/warith-harchaoui/audio-helper-rs/blob/master/PAYSAGE.md) · 🇬🇧 English

Related and competing crates in the "file-level audio wrangling in Rust" space, benchmarked against `audio-helper-rs`.
Ratings are ⭐ (1) to ⭐⭐⭐⭐⭐ (5), scored on `audio-helper-rs`'s intended job:
everyday file-level audio handling for Rust pipelines (probe, convert, load and save PCM, cut, concatenate, generate silence, mix room tone, split into fixed chunks). A crate optimised for a very different job is not penalised; the score
just reflects fit to *this* niche.

## At a glance

<!-- TABLE:START -->
| Rust Audio Wrangling | Multi-format decode | Encode / conversion | Split / concat / silence | Room tone | In-process (no subprocess) | Typed errors | Light dependency tree | Pipeline ergonomics |
| --- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **audio-helper-rs** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| symphonia | ⭐⭐⭐⭐⭐ | ⭐ | ⭐⭐ | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| rodio | ⭐⭐⭐⭐ | ⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| hound | ⭐ | ⭐⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| ffmpeg-next | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐ | ⭐⭐ |
| rsmpeg | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐ | ⭐ |
| video-rs | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
<!-- TABLE:END -->

## Positioning map

<!-- FIGURE:START -->
2D representation of the table above.

![Positioning map](https://raw.githubusercontent.com/warith-harchaoui/audio-helper-rs/master/assets/landscape.png)

The map is a 2-D summary of the eight criteria, so read it as a shape, not a scoreboard. `audio-helper-rs` is at the top-right corner. The axes read **Horizontal: Agility ↔ Precision** and **Vertical: Conciseness ↔ Versatility**.

The vertical axis is driven by how much of the file-level job a crate actually does — splitting, concatenating, encoding and room tone all load positively toward the top — and the horizontal axis by how light and well-typed it is to call, where a light dependency tree and typed errors carry the most weight. That produces three groups rather than a ranking.

The top band holds the only three crates that both decode *and* encode: `audio-helper-rs`, `ffmpeg-next` and `rsmpeg`. They split left to right on cost of entry — the two binding crates link the FFmpeg C libraries and need the headers and a C toolchain at build time, where this crate needs four pure-Rust crates and the binaries at run time.

The bottom right is the pure-Rust reading stack: `symphonia`, `video-rs`, `rodio` and, at the extreme, `hound`. They are light and well-typed, which is why they sit right, and they decode rather than transform, which is why they sit low. `symphonia` is the one to reach for when a subprocess is unacceptable; it simply cannot encode, and encoding is half of this crate's job.
<!-- FIGURE:END -->

## Where the claims come from

Versions and publication dates read from the crates.io API on 29 October 2026:

| Crate | Version | Last published |
|---|---|---|
| `symphonia` | 0.6.1 | 2026-08-13 |
| `rodio` | 0.23.0 | 2026-10-11 |
| `hound` | 3.5.1 | 2023-09-25 |
| `ffmpeg-next` | 9.0.0 | 2026-08-05 |
| `rsmpeg` | 0.18.0+ffmpeg.8.0 | 2025-08-24 |
| `video-rs` | 0.12.0 | 2026-09-09 |

`audio-helper-rs`'s own ratings are from this repository at version 0.3.0.

- **The `In-process (no subprocess)` column is where this crate scores worst, and that is the design.** `audio-helper-rs` drives the `ffmpeg` and `ffprobe` binaries through `std::process::Command`; only WAV reading and silence generation stay in-process, through `hound`. `symphonia` decodes in-process and is the right reach when a subprocess is unacceptable — it just cannot encode, which is half of this crate's job.

- **`ffmpeg-next` and `rsmpeg` score 1 on the dependency tree** because both link the FFmpeg C libraries: they need the headers and a C toolchain at build time, where this crate needs four pure-Rust crates (`thiserror`, `serde_json`, `hound`, and `clap` only behind the opt-in `cli` feature) and the binaries at run time.

- **`hound` scores 1 on decoding** because it reads WAV and nothing else — which is exactly why this crate depends on it rather than competing with it.

Deliberately left as opinion rather than sourced: the *pipeline ergonomics*
style columns, and the ordering of the rows below the reference.

## Ideas worth stealing

Researched rather than guessed: what each neighbour does better, and what this crate would have to change to match it. Nothing here is a commitment — it is the reading list the next version starts from.

- **`symphonia` — an in-process decode path, behind a feature** — This crate's worst column is `In-process (no subprocess)`, where it scores 2. `symphonia` decodes MP3, FLAC, AAC, ALAC, Vorbis and WAV in 100% safe Rust with no subprocess at all. Adding it behind an optional feature would give `load()` to a caller who cannot spawn a process, without making anyone else pay for it.

- **`rodio` — optional format-specific decoders rather than one hard dependency** — `rodio` handles playback through `cpal`, decoding through `symphonia` by default, and WAV through `hound` — each swappable, each behind a feature. This crate already gates `clap` that way for the CLI; the same shape fits a decode backend.

- **`video-rs` — a documented ten-line path to the first frame** — Its selling point is that you get usable data in ten lines. The equivalent here is `load()`, and it is worth making that the README's opening example rather than a function buried in a scope table.
