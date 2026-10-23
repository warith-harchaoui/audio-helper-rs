# CLI reference — `audio-helper-rs`

Installed with `cargo install audio-helper-rs`. The binary is named after the
crate, not after the Python twin's `audio-helper`, so both can sit on `PATH`
without either shadowing the other.

## Output contract

- Success: the produced path (or the JSON answer) on **stdout**, exit `0`.
- Failure: one line on **stderr** prefixed `audio-helper-rs: `, exit `1`.

`split` is the one command that prints more than one line, one path per piece.

## `probe`

```
audio-helper-rs probe <INPUT>
```

Prints `{"valid":true,"duration":12.5}`. **Exit code stays `0` even for an
unreadable file** — asking whether a file is valid is not itself a failure, so
the answer is in the `valid` field: `{"valid":false,"duration":null}`. Check
the field, not the exit code.

## `convert`

```
audio-helper-rs convert <INPUT> <OUTPUT>
    [--sample-rate HZ] [--channels N] [--encoding NAME] [--no-overwrite]
```

Defaults: `--sample-rate 44100`, `--channels 1` (mono), `--encoding pcm_s16le`
for WAV output. Overwrites by default; `--no-overwrite` makes an existing
output an error instead. Accepts video containers as input and takes their
audio track.

16 kHz mono (`--sample-rate 16000 --channels 1`) is what most speech models
expect.

## `chunk`

```
audio-helper-rs chunk <INPUT> <OUTPUT> --start SECONDS --end SECONDS
```

Extracts the `[start, end]` slice. Both bounds are in seconds and may be
fractional.

## `concat`

```
audio-helper-rs concat -o <OUTPUT> <INPUT> <INPUT> [INPUT...]
```

Joins in the order given. Two inputs minimum — joining one file is a copy, and
the command refuses rather than pretending to work.

The destination is a flag, not a positional, because the inputs are variadic:
an output positional cannot sit last among them the way `convert` and `chunk`
put it, and putting it first made `concat a.wav b.wav c.wav` overwrite
`a.wav`. Passing a source as the destination is refused outright.

## `split`

```
audio-helper-rs split <INPUT> [-o|--out-dir DIR] --seconds N
    [--ext EXT] [--suffix TEXT]
```

Cuts into fixed-duration pieces and prints one path per line. `--out-dir`
defaults to `.` and is created if missing; `--ext` defaults to `wav`;
`--suffix` defaults to `_part` and is inserted between the stem and the index.

## `silence`

```
audio-helper-rs silence <OUTPUT> --seconds N [--sample-rate HZ]
```

Writes a file of pure silence.

## `roomtone`

```
audio-helper-rs roomtone <INPUT> <OUTPUT>
    [--noise-db DBFS] [--color pink|brown|white] [--sample-rate HZ]
```

Mixes quiet ambient noise under the recording so abrupt edits between takes
stop being audible. Defaults: `-42` dBFS, `pink`. Quieter is more negative.

This masks edits; it does **not** remove noise. There is no denoiser here.

## Not on the command line

`load`, `save`, `PcmBuffer` and `LoadOptions` are library-only: a buffer of
`f32` samples is not something a binary can hand back over stdout. Use the
crate directly when the next step is a model or a DSP routine.
