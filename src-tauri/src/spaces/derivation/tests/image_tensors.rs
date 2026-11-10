use crate::spaces::derivation::image_tensors::{depth_tensor, rgb_tensor};
use image::RgbaImage;

fn gradient(width: u32, height: u32) -> RgbaImage {
    RgbaImage::from_fn(width, height, |x, y| {
        image::Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255])
    })
}

#[test]
fn rgb_tensors_are_planar_and_drop_alpha() {
    let tensor = rgb_tensor(&gradient(2, 2));
    assert_eq!(tensor.shape, [1, 3, 2, 2]);
    assert_eq!(tensor.bytes.len(), 12);
    assert_eq!(
        &tensor.bytes[8..],
        &[128, 128, 128, 128],
        "blue plane is last"
    );
}

#[test]
fn depth_tensors_align_both_edges_to_the_patch_size() {
    let tensor = depth_tensor(&gradient(640, 480)).expect("depth tensor");
    let [_, channels, height, width] = tensor.shape;
    assert_eq!(channels, 3);
    assert!(height % 14 == 0 && width % 14 == 0, "{height}x{width}");
    assert_eq!(height, 518, "the short edge drives the 518 pixel scale");
    assert_eq!(tensor.bytes.len(), (height * width * 12) as usize);
    assert!(
        depth_tensor(&gradient(4000, 400)).is_err(),
        "extreme aspect ratios exceed the depth budget"
    );
}
