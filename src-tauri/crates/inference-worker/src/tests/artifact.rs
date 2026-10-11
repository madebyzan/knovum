use super::super::artifact::{read_tensor, tensor_bytes};
use knovum_inference_contracts::image_wire::{TensorDataType, TensorFileReference};

#[test]
fn tensor_budget_is_checked_without_allocating_the_claimed_shape() {
    assert!(tensor_bytes([1, 3, 0, 4], TensorDataType::Uint8, 1024).is_err());
    assert!(tensor_bytes([1, 3, 16384, 16384], TensorDataType::Uint8, 1024).is_err());
    assert_eq!(
        tensor_bytes([1, 3, 4, 4], TensorDataType::Float32LittleEndian, 192).unwrap(),
        192
    );
}

#[test]
fn external_reference_cannot_escape_work_directory() {
    let reference = TensorFileReference {
        file_name: "../outside.tensor".into(),
        byte_length: 4,
        sha256: "0".repeat(64),
        shape: [1, 1, 1, 1],
        data_type: TensorDataType::Float32LittleEndian,
    };
    assert!(read_tensor(std::path::Path::new("/missing"), &reference, 4).is_err());
}
