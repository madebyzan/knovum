pub(super) const REVISION: &str = "3533c8cf8e369892e6b79ff1bf80f7b0286a54ee";
pub(super) const CONFIG_SHA256: &str =
    "48b707aa26903e23982353763edcfd03d7940d2388420af01762791366a3fd52";
pub(super) const CONFIG: &str = include_str!("community1-config.yaml");

pub(super) struct File {
    pub path: &'static str,
    pub size: u64,
    pub sha256: &'static str,
}

pub(super) const FILES: &[File] = &[
    File {
        path: "embedding/pytorch_model.bin",
        size: 26_646_242,
        sha256: "6f10ff60898a1d185fa22e1d11e0bfa8a92efec811f11bca48cb8cafebefd929",
    },
    File {
        path: "segmentation/pytorch_model.bin",
        size: 5_906_507,
        sha256: "7ad24338d844fb95985486eb1a464e32d229f6d7a03c9abe60f978bacf3f816e",
    },
    File {
        path: "plda/plda.npz",
        size: 133_852,
        sha256: "9b77bcd840692710dd3496f62ecfeed8d8e5f002fd991b785079b244eab7d255",
    },
    File {
        path: "plda/xvec_transform.npz",
        size: 134_376,
        sha256: "325f1ce8e48f7e55e9c8aa47e05d2766b7c48c4b25b8de8dd751e7a4cc5fbe8f",
    },
];

pub(super) const TOTAL_BYTES: u64 = 32_820_977;
