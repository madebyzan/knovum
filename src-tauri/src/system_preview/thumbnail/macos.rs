use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender},
        Arc, Mutex,
    },
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
    claim_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    let partial = super::claim_output_paths(target, claim_path)?;
    let request = request(source)?;
    // Draw to a sibling first. Closing the shared gate waits for an in-flight encoder and makes a
    // late callback inert before the caller can remove this partial or release its ownership.
    let active = Arc::new(Mutex::new(true));
    let (sender, receiver) = sync_channel::<Outcome>(1);
    unsafe {
        let generator = QLThumbnailGenerator::sharedGenerator();
        let completion = completion(sender, partial.clone(), Arc::clone(&active));
        generator.generateBestRepresentationForRequest_completionHandler(&request, &completion);
        let outcome = wait(receiver, cancellation, &generator, &request);
        let published = close_writer(&active)
            .and(outcome)
            .and_then(|()| fs::rename(&partial, target).map_err(|error| error.to_string()));
        if published.is_err() {
            let _ = fs::remove_file(&partial);
        }
        published
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

/// The block captures owned data only. The gate keeps its disk write inside `save`'s lifetime even
/// when Quick Look delivers the callback after cancellation or timeout.
fn completion(
    sender: SyncSender<Outcome>,
    partial: PathBuf,
    active: Arc<Mutex<bool>>,
) -> RcBlock<dyn Fn(*mut QLThumbnailRepresentation, *mut NSError)> {
    RcBlock::new(
        move |thumbnail: *mut QLThumbnailRepresentation, error: *mut NSError| {
            let outcome =
                with_open_writer(&active, || unsafe { write_png(thumbnail, error, &partial) });
            // try_send, never send: Quick Look owns this queue and must not be able to block on us.
            let _ = sender.try_send(outcome);
        },
    )
}

fn with_open_writer<T>(
    active: &Mutex<bool>,
    write: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let active = active
        .lock()
        .map_err(|_| "Quick Look thumbnail writer lock was poisoned".to_owned())?;
    if !*active {
        return Err("Quick Look thumbnail writer is closed".into());
    }
    write()
}

fn close_writer(active: &Mutex<bool>) -> Result<(), String> {
    let mut active = active
        .lock()
        .map_err(|_| "Quick Look thumbnail writer lock was poisoned".to_owned())?;
    *active = false;
    Ok(())
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

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::{close_writer, with_open_writer};

    #[test]
    fn closed_writer_rejects_late_callbacks_before_they_touch_disk() {
        let active = Mutex::new(true);
        close_writer(&active).unwrap();
        let mut wrote = false;
        let result = with_open_writer(&active, || {
            wrote = true;
            Ok(())
        });
        assert_eq!(result.unwrap_err(), "Quick Look thumbnail writer is closed");
        assert!(!wrote);
    }
}
