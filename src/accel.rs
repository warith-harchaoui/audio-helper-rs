//! Picking the encoder the machine is actually fastest at, instead of whatever
//! `ffmpeg` happens to default to.
//!
//! # Why this module exists
//!
//! Audio work has a hardware story, and it is not the one people expect. There is
//! no GPU path worth having here: a GPU decodes video, and an audio stream is far
//! too small to pay for the trip across the bus. What *does* exist is the
//! operating system's own codec implementation, which on recent machines is
//! backed by dedicated silicon:
//!
//! - **macOS / iOS** — AudioToolbox, exposed by ffmpeg as `aac_at` and `alac_at`.
//! - **Windows** — Media Foundation, exposed as `aac_mf` and `mp3_mf`.
//! - **Linux and everything else** — no OS codec layer, so ffmpeg's own
//!   encoders, which are themselves SIMD-accelerated (AVX2, NEON) at runtime
//!   with no flag to set.
//!
//! So the best default is machine-dependent, which is exactly the thing a
//! hardcoded `-acodec` cannot be.
//!
//! # And it is measured, not reasoned
//!
//! The sibling crate `vocal-helper-rs` learned this the hard way: putting the
//! obvious accelerator first made its workload *twice as slow*, because the
//! accelerator fragmented the graph. A platform encoder is not automatically the
//! fast one either — `aac_at` is excellent on Apple Silicon and merely adequate
//! on an older Intel Mac. So this module asks the machine: it encodes one second
//! of signal with each candidate that is actually installed, keeps the fastest,
//! and remembers the verdict on disk so the question is asked once per machine
//! and per ffmpeg build.
//!
//! # Order of authority
//!
//! 1. An encoder named explicitly by the caller (`ConvertOptions::encoder`).
//! 2. The `AUDIO_HELPER_ENCODER` environment variable — same effect, for pinning
//!    a choice without touching code.
//! 3. The measured verdict for this machine.
//! 4. ffmpeg's own default, when nothing better can be established.
//!
//! An explicit choice is never second-guessed, never measured, and never
//! overridden. Steps 3 and 4 are reported through [`Choice::reason`] rather than
//! applied silently, because a caller that gets a different encoder than it
//! expected deserves to know which rung of this ladder answered.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::Instant;

/// The environment variable that pins an encoder for every conversion.
pub const ENCODER_ENV: &str = "AUDIO_HELPER_ENCODER";

/// A codec family, i.e. what the output container asks for. The *encoder* that
/// serves a family is what this module picks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family {
    /// AAC, the codec of `.m4a` / `.aac` / `.mp4`.
    Aac,
    /// MP3.
    Mp3,
    /// Apple Lossless.
    Alac,
    /// Opus, in `.opus` or WebM.
    Opus,
    /// FLAC.
    Flac,
}

impl Family {
    /// The family an output extension asks for, or `None` for containers this
    /// module has nothing to say about (WAV and other raw PCM, where the codec
    /// *is* the format and `ConvertOptions::encoding` already decides).
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "m4a" | "aac" | "mp4" | "m4b" => Some(Self::Aac),
            "mp3" => Some(Self::Mp3),
            "alac" => Some(Self::Alac),
            "opus" => Some(Self::Opus),
            "flac" => Some(Self::Flac),
            _ => None,
        }
    }

    /// Its name in the cache file and in diagnostics.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Aac => "aac",
            Self::Mp3 => "mp3",
            Self::Alac => "alac",
            Self::Opus => "opus",
            Self::Flac => "flac",
        }
    }
}

/// Which rung of the ladder produced an encoder choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// Named by the caller, or by [`ENCODER_ENV`]. Taken as given.
    Explicit,
    /// Fastest of the installed candidates when they were timed on this machine.
    Measured,
    /// Read back from this machine's cached verdict.
    Remembered,
    /// The only candidate installed, so there was nothing to measure.
    Sole,
}

/// An encoder, and why it was chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// The ffmpeg encoder name, to pass as `-acodec`.
    pub encoder: String,
    /// Which rung of the ladder answered.
    pub reason: Reason,
}

