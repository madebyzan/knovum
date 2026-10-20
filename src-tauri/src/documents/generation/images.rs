use std::{cell::Cell, collections::BTreeMap, io::Cursor};

use image::ImageDecoder;

use super::document::{DocumentSource, ImageSource};

pub(crate) const MAX_IMAGE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 64 * 1024 * 1024;

pub(super) struct Image {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub(super) struct Images(pub(super) BTreeMap<String, Image>, pub(super) Cell<usize>);

impl Images {
    pub fn load(source: &DocumentSource) -> Result<Self, String> {
        Self::from_sources(&source.images)
    }

    pub fn from_sources(sources: &[ImageSource]) -> Result<Self, String> {
        if sources.len() > 2_000 {
            return Err("Document exceeds 2000 image resources".into());
        }
        let mut image_by_destination = BTreeMap::new();
        let mut total = 0;
        for source in sources {
            if image_by_destination.contains_key(&source.destination) {
                return Err("Document contains duplicate image resource identities".into());
            }
            let bytes = super::input::file::read_snapshot(&source.file, MAX_IMAGE_BYTES)?;
            total += bytes.len();
            let image = decode(&bytes)?;
            total += image.bytes.len();
            if total > MAX_TOTAL_BYTES {
                return Err("Document image resources exceed 64 MiB".into());
            }
            image_by_destination.insert(source.destination.clone(), image);
        }
        Ok(Self(image_by_destination, Cell::new(0)))
    }

    pub fn get(&self, destination: &str) -> Result<&Image, String> {
        self.0.get(destination).ok_or_else(|| {
            "Document image is unavailable; import the image into this Space before conversion"
                .into()
        })
    }

    /// Bound actual embedded copies too: one image referenced repeatedly is not free.
    pub fn clone_bytes(&self, destination: &str) -> Result<Vec<u8>, String> {
        let image = self.get(destination)?;
        self.record_embedded_bytes(image.bytes.len())?;
        Ok(image.bytes.clone())
    }

    pub fn record_embedded_bytes(&self, bytes: usize) -> Result<(), String> {
        let total = self.1.get().saturating_add(bytes);
        if total > MAX_TOTAL_BYTES {
            return Err("Document embedded image copies exceed 64 MiB".into());
        }
        self.1.set(total);
        Ok(())
    }

    pub fn binaries(&self) -> Vec<(String, Vec<u8>)> {
        self.0
            .iter()
            .enumerate()
            .map(|(index, (_, image))| (format!("image-{index}.png"), image.bytes.clone()))
            .collect()
    }

    pub fn filename(&self, destination: &str) -> Result<String, String> {
        self.0
            .keys()
            .position(|key| key == destination)
            .map(|index| format!("image-{index}.png"))
            .ok_or_else(|| "Document image resource is unavailable".into())
    }
}

pub(super) fn decode(bytes: &[u8]) -> Result<Image, String> {
    if bytes.len() as u64 > MAX_IMAGE_BYTES {
        return Err("Document image exceeds 32 MiB".into());
    }
    let format = image::guess_format(bytes).map_err(|error| error.to_string())?;
    let (width, height) = image::ImageReader::with_format(Cursor::new(bytes), format)
        .into_dimensions()
        .map_err(|error| error.to_string())?;
    if u64::from(width) * u64::from(height) > 24_000_000 {
        return Err("Document image exceeds 24 megapixels".into());
    }
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16_384);
    limits.max_image_height = Some(16_384);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(|error| error.to_string())?;
    let orientation = decoder.orientation().map_err(|error| error.to_string())?;
    let mut image =
        image::DynamicImage::from_decoder(decoder).map_err(|error| error.to_string())?;
    image.apply_orientation(orientation);
    let (width, height) = (image.width(), image.height());
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    if output.get_ref().len() as u64 > MAX_IMAGE_BYTES {
        return Err("Normalized document image exceeds 32 MiB".into());
    }
    Ok(Image {
        bytes: output.into_inner(),
        width,
        height,
    })
}
