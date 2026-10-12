use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Title placeholders have two lines; overflow remains visible as paginated body content.
pub(super) fn split(text: &str) -> (String, String) {
    let mut width = 0;
    let mut boundary = text.len();
    for (index, grapheme) in text.grapheme_indices(true) {
        let units = grapheme.width_cjk().max(1);
        if grapheme.contains('\n') || width + units > 32 {
            boundary = index;
            break;
        }
        width += units;
    }
    (text[..boundary].to_owned(), text[boundary..].to_owned())
}
