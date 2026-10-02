//! Test-only WAV fixture generator, shared by every module's unit tests. Writes a
//! real (low-amplitude sine) signal rather than silence, so tests exercise ffmpeg
//! on genuine audio content instead of degenerate all-zero input.

use std::path::Path;

pub(crate) fn write_test_wav(path: &Path, sample_rate: u32, seconds: f64) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).expect("write test wav");
    let num_samples = (seconds * sample_rate as f64).round() as usize;
    let freq = 440.0_f64;
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let sample = (t * freq * std::f64::consts::TAU).sin() * 0.2 * i16::MAX as f64;
        writer.write_sample(sample as i16).expect("write sample");
    }
    writer.finalize().expect("finalize test wav");
}

/// Whether both `ffmpeg` and `ffprobe` can actually be executed on this machine.
///
/// Probed once and cached: every test in the crate asks, and spawning two
/// processes per test would dominate the suite's runtime.
pub(crate) fn ffmpeg_available() -> bool {
    use std::sync::OnceLock;
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        ["ffmpeg", "ffprobe"].iter().all(|bin| {
            std::process::Command::new(bin)
                .arg("-version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .is_ok_and(|s| s.success())
        })
    })
}

/// Ends the calling test early, with a printed reason, when `ffmpeg`/`ffprobe`
/// are not installed — *unless* this is CI, where their absence is a broken
/// workflow and must fail loudly.
///
/// Most of this crate is a thin, careful shell around those two binaries, so
/// most of its tests genuinely need them. Without this, a contributor who has
/// not installed ffmpeg gets fifteen `MissingBinary` panics that look like the
/// crate is broken. The `CI` check is what stops the escape hatch from hiding a
/// workflow that forgot to install ffmpeg: there, a skip is a failure.
macro_rules! require_ffmpeg {
    () => {
        if !$crate::testutil::ffmpeg_available() {
            assert!(
                std::env::var_os("CI").is_none(),
                "ffmpeg/ffprobe are missing under CI — the workflow must install them \
                 (see .github/workflows/ci.yml); skipping here would make CI green \
                 while testing nothing"
            );
            eprintln!("skipped: ffmpeg/ffprobe not on PATH (install ffmpeg to run the full suite)");
            return;
        }
    };
}
pub(crate) use require_ffmpeg;
