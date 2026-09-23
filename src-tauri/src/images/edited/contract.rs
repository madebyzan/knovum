use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ImageEdit {
    version: u8,
    pub crop: Crop,
    pub rotation: f64,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub adjustments: Adjustments,
    pub eraser_strokes: Vec<EraserStroke>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Crop {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Adjustments {
    pub exposure: f64,
    pub contrast: f64,
    pub highlights: f64,
    pub shadows: f64,
    pub whites: f64,
    pub blacks: f64,
    pub saturation: f64,
    pub warmth: f64,
    pub sharpness: f64,
    pub vignette: f64,
    pub vibrance: f64,
    pub tint: f64,
    pub clarity: f64,
    pub grain: f64,
    pub soft_light: f64,
    pub glow: f64,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EraserStroke {
    pub diameter: f64,
    pub points: Vec<f64>,
}

impl ImageEdit {
    pub(super) fn parse(value: &Value) -> Result<Self, String> {
        if value.is_null() {
            return Ok(Self::identity());
        }
        let edit: Self = serde_json::from_value(value.clone())
            .map_err(|error| format!("Invalid imageEdit: {error}"))?;
        edit.validate()?;
        Ok(edit)
    }

    fn identity() -> Self {
        Self {
            version: 5,
            crop: Crop { x: 0.0, y: 0.0, width: 1.0, height: 1.0 },
            rotation: 0.0,
            flip_horizontal: false,
            flip_vertical: false,
            adjustments: Adjustments {
                exposure: 0.0,
                contrast: 0.0,
                highlights: 0.0,
                shadows: 0.0,
                whites: 0.0,
                blacks: 0.0,
                saturation: 0.0,
                warmth: 0.0,
                sharpness: 0.0,
                vignette: 0.0,
                vibrance: 0.0,
                tint: 0.0,
                clarity: 0.0,
                grain: 0.0,
                soft_light: 0.0,
                glow: 0.0,
            },
            eraser_strokes: Vec::new(),
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.version != 5 {
            return Err("imageEdit.version must be 5".into());
        }
        finite_range(self.crop.x, 0.0, 1.0, "imageEdit.crop.x")?;
        finite_range(self.crop.y, 0.0, 1.0, "imageEdit.crop.y")?;
        finite_range(self.crop.width, 0.0, 1.0, "imageEdit.crop.width")?;
        finite_range(self.crop.height, 0.0, 1.0, "imageEdit.crop.height")?;
        if self.crop.width == 0.0
            || self.crop.height == 0.0
            || self.crop.x + self.crop.width > 1.0
            || self.crop.y + self.crop.height > 1.0
        {
            return Err("imageEdit.crop exceeds the image bounds".into());
        }
        finite_range(self.rotation, 0.0, 360.0, "imageEdit.rotation")?;
        if self.rotation >= 360.0 {
            return Err("imageEdit.rotation is invalid".into());
        }
        for (name, value) in self.adjustments.values() {
            let min = if ["sharpness", "clarity", "grain", "softLight", "glow"]
                .iter().any(|key| name.ends_with(key)) { 0.0 } else { -100.0 };
            finite_range(value, min, 100.0, name)?;
        }
        if self.eraser_strokes.len() > 500 {
            return Err("imageEdit.eraserStrokes exceeds 500 entries".into());
        }
        for stroke in &self.eraser_strokes {
            finite_range(stroke.diameter, 0.0, 0.5, "imageEdit.eraserStroke.diameter")?;
            if stroke.diameter == 0.0 {
                return Err("imageEdit.eraserStroke.diameter must be positive".into());
            }
            if stroke.points.len() < 4 || stroke.points.len() > 4096 || stroke.points.len() % 2 != 0 {
                return Err("imageEdit.eraserStroke.points length is invalid".into());
            }
            for point in &stroke.points {
                finite_range(*point, 0.0, 1.0, "imageEdit.eraserStroke.points")?;
            }
        }
        Ok(())
    }
}

impl Adjustments {
    fn values(&self) -> [(&'static str, f64); 16] {
        [
            ("imageEdit.adjustments.exposure", self.exposure),
            ("imageEdit.adjustments.contrast", self.contrast),
            ("imageEdit.adjustments.highlights", self.highlights),
            ("imageEdit.adjustments.shadows", self.shadows),
            ("imageEdit.adjustments.whites", self.whites),
            ("imageEdit.adjustments.blacks", self.blacks),
            ("imageEdit.adjustments.saturation", self.saturation),
            ("imageEdit.adjustments.warmth", self.warmth),
            ("imageEdit.adjustments.sharpness", self.sharpness),
            ("imageEdit.adjustments.vignette", self.vignette),
            ("imageEdit.adjustments.vibrance", self.vibrance),
            ("imageEdit.adjustments.tint", self.tint),
            ("imageEdit.adjustments.clarity", self.clarity),
            ("imageEdit.adjustments.grain", self.grain),
            ("imageEdit.adjustments.softLight", self.soft_light),
            ("imageEdit.adjustments.glow", self.glow),
        ]
    }
}

fn finite_range(value: f64, min: f64, max: f64, label: &str) -> Result<(), String> {
    if value.is_finite() && value >= min && value <= max {
        Ok(())
    } else {
        Err(format!("{label} must be between {min} and {max}"))
    }
}