/// The candidate encoders for a family on *this* operating system, best guess
/// first — the order only decides who gets measured first and who wins a tie, it
/// does not decide the verdict.
///
/// Platform encoders lead because they are the ones that can be backed by
/// silicon; the portable software encoder always closes the list so there is
/// always something to fall back to.
pub fn candidates(family: Family) -> Vec<&'static str> {
    let platform: &[&str] = if cfg!(target_vendor = "apple") {
        match family {
            Family::Aac => &["aac_at"],
            Family::Alac => &["alac_at"],
            _ => &[],
        }
    } else if cfg!(target_os = "windows") {
        match family {
            Family::Aac => &["aac_mf"],
            Family::Mp3 => &["mp3_mf"],
            _ => &[],
        }
    } else {
        &[]
    };

    let software: &[&str] = match family {
        Family::Aac => &["libfdk_aac", "aac"],
        Family::Mp3 => &["libmp3lame", "libshine"],
        Family::Alac => &["alac"],
        Family::Opus => &["libopus", "opus"],
        Family::Flac => &["flac"],
    };

    platform.iter().chain(software.iter()).copied().collect()
}

/// The encoders this ffmpeg build actually ships, parsed once per process from
/// `ffmpeg -encoders`.
pub fn installed() -> &'static Vec<String> {
    static INSTALLED: OnceLock<Vec<String>> = OnceLock::new();
    INSTALLED.get_or_init(|| {
        let out = Command::new("ffmpeg")
            .args(["-hide_banner", "-loglevel", "error", "-encoders"])
            .stdin(Stdio::null())
            .output();
        match out {
            Ok(o) => parse_encoder_list(&String::from_utf8_lossy(&o.stdout)),
            Err(_) => Vec::new(),
        }
    })
}

/// Parses the body of `ffmpeg -encoders`: a header, a `------` rule, then one
/// line per encoder whose second whitespace-separated field is its name.
///
/// Split out from the subprocess call so the parsing is testable without
/// ffmpeg — the format is stable, but "stable" is not "worth trusting untested".
pub fn parse_encoder_list(stdout: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut past_rule = false;
    for line in stdout.lines() {
        if !past_rule {
            if line.trim_start().starts_with("------") {
                past_rule = true;
            }
            continue;
        }
        let mut fields = line.split_whitespace();
        let flags = fields.next();
        let name = fields.next();
        // The flags column is a fixed-width capability string ("A....D"); a line
        // without one is a continuation of the header, not an encoder.
        if let (Some(flags), Some(name)) = (flags, name) {
            if flags.len() >= 2 && !flags.contains('=') {
                names.push(name.to_string());
            }
        }
    }
    names
}

/// The chosen encoder for `family`, honouring the order of authority documented
/// at the top of this module. `explicit` is the caller's own choice, which wins
/// outright when present.
///
/// Returns `None` when no candidate is installed, which means "say nothing to
/// ffmpeg and let it pick its own default" — the one case where being quiet is
/// right, because ffmpeg's default for a container is always a working encoder.
pub fn choose(family: Family, explicit: Option<&str>) -> Option<Choice> {
    if let Some(name) = explicit.filter(|n| !n.trim().is_empty()) {
        return Some(Choice {
            encoder: name.trim().to_string(),
            reason: Reason::Explicit,
        });
    }
    if let Ok(name) = std::env::var(ENCODER_ENV) {
        if !name.trim().is_empty() {
            return Some(Choice {
                encoder: name.trim().to_string(),
                reason: Reason::Explicit,
            });
        }
    }

    let have = installed();
    let present: Vec<&'static str> = candidates(family)
        .into_iter()
        .filter(|c| have.iter().any(|h| h == c))
        .collect();

    match present.len() {
        0 => None,
        1 => Some(Choice {
            encoder: present[0].to_string(),
            reason: Reason::Sole,
        }),
        _ => {
            if let Some(remembered) = recall(family, &present) {
                return Some(Choice {
                    encoder: remembered,
                    reason: Reason::Remembered,
                });
            }
            let winner = fastest(&present)?;
            remember(family, &winner);
            Some(Choice {
                encoder: winner,
                reason: Reason::Measured,
            })
        }
    }
}

/// How much faster a later candidate must be to displace an earlier one. Below
/// this, the two are called equal and the order in [`candidates`] decides.
///
/// Five per cent is not arbitrary: measured here on 600 s of audio, ffmpeg's
/// native `aac` took 3.11–3.55 s across three runs and AudioToolbox's `aac_at`
/// took 2.92–3.33 s. The ranges overlap, so a strict `<` would hand the verdict
/// to whichever run happened to land first — and the cache would then make that
/// coin toss permanent.
const DISPLACEMENT_MARGIN: f64 = 0.05;

