use super::*;
#[test]
fn image_output_shape_is_owned_by_its_operation() {
    assert_eq!(
        output_shape(&ImageTensorOperation::Upscale, [1, 3, 30, 20])
            .unwrap()
            .0,
        [1, 3, 120, 80]
    );
    assert!(output_shape(&ImageTensorOperation::Upscale, [1, 3, 4096, 4096]).is_err());
    assert!(output_shape(&ImageTensorOperation::Depth, [2, 3, 518, 518]).is_err());
    assert!(byte_length([1, 3, 0, 10], TensorDataType::Uint8).is_err());
}
