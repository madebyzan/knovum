use std::path::Path;
use windows::{
    core::{Interface, HSTRING},
    Data::Pdf::{PdfDocument, PdfPageRenderOptions},
    Foundation::IClosable,
    Graphics::Imaging::BitmapEncoder,
    Storage::{
        StorageFile,
        Streams::{DataReader, DataWriter, InMemoryRandomAccessStream},
    },
    Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
};

struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { RoUninitialize() };
    }
}

/// WinRT Close releases rendering buffers even when a later fallible operation returns early.
struct CloseOnDrop(IClosable);
impl Drop for CloseOnDrop {
    fn drop(&mut self) {
        if let Err(error) = self.0.Close() {
            log::warn!("Native PDF resource cleanup failed: {error}");
        }
    }
}

pub(super) fn render(
    source: &Path,
    page_number: u32,
    region: Option<crate::documents::pdf_input::PdfRegion>,
) -> Result<super::PdfRaster, Box<dyn std::error::Error>> {
    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)?;
    }
    let _apartment = Apartment;
    let path = HSTRING::from(source);
    let file = StorageFile::GetFileFromPathAsync(&path)?.join()?;
    let document = PdfDocument::LoadFromFileAsync(&file)?.join()?;
    render_document(&document, page_number, region)
}

pub(super) fn render_bytes(
    bytes: &[u8],
    page_number: u32,
    region: Option<crate::documents::pdf_input::PdfRegion>,
) -> Result<super::PdfRaster, Box<dyn std::error::Error>> {
    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)?;
    }
    let _apartment = Apartment;
    let stream = InMemoryRandomAccessStream::new()?;
    let _stream = CloseOnDrop(stream.cast()?);
    let writer = DataWriter::CreateDataWriter(&stream.GetOutputStreamAt(0)?)?;
    let _writer = CloseOnDrop(writer.cast()?);
    writer.WriteBytes(bytes)?;
    writer.StoreAsync()?.join()?;
    writer.FlushAsync()?.join()?;
    stream.Seek(0)?;
    let document = PdfDocument::LoadFromStreamAsync(&stream)?.join()?;
    render_document(&document, page_number, region)
}

fn render_document(
    document: &PdfDocument,
    page_number: u32,
    region: Option<crate::documents::pdf_input::PdfRegion>,
) -> Result<super::PdfRaster, Box<dyn std::error::Error>> {
    let page = document.GetPage(page_number - 1)?;
    let _page = CloseOnDrop(page.cast()?);
    let size = page.Size()?;
    // WinRT exposes DIPs (1/96 inch); pdf-inspector Display uses points (1/72 inch).
    const DIPS_PER_POINT: f64 = 96.0 / 72.0;
    let page_height = f64::from(size.Height) / DIPS_PER_POINT;
    let crop = super::viewport(f64::from(size.Width) / DIPS_PER_POINT, page_height, region)?;
    let (width, height) = super::dimensions(crop.width, crop.height)?;
    let options = PdfPageRenderOptions::new()?;
    options.SetSourceRect(windows::Foundation::Rect {
        X: (crop.x * DIPS_PER_POINT) as f32,
        Y: ((page_height - crop.y - crop.height) * DIPS_PER_POINT) as f32,
        Width: (crop.width * DIPS_PER_POINT) as f32,
        Height: (crop.height * DIPS_PER_POINT) as f32,
    })?;
    options.SetDestinationWidth(width)?;
    options.SetDestinationHeight(height)?;
    options.SetBitmapEncoderId(BitmapEncoder::PngEncoderId()?)?;
    let stream = InMemoryRandomAccessStream::new()?;
    let _stream = CloseOnDrop(stream.cast()?);
    page.RenderWithOptionsToStreamAsync(&stream, &options)?
        .join()?;
    let length = u32::try_from(stream.Size()?)?;
    if length > 32 * 1024 * 1024 {
        return Err("Rendered PDF page exceeds 32 MiB".into());
    }
    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0)?)?;
    let _reader = CloseOnDrop(reader.cast()?);
    reader.LoadAsync(length)?.join()?;
    let mut bytes = vec![0; length as usize];
    reader.ReadBytes(&mut bytes)?;
    Ok(super::PdfRaster {
        bytes,
        width,
        height,
    })
}
