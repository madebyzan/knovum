//! What the image graphs run by the worker read. Each constant states the input contract of one
//! graph family and nothing else, so a consumer reads its own numbers instead of unpacking a
//! general shape it does not use. Background-removal graphs are not here: the worker runs them
//! verbatim and the host owns their pre- and post-processing.

/// Channel-wise standardization applied after scaling a pixel to `[0, 1]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Standardization {
    pub mean: [f32; 3],
    pub std: [f32; 3],
}

impl Standardization {
    /// The standardized value of one `[0, 255]` sample of `channel`.
    pub fn apply(self, channel: usize, value: u8) -> f32 {
        (f32::from(value) / 255.0 - self.mean[channel]) / self.std[channel]
    }
}

pub const IMAGENET: Standardization = Standardization {
    mean: [0.485, 0.456, 0.406],
    std: [0.229, 0.224, 0.225],
};

/// MobileSAM resizes the source so its longest edge is this long; the encoder pads and normalizes
/// the raw `[0, 255]` pixels itself.
pub const MOBILE_SAM_LONGEST_EDGE: u32 = 1024;

/// SAM 2.1 Hiera-Tiny reads the source stretched to this square, unit-scaled and standardized.
pub const SAM2_EDGE: u32 = 1024;
pub const SAM2_STANDARDIZATION: Standardization = IMAGENET;

/// Big LaMa paints a window around each hole, resampled to the graph's fixed square. The window
/// keeps context on every side as a share of the hole's longer side, and never less than
/// `min_context` pixels: the model continues what surrounds a hole, so it must see enough of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InpaintingWindow {
    pub edge: u32,
    pub context_share: f32,
    pub min_context: u32,
}

pub const LAMA_WINDOW: InpaintingWindow = InpaintingWindow {
    edge: 512,
    context_share: 0.5,
    min_context: 32,
};

/// Real-ESRGAN graphs read one fixed square tile and return it `factor` times larger. Each tile
/// carries `context` pixels of its neighbours on every side; only its core is kept, so tile seams
/// fall in the discarded margin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UpscaleTiling {
    pub tile: u32,
    pub context: u32,
    pub factor: u32,
}

impl UpscaleTiling {
    /// Source pixels each tile contributes to the output.
    pub const fn core(self) -> u32 {
        self.tile - 2 * self.context
    }
}

/// Real-ESRGAN's reference tiling pads each tile by 10 pixels; a wider margin only repeats work.
pub const REAL_ESRGAN_TILING: UpscaleTiling = UpscaleTiling {
    tile: 128,
    context: 10,
    factor: 4,
};

/// Depth Anything V2 reads the source resized so its shorter edge is near `short_edge`, both
/// edges aligned to the ViT `patch`, unit-scaled and standardized.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepthInput {
    pub short_edge: u32,
    pub patch: u32,
    pub standardization: Standardization,
}

pub const DEPTH_ANYTHING_V2_INPUT: DepthInput = DepthInput {
    short_edge: 518,
    patch: 14,
    standardization: IMAGENET,
};
