use super::{processing_error, ASSET_PROCESSING_CANCELLED};

#[test]
fn asset_processing_cancellation_has_an_explicit_product_code() {
    let error = processing_error(ASSET_PROCESSING_CANCELLED.to_owned());
    assert_eq!(error.code, "space_asset_processing_cancelled");
    assert_eq!(
        processing_error("enrichment-error: vendor detail".into()).code,
        "operation_failed"
    );
}
