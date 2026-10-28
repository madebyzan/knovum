//! Reduces a stream of 16-bit mono samples to one peak per fixed slice of time. The waveform is
//! read at whatever zoom the timeline has, so peaks are kept at a fixed rate rather than a fixed
//! count: a long source gets more of them instead of coarser ones.

/// Peaks per second of source; fine enough for the timeline's closest zoom.
pub(super) const PEAKS_PER_SECOND: u32 = 100;
/// Rate the audio is decoded at for peaks. Peaks need amplitude, not fidelity.
pub(super) const SAMPLE_RATE: u32 = 8_000;
const SAMPLES_PER_PEAK: usize = (SAMPLE_RATE / PEAKS_PER_SECOND) as usize;

/// Accepts little-endian `s16` bytes in arbitrary pieces, as a pipe delivers them.
#[derive(Default)]
pub(super) struct PeakReducer {
    carry: Option<u8>,
    samples: usize,
    peak: u16,
    ready: Vec<u8>,
}

impl PeakReducer {
    pub(super) fn push(&mut self, bytes: &[u8]) {
        let mut bytes = bytes;
        if let Some(low) = self.carry.take() {
            let Some((&high, rest)) = bytes.split_first() else {
                self.carry = Some(low);
                return;
            };
            self.sample(i16::from_le_bytes([low, high]));
            bytes = rest;
        }
        let mut pairs = bytes.chunks_exact(2);
        for pair in pairs.by_ref() {
            self.sample(i16::from_le_bytes([pair[0], pair[1]]));
        }
        self.carry = pairs.remainder().first().copied();
    }

    /// Peaks completed since the last call, scaled to `0..=255`.
    pub(super) fn take(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.ready)
    }

    /// The trailing partial slice is still a peak: the source ends inside it.
    pub(super) fn finish(mut self) -> Vec<u8> {
        if self.samples > 0 {
            self.close();
        }
        self.ready
    }

    fn sample(&mut self, value: i16) {
        self.peak = self.peak.max(value.unsigned_abs());
        self.samples += 1;
        if self.samples == SAMPLES_PER_PEAK {
            self.close();
        }
    }

    fn close(&mut self) {
        let scaled = (u32::from(self.peak.min(32_767)) * 255 + 16_383) / 32_767;
        self.ready.push(scaled as u8);
        self.samples = 0;
        self.peak = 0;
    }
}

#[cfg(test)]
mod tests {
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
}