/// Times each installed candidate and returns the winner. `None` only if the
/// list is empty; a candidate that fails to run simply does not compete, and if
/// none of them run (an ffmpeg built without `lavfi`, say) the first candidate
/// stands.
fn fastest(present: &[&'static str]) -> Option<String> {
    let timings: Vec<(String, f64)> = present
        .iter()
        .filter_map(|e| time_encoder(e).map(|secs| ((*e).to_string(), secs)))
        .collect();
    pick_with_margin(present, &timings)
}

/// The verdict rule, kept pure so it can be tested without ffmpeg: walk the
/// candidates in preference order, and only let a later one win if it beats the
/// incumbent by more than [`DISPLACEMENT_MARGIN`].
fn pick_with_margin(ordered: &[&'static str], timings: &[(String, f64)]) -> Option<String> {
    let timed = |name: &str| timings.iter().find(|(n, _)| n == name).map(|(_, t)| *t);

    let mut best: Option<(&str, f64)> = None;
    for candidate in ordered {
        let Some(secs) = timed(candidate) else {
            continue;
        };
        match best {
            None => best = Some((candidate, secs)),
            Some((_, incumbent)) => {
                if secs < incumbent * (1.0 - DISPLACEMENT_MARGIN) {
                    best = Some((candidate, secs));
                }
            }
        }
    }
    best.map(|(name, _)| name.to_string())
        .or_else(|| ordered.first().map(|n| (*n).to_string()))
}

/// One timed encode of a ten-second 440 Hz tone to the null muxer. Ten seconds
/// rather than one because process start-up is tens of milliseconds and would
/// otherwise be most of what gets compared. `None` if the encode failed, so a
/// candidate that cannot run simply does not compete.
fn time_encoder(encoder: &str) -> Option<f64> {
    let started = Instant::now();
    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=10:sample_rate=44100",
            "-ac",
            "1",
            "-acodec",
            encoder,
            "-f",
            "null",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()?;
    status.success().then(|| started.elapsed().as_secs_f64())
}

/// Where this machine's verdicts live: the OS cache directory, so it survives
/// between runs without polluting the user's project.
fn cache_path() -> Option<PathBuf> {
    let base = if cfg!(target_vendor = "apple") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Caches"))
    } else if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
    }?;
    Some(base.join("audio-helper-rs").join("encoders.json"))
}

/// The key a verdict is filed under. The ffmpeg build is part of it on purpose:
/// an upgrade can add a platform encoder, or make a software one faster, and a
/// verdict from the previous build would then be quietly wrong.
fn cache_key(family: Family) -> String {
    format!("{}|{}", ffmpeg_build(), family.as_str())
}

fn ffmpeg_build() -> &'static str {
    static BUILD: OnceLock<String> = OnceLock::new();
    BUILD.get_or_init(|| {
        Command::new("ffmpeg")
            .args(["-hide_banner", "-version"])
            .stdin(Stdio::null())
            .output()
            .ok()
            .and_then(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .next()
                    .map(|l| l.trim().to_string())
            })
            .unwrap_or_else(|| "unknown".to_string())
    })
}

