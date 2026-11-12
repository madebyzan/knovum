use image::{Rgba, RgbaImage};

use super::field::{coverage, model_input, placement};
use crate::local_resources::models::catalog::{SegmentationModel, SegmentationSpec};

fn spec(model: SegmentationModel) -> &'static SegmentationSpec {
    model.spec()
}

const PLANE: usize = 1024 * 1024;

#[test]
fn a_stretched_graph_reads_the_whole_square_standardized_channel_first() {
    let spec = spec(SegmentationModel::BirefnetLite);
    let source = RgbaImage::from_pixel(3, 2, Rgba([255, 128, 0, 255]));
    assert_eq!(placement(spec, (3, 2)), [0, 0, 1024, 1024]);
    let input = model_input(spec, &source);
    assert_eq!(input.len(), PLANE * 3);
    let expected = |channel: usize, value: u8| spec.standardization.apply(channel, value);
    for (channel, value) in [(0, 255), (1, 128), (2, 0)] {
        assert!((input[channel * PLANE] - expected(channel, value)).abs() < 1e-5);
        assert!((input[(channel + 1) * PLANE - 1] - expected(channel, value)).abs() < 1e-5);
    }
}

#[test]
fn a_letterboxed_graph_keeps_the_aspect_ratio_centred_on_black() {
    let spec = spec(SegmentationModel::IsnetAnime);
    // A 2:1 landscape fills the width and half the height, centred: rows 256..768.
    assert_eq!(placement(spec, (2000, 1000)), [0, 256, 1024, 512]);
    // A portrait fills the height; the reference truncates the shorter side.
    assert_eq!(placement(spec, (1000, 3000)), [341, 0, 341, 1024]);
    let input = model_input(
        spec,
        &RgbaImage::from_pixel(2, 1, Rgba([255, 255, 255, 255])),
    );
    assert_eq!(input[0], 0.0, "padding above the picture is black");
    assert_eq!(
        input[600 * 1024 + 512],
        1.0,
        "the picture is unit-scaled, not standardized"
    );
    assert_eq!(
        input[1023 * 1024],
        0.0,
        "padding below the picture is black"
    );
}

#[test]
fn logits_read_as_their_sigmoid_and_probabilities_as_themselves() {
    let logits = coverage(
        spec(SegmentationModel::BirefnetLite),
        &vec![0.0; PLANE],
        (4, 4),
    )
    .unwrap();
    assert!(logits.iter().all(|value| (value - 0.5).abs() < 1e-6));
    // A confident logit is opaque: min-max stretching once turned these into a translucent subject.
    let confident = coverage(
        spec(SegmentationModel::BirefnetLite),
        &vec![12.0; PLANE],
        (4, 4),
    )
    .unwrap();
    assert!(confident.iter().all(|value| *value > 0.9999));
    let probabilities = coverage(
        spec(SegmentationModel::Ben2Base),
        &vec![0.25; PLANE],
        (4, 4),
    )
    .unwrap();
    assert!(probabilities
        .iter()
        .all(|value| (value - 0.25).abs() < 1e-6));
}

#[test]
fn a_field_with_no_subject_stays_empty() {
    // A probability model that sees nothing must not have its noise stretched to full coverage.
    let mut field = vec![0.0; PLANE];
    field[0] = 0.02;
    let read = coverage(spec(SegmentationModel::IsnetAnime), &field, (1024, 1024)).unwrap();
    assert!(read.iter().all(|value| *value <= 0.02));
}

#[test]
fn coverage_is_read_from_the_placed_picture_only() {
    let spec = spec(SegmentationModel::IsnetAnime);
    // Subject in the letterbox rows of a 2:1 picture, nothing in the padding around it.
    let mut field = vec![0.0; PLANE];
    for y in 256..768 {
        for x in 0..1024 {
            field[y * 1024 + x] = 1.0;
        }
    }
    let read = coverage(spec, &field, (20, 10)).unwrap();
    assert_eq!(read.len(), 200);
    assert!(read.iter().all(|value| (value - 1.0).abs() < 1e-6));
}

#[test]
fn coverage_is_resampled_bilinearly_to_the_source() {
    let spec = spec(SegmentationModel::Ben2Base);
    // Left half subject, right half backdrop: the source sees a soft step, never a ringing one.
    let field = (0..PLANE)
        .map(|index| if index % 1024 < 512 { 1.0 } else { 0.0 })
        .collect::<Vec<_>>();
    let read = coverage(spec, &field, (4096, 1)).unwrap();
    assert!(read.iter().all(|value| (0.0..=1.0).contains(value)));
    assert!(
        read.windows(2).all(|pair| pair[1] <= pair[0]),
        "monotonic edge"
    );
    assert_eq!(read[0], 1.0);
    assert_eq!(read[4095], 0.0);
}

#[test]
fn a_field_of_the_wrong_size_is_an_invalid_output() {
    assert!(coverage(
        spec(SegmentationModel::BirefnetLite),
        &vec![0.0; PLANE - 1],
        (2, 2)
    )
    .is_err());
}

#[test]
fn every_model_is_addressed_by_its_own_specification() {
    for model in SegmentationModel::ALL {
        let spec = model.spec();
        assert_eq!(
            SegmentationModel::from_spec_id(spec.spec_id).unwrap(),
            model
        );
        assert!(
            spec.spec_id.contains(spec.sha256),
            "{model:?} spec names its weights"
        );
        assert_eq!(model.descriptor().sha256, spec.sha256);
        assert!(model.models() == [model.weights()]);
    }
    assert!(SegmentationModel::from_spec_id("retired-model").is_err());
}
