# Changelog

All notable changes to `audio-helper-rs` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/), and the project adheres to
[Semantic Versioning](https://semver.org/).

## [0.2.0] — 2026-10-02

The missing half of the round trip, and a test suite that no longer lies about its
environment. Minor rather than patch because of the new error variant below: under
Cargo's 0.x rules that is the only way to signal it.

### Added
- **`pcm::save(&PcmBuffer, path)`** — write interleaved PCM back to a file, the inverse of
  `pcm::load`. A `.wav` output is written directly with `hound` as 16-bit PCM and needs no
  `ffmpeg` at all; any other extension goes through a temporary WAV and `convert`, so the
  container and codec follow the extension like everywhere else in this crate. Samples outside
  `-1.0..=1.0` are clamped, not wrapped — an overshoot is a mixing mistake, and wrapping turns
  it into a click that is far harder to trace than clipping. The README used to tell you to do
  this yourself; it no longer has to.
- `AudioHelperError::InvalidPcmLayout`, raised when a buffer cannot describe interleaved audio
  (zero channels, zero sample rate, or a sample count that is not whole frames). Each of those
  would otherwise produce a file that plays as noise instead of failing.

### Changed
- **The 15 tests that shell out to `ffmpeg`/`ffprobe` now skip, with a printed reason, when
  those binaries are absent — except under CI, where a missing `ffmpeg` fails loudly.** On a
  machine without `ffmpeg` the suite used to report fifteen `MissingBinary` panics that read
  like a broken crate. The `CI` check is what keeps the escape hatch from hiding a workflow
  that forgot to install `ffmpeg`: there, a skip is a failure.
- **CI is one Linux job instead of a three-OS matrix**, and it installs `ffmpeg` — which is
  why every job of that matrix was red: no runner image ships it. The heavy, cross-platform
  testing is the local pre-push gate's job; CI confirms, it does not discover.
- `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` replace `#![warn(missing_docs)]`, so
  both hold locally and not only under CI's `-D warnings`.
- `rust-version = "1.88"` is now declared. Verified, not guessed: the full suite passes on
  1.88.0, and `slice::as_chunks` in the PCM decoder is what sets the floor.

## [0.1.1] — 2026-09-05

### Changed
- Repo hygiene only, no library change. Added `scripts/check-fresh-resolve.sh` — it builds the
  crate with no `Cargo.lock`, the way a downstream consumer resolves it — and wired it into CI
  and a versioned `.githooks` pre-push gate.
- Every public item documented, so docs.rs carries the whole API.

### Fixed
- The pre-push gate never actually ran: a global `core.hooksPath` (installed by git-lfs)
  silently shadowed `.git/hooks`. The gate now lives in a versioned `.githooks/` that
  `scripts/install-hooks.sh` points `core.hooksPath` at.

## [0.1.0] — 2026-09-05

First release. File-level audio utilities on top of the system `ffmpeg`/`ffprobe` binaries:
`is_valid_audio_file`, `get_audio_duration`, `convert`, `load`, `extract_chunk`,
`generate_silence`, `concatenate`, `mix_room_tone`, `split_regularly`. Rust rewrite of the
Python [`audio-helper`](https://github.com/warith-harchaoui/audio-helper); Demucs source
separation and MFCC similarity are deliberately not ported (see the README).