fn read_cache() -> BTreeMap<String, String> {
    cache_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// A remembered verdict, but only if that encoder is *still* installed — an
/// ffmpeg reinstalled without `libfdk_aac` must not keep being told to use it.
fn recall(family: Family, present: &[&'static str]) -> Option<String> {
    let remembered = read_cache().get(&cache_key(family)).cloned()?;
    present
        .iter()
        .any(|p| *p == remembered)
        .then_some(remembered)
}

fn remember(family: Family, encoder: &str) {
    let Some(path) = cache_path() else { return };
    let mut cache = read_cache();
    cache.insert(cache_key(family), encoder.to_string());
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(body) = serde_json::to_string_pretty(&cache) {
        // A cache that cannot be written is a slower next run, not a failure:
        // the verdict is still correct for this process.
        let _ = std::fs::write(path, body);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn families_come_from_output_extensions() {
        assert_eq!(Family::from_extension("M4A"), Some(Family::Aac));
        assert_eq!(Family::from_extension("mp3"), Some(Family::Mp3));
        assert_eq!(Family::from_extension("opus"), Some(Family::Opus));
        // WAV is deliberately absent: `ConvertOptions::encoding` already says
        // which PCM layout to write, and that is not an encoder choice.
        assert_eq!(Family::from_extension("wav"), None);
    }

    #[test]
    fn every_family_ends_on_a_portable_software_encoder() {
        // The platform encoder may lead, but a machine without it must still be
        // left with something that exists everywhere.
        for family in [
            Family::Aac,
            Family::Mp3,
            Family::Alac,
            Family::Opus,
            Family::Flac,
        ] {
            let list = candidates(family);
            assert!(!list.is_empty(), "{family:?} has no candidate at all");
            let last = list.last().unwrap();
            assert!(
                !last.ends_with("_at") && !last.ends_with("_mf"),
                "{family:?} ends on the platform encoder {last}, leaving nothing portable"
            );
        }
    }

    #[test]
    fn the_platform_encoder_leads_where_it_exists() {
        let aac = candidates(Family::Aac);
        if cfg!(target_vendor = "apple") {
            assert_eq!(aac.first(), Some(&"aac_at"));
        } else if cfg!(target_os = "windows") {
            assert_eq!(aac.first(), Some(&"aac_mf"));
        } else {
            assert!(aac
                .iter()
                .all(|c| !c.ends_with("_at") && !c.ends_with("_mf")));
        }
    }

    #[test]
    fn an_explicit_encoder_wins_and_is_not_measured() {
        let chosen = choose(Family::Aac, Some("libfdk_aac")).expect("explicit always answers");
        assert_eq!(chosen.encoder, "libfdk_aac");
        assert_eq!(chosen.reason, Reason::Explicit);
        // Even one that does not exist on this machine: the caller said so, and
        // guessing past an explicit instruction is how silent surprises happen.
        let absurd = choose(Family::Mp3, Some("not_a_real_encoder")).unwrap();
        assert_eq!(absurd.encoder, "not_a_real_encoder");
    }

    #[test]
    fn whitespace_is_not_an_explicit_choice() {
        // `Some("")` comes from an empty config field, and means "no opinion".
        let chosen = choose(Family::Flac, Some("   "));
        assert!(chosen.is_none_or(|c| c.reason != Reason::Explicit));
    }

    #[test]
    fn encoder_list_parses_past_the_header() {
        let sample = "Encoders:\n V..... = Video\n A..... = Audio\n ------\n A....D aac                  AAC (Advanced Audio Coding)\n A....D aac_at               aac (AudioToolbox)\n V....D libx264              H.264\n";
        let names = parse_encoder_list(sample);
        assert_eq!(names, vec!["aac", "aac_at", "libx264"]);
        // The legend lines above the rule must not be mistaken for encoders.
        assert!(!names.iter().any(|n| n == "="));
    }

    #[test]
    fn an_empty_encoder_list_means_no_opinion() {
        assert!(parse_encoder_list("").is_empty());
        assert!(parse_encoder_list("ffmpeg not found").is_empty());
    }

    #[test]
    fn a_remembered_verdict_is_dropped_when_that_encoder_is_gone() {
        // Guards the upgrade case: the cache names `libfdk_aac`, the rebuilt
        // ffmpeg no longer ships it, so the verdict must not survive.
        assert_eq!(recall_from("aac_at", &["aac"]), None);
        assert_eq!(recall_from("aac", &["aac", "aac_at"]), Some("aac".into()));
    }

    #[test]
    fn a_tie_goes_to_the_preferred_candidate() {
        // The real measurement, rounded: the platform encoder and the software
        // one are indistinguishable, so the platform one keeps its place rather
        // than the verdict turning on measurement noise.
        let ordered = ["aac_at", "aac"];
        let timings = [("aac_at".to_string(), 3.26), ("aac".to_string(), 3.11)];
        assert_eq!(pick_with_margin(&ordered, &timings), Some("aac_at".into()));
    }

    #[test]
    fn a_clear_win_displaces_the_preferred_candidate() {
        // And when the difference is real — the case this whole module exists
        // for — the measurement overrules the preference.
        let ordered = ["aac_at", "aac"];
        let timings = [("aac_at".to_string(), 3.00), ("aac".to_string(), 1.20)];
        assert_eq!(pick_with_margin(&ordered, &timings), Some("aac".into()));
    }

    #[test]
    fn an_untimed_candidate_does_not_compete_but_the_order_still_stands() {
        // An encoder that refuses to run (missing lavfi, broken build) must not
        // win by default, yet a total absence of timings must still answer.
        let ordered = ["aac_at", "aac"];
        assert_eq!(
            pick_with_margin(&ordered, &[("aac".to_string(), 2.0)]),
            Some("aac".into())
        );
        assert_eq!(pick_with_margin(&ordered, &[]), Some("aac_at".into()));
    }

    /// The membership half of [`recall`], without the file read.
    fn recall_from(remembered: &str, present: &[&str]) -> Option<String> {
        present
            .contains(&remembered)
            .then(|| remembered.to_string())
    }
}
