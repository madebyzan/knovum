use super::{Mix, Span, BLOCK_FRAMES, OVERLAP_FRAMES};

#[test]
fn neighbouring_blocks_share_every_overlapping_frame_exactly() {
    let total = 3 * BLOCK_FRAMES + BLOCK_FRAMES / 2;
    let blocks = total.div_ceil(BLOCK_FRAMES);
    let spans = (0..blocks)
        .map(|index| Span::of(index, blocks, total))
        .collect::<Vec<_>>();
    for frame in (0..total).step_by(997) {
        let sum: f32 = spans
            .iter()
            .filter(|span| (span.start..span.end).contains(&frame))
            .map(|span| span.weight(frame))
            .sum();
        assert!((sum - 1.0).abs() < 1e-5, "frame {frame} weighs {sum}");
    }
}

#[test]
fn a_source_shorter_than_one_block_is_one_call_taken_whole() {
    let span = Span::of(0, 1, 1_000);
    assert_eq!((span.start, span.end, span.settled), (0, 1_000, 1_000));
    assert_eq!(span.weight(0), 1.0);
    assert_eq!(span.weight(999), 1.0);
}

#[test]
fn a_block_settles_only_the_frames_its_successor_cannot_reach() {
    let total = 2 * BLOCK_FRAMES;
    let first = Span::of(0, 2, total);
    let second = Span::of(1, 2, total);
    assert_eq!(first.settled, second.start);
    assert_eq!(second.start, BLOCK_FRAMES - OVERLAP_FRAMES);
    assert_eq!(second.settled, total);
}

#[test]
fn the_mix_hands_out_cross_faded_frames_in_order() {
    // Two blocks of a constant estimate must settle to the same constant, seam included.
    let total = BLOCK_FRAMES + 2 * OVERLAP_FRAMES;
    let mut mix = Mix::default();
    let mut written = 0;
    for index in 0..2 {
        let span = Span::of(index, 2, total);
        let samples = vec![0.5_f32; (span.end - span.start) * 2];
        mix.add(&span, &samples, &samples);
        let (voice, rest) = mix.settle(span.settled);
        assert!(voice
            .iter()
            .chain(&rest)
            .all(|value| (value - 0.5).abs() < 1e-5));
        written += voice.len() / 2;
    }
    assert_eq!(written, total);
}
