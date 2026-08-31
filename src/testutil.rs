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
