//! How many columns a terminal or an editor shows a string in, as aligning a table needs.

use std::cmp::Ordering;

use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

/// The characters shown two columns wide: East Asian wide and full-width characters, and emoji shown as pictures.
const WIDE: &[(u32, u32)] = &[
    (0x1100, 0x115F),
    (0x231A, 0x231B),
    (0x23E9, 0x23EC),
    (0x23F0, 0x23F0),
    (0x23F3, 0x23F3),
    (0x25FD, 0x25FE),
    (0x2614, 0x2615),
    (0x2648, 0x2653),
    (0x267F, 0x267F),
    (0x2693, 0x2693),
    (0x26A1, 0x26A1),
    (0x26AA, 0x26AB),
    (0x26BD, 0x26BE),
    (0x26C4, 0x26C5),
    (0x26CE, 0x26CE),
    (0x26D4, 0x26D4),
    (0x26EA, 0x26EA),
    (0x26F2, 0x26F3),
    (0x26F5, 0x26F5),
    (0x26FA, 0x26FA),
    (0x26FD, 0x26FD),
    (0x2705, 0x2705),
    (0x270A, 0x270B),
    (0x2728, 0x2728),
    (0x274C, 0x274C),
    (0x274E, 0x274E),
    (0x2753, 0x2755),
    (0x2757, 0x2757),
    (0x2795, 0x2797),
    (0x27B0, 0x27B0),
    (0x27BF, 0x27BF),
    (0x2B1B, 0x2B1C),
    (0x2B50, 0x2B50),
    (0x2B55, 0x2B55),
    (0x2E80, 0x303E),
    (0x3041, 0x33FF),
    (0x3400, 0x4DBF),
    (0x4E00, 0x9FFF),
    (0xA000, 0xA4CF),
    (0xAC00, 0xD7A3),
    (0xF900, 0xFAFF),
    (0xFE30, 0xFE4F),
    (0xFF00, 0xFF60),
    (0xFFE0, 0xFFE6),
    (0x1F300, 0x1F64F),
    (0x1F680, 0x1F6FF),
    (0x1F900, 0x1F9FF),
    (0x1FA70, 0x1FAFF),
    (0x20000, 0x3FFFD),
];

const ZERO_WIDTH_JOINER: char = '\u{200D}';
const EMOJI_PRESENTATION: char = '\u{FE0F}';

/// The columns `c` takes on its own: none for a combining mark or a joiner, two for a wide character, else one.
fn char_width(c: char) -> usize {
    let zero = matches!(
        c.general_category(),
        GeneralCategory::NonspacingMark | GeneralCategory::EnclosingMark
    ) || c == ZERO_WIDTH_JOINER
        || ('\u{FE00}'..=EMOJI_PRESENTATION).contains(&c);
    if zero {
        return 0;
    }
    let code = u32::from(c);
    let wide = WIDE.binary_search_by(|&(lo, hi)| {
        if hi < code {
            Ordering::Less
        } else if lo > code {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
    if wide.is_ok() { 2 } else { 1 }
}

/// The columns `text` takes. A character followed by U+FE0F, which asks for its emoji form, takes two.
pub(super) fn display_width(text: &str) -> usize {
    let mut width = 0;
    let mut previous = None;
    for c in text.chars() {
        if c == EMOJI_PRESENTATION && previous.is_some_and(|previous| char_width(previous) == 1) {
            width += 1;
        }
        width += char_width(c);
        previous = Some(c);
    }
    width
}
