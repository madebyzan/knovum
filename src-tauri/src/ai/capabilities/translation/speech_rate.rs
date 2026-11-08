//! Spoken-duration estimates for dubbing scripts.
//!
//! Syllable rates are the measured adult narrative speech rates of each language
//! (Pellegrino, Coupe & Marsico 2011 and per-language follow-up studies), in syllables per
//! second. The estimator is an orthographic approximation whose only job is deciding whether a
//! translation clearly exceeds its physical time window; marginal cases are deliberately left
//! alone because a needless reduction round costs a model call and can strip nuance.

/// How far the estimated spoken duration may exceed the physical window before a shortened
/// adaptation is requested. The tolerance must stay above the estimator's own error so only
/// clear overruns trigger a second model call.
const REDUCTION_TOLERANCE: f64 = 1.2;

pub(crate) fn syllables_per_second(language: &str) -> Option<f64> {
    Some(match primary_language(language) {
        "zh" => 5.2,
        "en" => 6.0,
        "ja" => 7.8,
        "ko" => 6.4,
        "fr" => 7.7,
        "es" => 7.8,
        "pt" => 7.0,
        "it" => 7.3,
        "de" => 6.0,
        "ru" => 6.9,
        "ar" => 6.0,
        _ => return None,
    })
}

pub(crate) fn estimate_duration_ms(text: &str, language: &str) -> Option<u64> {
    let rate = syllables_per_second(language)?;
    let syllables = estimate_syllables(text, primary_language(language));
    Some(((syllables as f64 / rate) * 1_000.0).round() as u64)
}

pub(crate) fn requires_reduction(estimated_duration_ms: u64, window_ms: u64) -> bool {
    estimated_duration_ms as f64 > window_ms as f64 * REDUCTION_TOLERANCE
}

fn primary_language(language: &str) -> &str {
    language.split('-').next().unwrap_or(language)
}

/// Counts one syllable per CJK or hangul character, per maximal vowel group in alphabetic and
/// Cyrillic words, and per written digit. Two biases are accepted and documented: Japanese kanji
/// readings can span several morae but count once, and unvocalized Arabic omits its short vowels;
/// both underestimate, which only ever skips a reduction round the fitter can still absorb.
fn estimate_syllables(text: &str, language: &str) -> usize {
    let is_vowel = vowel_set(language);
    let mut syllables = 0;
    let mut in_vowel_group = false;
    for character in text.chars() {
        if is_syllabic_character(character) || character.is_numeric() {
            syllables += 1;
            in_vowel_group = false;
            continue;
        }
        let lower = character.to_lowercase().next().unwrap_or(character);
        if is_vowel(lower) {
            syllables += usize::from(!in_vowel_group);
            in_vowel_group = true;
        } else {
            in_vowel_group = false;
        }
    }
    syllables
}

fn is_syllabic_character(character: char) -> bool {
    matches!(character as u32,
        0x3041..=0x30FF   // Hiragana and katakana, one mora each
        | 0x3400..=0x4DBF // CJK unified ideographs extension A
        | 0x4E00..=0x9FFF // CJK unified ideographs
        | 0xAC00..=0xD7AF // Hangul syllable blocks
        | 0xF900..=0xFAFF) // CJK compatibility ideographs
}

fn vowel_set(language: &str) -> fn(char) -> bool {
    match language {
        "ru" => |character| {
            matches!(
                character,
                'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я'
            )
        },
        "ar" => {
            |character| {
                matches!(
                    character,
                    'ا' | 'أ' | 'إ' | 'آ' | 'ٱ' | 'و' | 'ي' | 'ى'
                    // Short-vowel diacritics, when the text carries them at all.
                    | '\u{064B}'..='\u{0650}'
                )
            }
        }
        _ => |character| {
            matches!(
                character,
                'a' | 'e'
                    | 'i'
                    | 'o'
                    | 'u'
                    | 'y'
                    | 'à'
                    | 'á'
                    | 'â'
                    | 'ã'
                    | 'ä'
                    | 'å'
                    | 'ā'
                    | 'ă'
                    | 'æ'
                    | 'è'
                    | 'é'
                    | 'ê'
                    | 'ë'
                    | 'ē'
                    | 'ĕ'
                    | 'ì'
                    | 'í'
                    | 'î'
                    | 'ï'
                    | 'ī'
                    | 'ĭ'
                    | 'ò'
                    | 'ó'
                    | 'ô'
                    | 'õ'
                    | 'ö'
                    | 'ø'
                    | 'ō'
                    | 'ŏ'
                    | 'œ'
                    | 'ù'
                    | 'ú'
                    | 'û'
                    | 'ü'
                    | 'ū'
                    | 'ŭ'
                    | 'ý'
                    | 'ÿ'
            )
        },
    }
}
