use super::{scan::WindowScanner, SignalWarningKind};

fn window(scanner: &mut WindowScanner, start: u64, peak: &str, rms: &str) {
    for line in [
        format!(
            "frame:{} pts:{} pts_time:{:.1}",
            start / 100,
            start * 48,
            start as f64 / 1000.0
        ),
        format!("lavfi.astats.Overall.Peak_level={peak}"),
        format!("lavfi.astats.Overall.RMS_level={rms}"),
        "lavfi.astats.1.Number of NaNs=0".into(),
        "lavfi.astats.1.Number of Infs=0".into(),
        "lavfi.astats.Overall.Number_of_samples=4800".into(),
    ] {
        scanner.read(&line).unwrap();
    }
}
#[test]
fn signal_report_locates_silence_clipping_and_relative_volume() {
    let mut scanner = WindowScanner::new();
    for start in (0..1000).step_by(100) {
        window(&mut scanner, start, "-inf", "-inf");
    }
    for start in (1000..4000).step_by(100) {
        window(&mut scanner, start, "-10", "-20");
    }
    for start in (4000..4500).step_by(100) {
        window(&mut scanner, start, "0", "-5");
    }
    for start in (4500..5000).step_by(100) {
        window(&mut scanner, start, "-30", "-40");
    }
    let report = scanner.finish().unwrap();
    assert_eq!(report.scanned_duration_ms, 5000);
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.kind == SignalWarningKind::Silence
            && warning.start_ms == 0
            && warning.end_ms == 1000));
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.kind == SignalWarningKind::Clipping && warning.start_ms == 4000));
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.kind == SignalWarningKind::Loud && warning.start_ms == 4000));
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.kind == SignalWarningKind::Quiet && warning.start_ms == 4500));
}
#[test]
fn signal_report_rejects_overlapping_or_incomplete_windows() {
    let mut scanner = WindowScanner::new();
    window(&mut scanner, 0, "-10", "-20");
    window(&mut scanner, 0, "-10", "-20");
    assert!(scanner.finish().is_err());
    let mut scanner = WindowScanner::new();
    scanner.read("frame:0 pts:0 pts_time:0").unwrap();
    assert!(scanner.finish().is_err());
}
#[test]
fn signal_report_limits_findings_and_scan_duration() {
    let mut scanner = WindowScanner::new();
    for index in 0..202 {
        window(
            &mut scanner,
            index * 100,
            if index % 2 == 0 { "0" } else { "-10" },
            "-20",
        );
    }
    let report = scanner.finish().unwrap();
    assert!(report.truncated);
    assert_eq!(report.warnings.len(), 100);
    let mut scanner = WindowScanner::new();
    window(&mut scanner, 7_200_000, "-10", "-20");
    assert!(scanner.finish().unwrap().truncated);
}

#[test]
#[ignore = "Requires the explicit FFmpeg test binary"]
fn ffmpeg_inspection_reports_stereo_silence_and_float_clipping() {
    let directory = std::env::temp_dir().join(format!("knovum-qc-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let source = directory.join("signal.wav");
    let mut writer = hound::WavWriter::create(
        &source,
        hound::WavSpec {
            channels: 2,
            sample_rate: 48000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for frame in 0..96000 {
        for channel in 0..2 {
            writer
                .write_sample(if frame < 48000 {
                    0.0_f32
                } else if channel == 0 {
                    1.25
                } else {
                    0.1
                })
                .unwrap();
        }
    }
    writer.finalize().unwrap();
    let ffmpeg = std::path::PathBuf::from(
        std::env::var_os("KNOVUM_FFMPEG_PATH").expect("Set KNOVUM_FFMPEG_PATH"),
    );
    let report = super::inspect_file(
        &ffmpeg,
        &source,
        &tokio_util::sync::CancellationToken::new(),
        |_| {},
    )
    .unwrap();
    assert_eq!(report.scanned_duration_ms, 2000);
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.kind == SignalWarningKind::Silence
            && warning.start_ms == 0
            && warning.end_ms == 1000));
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.kind == SignalWarningKind::Clipping
            && warning.start_ms == 1000
            && warning.end_ms == 2000));
    std::fs::remove_dir_all(directory).unwrap();
}
