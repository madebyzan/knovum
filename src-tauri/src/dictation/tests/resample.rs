use super::*;

#[test]
fn mono_16k_passthrough() {
    let input = vec![100, -200, 300, -400];
    let output = to_16k_mono(&input, 1, 16_000);
    assert_eq!(output, input);
}

#[test]
fn stereo_to_mono_16k() {
    let input = vec![100, 300, -200, -400];
    let output = to_16k_mono(&input, 2, 16_000);
    assert_eq!(output, vec![200, -300]);
}

#[test]
fn downsamples_48k_to_16k() {
    // 48kHz has exactly 3 samples per 16kHz sample
    let input = vec![100, 100, 100, 300, 300, 300];
    let output = to_16k_mono(&input, 1, 48_000);
    assert_eq!(output, vec![100, 300]);
}

#[test]
fn downsamples_44k1_to_16k() {
    let input = vec![1000; 4410]; // 100ms at 44.1kHz
    let output = to_16k_mono(&input, 1, 44_100);
    assert_eq!(output.len(), 1600); // 100ms at 16kHz
    assert!((output[0] - 1000).abs() <= 1);
}
