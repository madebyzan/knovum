use super::*;

#[test]
fn available_gpu_precedes_cpu() {
    for (cuda, mps, expected) in [
        (true, true, "cuda"),
        (true, false, "cuda"),
        (false, true, "mps"),
        (false, false, "cpu"),
    ] {
        assert_eq!(Backends { cuda, mps }.preferred_device(), expected);
    }
}
