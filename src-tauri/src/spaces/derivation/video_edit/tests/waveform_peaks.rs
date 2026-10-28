use super::*;

fn bytes(samples: &[i16]) -> Vec<u8> {
    samples
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect()
}

#[test]
fn keeps_one_peak_per_slice_of_time() {
    let mut samples = vec![0_i16; SAMPLES_PER_PEAK * 2];
    samples[3] = -32_768;
    samples[SAMPLES_PER_PEAK + 1] = 16_384;
    let mut reducer = PeakReducer::default();
    reducer.push(&bytes(&samples));
    assert_eq!(reducer.take(), vec![255, 128]);
    assert!(reducer.finish().is_empty());
}

#[test]
fn joins_a_sample_split_across_two_reads() {
    let encoded = bytes(&[0, 32_767]);
    let mut reducer = PeakReducer::default();
    reducer.push(&encoded[..3]);
    reducer.push(&encoded[3..]);
    assert_eq!(reducer.finish(), vec![255]);
}

#[test]
fn closes_the_slice_the_source_ends_in() {
    let mut reducer = PeakReducer::default();
    reducer.push(&bytes(&[100; SAMPLES_PER_PEAK + 1]));
    assert_eq!(reducer.take().len(), 1);
    assert_eq!(reducer.finish(), vec![1]);
}
