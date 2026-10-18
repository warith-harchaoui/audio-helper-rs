# Surfaces — audio-helper-rs

The Python suite's rule is that a helper exposes five surfaces: a library, two
CLIs, a FastAPI HTTP server, an MCP tool set mounted on that same app, and an
installable agent skill. This crate carries three, and the reasoning for the
two it skips matters more than the gap.

## What exists

**Library** — `cargo add audio-helper-rs`. The primary surface, and the only
one that reaches the in-memory PCM path (`load`, `save`, `PcmBuffer`).

**CLI** — `cargo install audio-helper-rs` gives the `audio-helper-rs` binary.
Thin by design: parse arguments, call one library function, print, exit.

**Agent skill** — this directory.

## What does not exist, and why

**No HTTP API.** The Python twin already serves one over a strictly larger
feature set. A second HTTP surface with fewer routes would be another thing to
keep in sync, for no gain.

**No MCP.** The suite's rule is that MCP is *never* a second implementation:
`fastapi-mcp` mounts `/mcp` onto the FastAPI app the helper already serves, so
every MCP tool **is** one of its routes and the two cannot drift apart. Rust
has no equivalent of that mounting, so an MCP surface here would necessarily be
a separate implementation of the same tools — exactly the drift the rule exists
to prevent. Agents reach this crate through the skill above, and the Python
twin's `/mcp` when they need the full feature set.

**No GUI.** Nothing here is visual.

**Only one CLI, not two.** The Python side ships an argparse CLI and a click
twin because both argument parsers are idiomatic in that ecosystem. Rust has
one idiomatic answer, `clap`, so a second binary would be duplication with
nothing to learn from it.

## Choosing between the two implementations

| Need | Reach for |
|---|---|
| Rust calling code, one static binary | `audio-helper-rs` |
| PCM in memory as typed `f32` samples | `audio-helper-rs` |
| Stem separation (Demucs), karaoke, acapella | Python `audio-helper` |
| MFCC resemblance between two takes | Python `audio-helper` |
| HTTP API, MCP tools, browser GUI | Python `audio-helper` |

Both install side by side without colliding.
