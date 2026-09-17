use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender},
    time::{Duration, Instant},
};

use block2::RcBlock;
use objc2::{rc::Retained, AnyThread};
use objc2_core_foundation::{CFString, CGSize, CFURL};
use objc2_foundation::{NSError, NSString, NSURL};
use objc2_image_io::CGImageDestination;
use objc2_quick_look_thumbnailing::{
    QLThumbnailGenerationRequest, QLThumbnailGenerationRequestRepresentationTypes,
    QLThumbnailGenerator, QLThumbnailRepresentation,
};
use tokio_util::sync::CancellationToken;

use super::ensure_not_cancelled;

/// Long edge of the requested bitmap. Quick Look fits the drawing inside this box, so a portrait
/// page keeps its own ratio instead of being stretched into a square.
const LONG_EDGE: f64 = 1024.0;
const POLL: Duration = Duration::from_millis(40);
const DEADLINE: Duration = Duration::from_secs(20);
const PNG: &str = "public.png";

type Outcome = Result<(), String>;

pub(super) fn save(
    source: &Path,
    target: &Path,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let request = request(source)?;
    // Drawn to a sibling first: a callback that lands after we gave up must never be able to
    // publish a half-written file under the name the asset payload points at.
    let partial = target.with_extension("partial");
    let (sender, receiver) = sync_channel::<Outcome>(1);
    unsafe {
        let generator = QLThumbnailGenerator::sharedGenerator();
        let completion = completion(sender, partial.clone());
        generator.generateBestRepresentationForRequest_completionHandler(&request, &completion);
        match wait(receiver, cancellation, &generator, &request) {
            Ok(()) => fs::rename(&partial, target).map_err(|error| error.to_string()),
            Err(error) => {
                let _ = fs::remove_file(&partial);
                Err(error)
            }
        }
    }
}

fn request(source: &Path) -> Result<Retained<QLThumbnailGenerationRequest>, String> {
    let url = NSURL::fileURLWithPath(&NSString::from_str(utf8(source)?));
    Ok(unsafe {
        QLThumbnailGenerationRequest::initWithFileAtURL_size_scale_representationTypes(
            QLThumbnailGenerationRequest::alloc(),
            &url,
            CGSize::new(LONG_EDGE, LONG_EDGE),
            1.0,
            // A real render only. The low-quality stand-in is a blurred cached bitmap and the icon
            // type is a generic document glyph, and the node sizes itself from this image.
            QLThumbnailGenerationRequestRepresentationTypes::Thumbnail,
        )
    })
}

/// The block captures owned data only, so a callback that fires after `save` returned writes its
/// file and drops its send into a closed channel instead of touching a borrowed stack frame.
fn completion(
    sender: SyncSender<Outcome>,
    partial: PathBuf,
) -> RcBlock<dyn Fn(*mut QLThumbnailRepresentation, *mut NSError)> {
    RcBlock::new(
        move |thumbnail: *mut QLThumbnailRepresentation, error: *mut NSError| {
            // try_send, never send: Quick Look owns this queue and must not be able to block on us.
            let _ = sender.try_send(unsafe { write_png(thumbnail, error, &partial) });
        },
    )
}

fn wait(
    receiver: Receiver<Outcome>,
    cancellation: &CancellationToken,
    generator: &QLThumbnailGenerator,
    request: &QLThumbnailGenerationRequest,
) -> Result<(), String> {
    let deadline = Instant::now() + DEADLINE;
    loop {
        if let Err(error) = ensure_not_cancelled(cancellation) {
            return abandon(generator, request, error);
        }
        match receiver.recv_timeout(POLL) {
            Ok(outcome) => return outcome,
            Err(RecvTimeoutError::Timeout) if Instant::now() < deadline => {}
            Err(RecvTimeoutError::Timeout) => {
                return abandon(
                    generator,
                    request,
                    "Quick Look did not finish the thumbnail in time".into(),
                );
            }
            Err(RecvTimeoutError::Disconnected) => {
                return Err("Quick Look dropped the thumbnail request without answering".into());
            }
        }
    }
}

fn abandon(
    generator: &QLThumbnailGenerator,
    request: &QLThumbnailGenerationRequest,
    error: String,
) -> Result<(), String> {
    unsafe { generator.cancelRequest(request) };
    Err(error)
}

/// Runs on Quick Look's own queue. `QLThumbnailRepresentation` is not `Send`, so it is encoded here
/// rather than handed back: ImageIO is a thread-safe C API and only the finished file crosses over.
///
/// # Safety
///
/// `thumbnail` must be null or point to a live `QLThumbnailRepresentation`, and `error` must be
/// null or point to a live `NSError`, for the duration of the call.
unsafe fn write_png(
    thumbnail: *mut QLThumbnailRepresentation,
    error: *mut NSError,
    partial: &Path,
) -> Result<(), String> {
    if !error.is_null() {
        return Err(describe(error));
    }
    if thumbnail.is_null() {
        return Err("Quick Look returned no thumbnail".into());
    }
    let image = unsafe { (*thumbnail).CGImage() };
    let url =
        CFURL::from_file_path(partial).ok_or("Thumbnail path cannot be expressed as a URL")?;
    let destination =
        unsafe { CGImageDestination::with_url(&url, &CFString::from_str(PNG), 1, None) }
            .ok_or("ImageIO refused to open the thumbnail destination")?;
    unsafe { destination.add_image(&image, None) };
    if unsafe { destination.finalize() } {
        Ok(())
    } else {
        Err("ImageIO could not encode the thumbnail as PNG".into())
    }
}

/// # Safety
///
/// `error` must point to a live `NSError`.
unsafe fn describe(error: *const NSError) -> String {
    let description = unsafe { (*error).localizedDescription() };
    format!("Quick Look failed: {description}")
}

fn utf8(path: &Path) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| "Asset path is not valid Unicode".to_owned())
}
