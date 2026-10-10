use super::super::contracts::ImageOperation;
use super::raster;
use crate::ai::{
    contracts::image_wire::TensorDataType, local_runtime::image::compute::ImageTensor,
};
use image::{DynamicImage, Rgba, RgbaImage};
use tokio_util::sync::CancellationToken;

#[test]
fn inpainting_materialization_preserves_alpha_and_pixel_coordinates() {
    let source = RgbaImage::from_fn(2, 1, |x, _| {
        Rgba([10, 20, 30, if x == 0 { 0 } else { 127 }])
    });
    let tensor = ImageTensor {
        bytes: vec![40, 41, 50, 51, 60, 61],
        shape: [1, 3, 1, 2],
        data_type: TensorDataType::Uint8,
    };
    let image = raster::materialize(
        source,
        tensor,
        &ImageOperation::RemoveObject { strokes: vec![] },
        &CancellationToken::new(),
    )
    .unwrap();
    let decoded = image::load_from_memory(&image.png).unwrap().to_rgba8();
    assert_eq!(decoded.get_pixel(0, 0), &Rgba([40, 50, 60, 0]));
    assert_eq!(decoded.get_pixel(1, 0), &Rgba([41, 51, 61, 127]));
}

#[test]
fn depth_publication_keeps_sixteen_bit_relative_values() {
    let tensor = ImageTensor {
        bytes: [0_f32, 0.5, 1.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect(),
        shape: [1, 1, 1, 3],
        data_type: TensorDataType::Float32LittleEndian,
    };
    let image = raster::materialize(
        RgbaImage::new(3, 1),
        tensor,
        &ImageOperation::GenerateDepth,
        &CancellationToken::new(),
    )
    .unwrap();
    let DynamicImage::ImageLuma16(field) = image::load_from_memory(&image.png).unwrap() else {
        panic!("Depth PNG lost its precision");
    };
    assert_eq!(field.into_raw(), vec![0, 32768, 65535]);
}

#[test]
fn cancelled_materialization_cannot_publish() {
    let token = CancellationToken::new();
    token.cancel();
    let tensor = ImageTensor {
        bytes: vec![0, 0, 0],
        shape: [1, 3, 1, 1],
        data_type: TensorDataType::Uint8,
    };
    assert!(raster::materialize(
        RgbaImage::new(1, 1),
        tensor,
        &ImageOperation::RemoveObject { strokes: vec![] },
        &token
    )
    .is_err());
}

#[test]
fn selection_preparation_keeps_small_sources_and_reduces_only_large_sources() {
    for (width, height, expected) in [(128, 96, [1, 3, 96, 128]), (4096, 64, [1, 3, 32, 2048])] {
        let source = RgbaImage::from_pixel(width, height, Rgba([10, 20, 30, 127]));
        let mut bytes = Vec::new();
        DynamicImage::ImageRgba8(source.clone())
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
        let (original, input) = super::selection::prepare_source(&bytes).unwrap();
        assert_eq!(original, source);
        assert_eq!(input.shape, expected);
        assert_eq!(input.bytes.len(), (expected[2] * expected[3] * 3) as usize);
    }
}
