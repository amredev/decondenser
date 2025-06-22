//! Adapted from <https://github.com/nik-rev/simply-colored/blob/77b3c4c2df572494992cecb59d420e7e0569a8a9/src/lib.rs>
#![expect(
    dead_code,
    reason = "This is just a bag of constants that we may or may not use. \
        The list of used constants can change any time, so unused items may \
        suddenly become needed"
)]

/// Reset styling
pub(crate) const RESET: &str = "\x1b[0m";

/// Following text will be bold
pub(crate) const BOLD: &str = "\x1b[1m";
/// Following text will NOT be bold
pub(crate) const NO_BOLD: &str = "\x1b[22m";

/// Following text will be dim
pub(crate) const DIM: &str = "\x1b[2m";
/// Following text will NOT be dim
pub(crate) const NO_DIM: &str = "\x1b[22m";

/// Following text will be italic
pub(crate) const ITALIC: &str = "\x1b[3m";
/// Following text will NOT be italic
pub(crate) const NO_ITALIC: &str = "\x1b[23m";

/// Following text will be underlined
pub(crate) const UNDERLINE: &str = "\x1b[4m";
/// Following text will NOT be underlined
pub(crate) const NO_UNDERLINE: &str = "\x1b[24m";

/// Following text will be blinking
pub(crate) const BLINK: &str = "\x1b[5m";
/// Following text will NOT be blinking
pub(crate) const NO_BLINK: &str = "\x1b[25m";

/// Foreground and background for the following text will be reversed
pub(crate) const REVERSE: &str = "\x1b[7m";
/// Foreground and background for the following text will NOT be reversed
pub(crate) const NO_REVERSE: &str = "\x1b[27m";

/// Following text will be invisible
pub(crate) const HIDE: &str = "\x1b[8m";
/// Following text will be visible
pub(crate) const NO_HIDE: &str = "\x1b[28m";

/// Following text will be crossed out
pub(crate) const STRIKETHROUGH: &str = "\x1b[9m";
/// Following text will NOT be crossed out
pub(crate) const NO_STRIKETHROUGH: &str = "\x1b[29m";

/// Set color of text to black
pub(crate) const BLACK: &str = "\x1b[90m";
/// Set background of text to black
pub(crate) const BG_BLACK: &str = "\x1b[100m";
/// Set color of text to dim black
pub(crate) const DIM_BLACK: &str = "\x1b[30m";
/// Set background of text to dim black
pub(crate) const BG_DIM_BLACK: &str = "\x1b[40m";

/// Set color of text to red
pub(crate) const RED: &str = "\x1b[91m";
/// Set background of text to red
pub(crate) const BG_RED: &str = "\x1b[101m";
/// Set color of text to dim red
pub(crate) const DIM_RED: &str = "\x1b[31m";
/// Set background of text to dim red
pub(crate) const BG_DIM_RED: &str = "\x1b[41m";

/// Set color of text to green
pub(crate) const GREEN: &str = "\x1b[92m";
/// Set background of text to green
pub(crate) const BG_GREEN: &str = "\x1b[102m";
/// Set color of text to dim green
pub(crate) const DIM_GREEN: &str = "\x1b[32m";
/// Set background of text to dim green
pub(crate) const BG_DIM_GREEN: &str = "\x1b[42m";

/// Set color of text to yellow
pub(crate) const YELLOW: &str = "\x1b[93m";
/// Set background of text to yellow
pub(crate) const BG_YELLOW: &str = "\x1b[103m";
/// Set color of text to dim yellow
pub(crate) const DIM_YELLOW: &str = "\x1b[33m";
/// Set background of text to dim yellow
pub(crate) const BG_DIM_YELLOW: &str = "\x1b[43m";

/// Set color of text to blue
pub(crate) const BLUE: &str = "\x1b[94m";
/// Set background of text to blue
pub(crate) const BG_BLUE: &str = "\x1b[104m";
/// Set color of text to dim blue
pub(crate) const DIM_BLUE: &str = "\x1b[34m";
/// Set background of text to dim blue
pub(crate) const BG_DIM_BLUE: &str = "\x1b[44m";

/// Set color of text to magenta
pub(crate) const MAGENTA: &str = "\x1b[95m";
/// Set background of text to magenta
pub(crate) const BG_MAGENTA: &str = "\x1b[105m";
/// Set color of text to dim magenta
pub(crate) const DIM_MAGENTA: &str = "\x1b[35m";
/// Set background of text to dim magenta
pub(crate) const BG_DIM_MAGENTA: &str = "\x1b[45m";

/// Set color of text to cyan
pub(crate) const CYAN: &str = "\x1b[96m";
/// Set background of text to cyan
pub(crate) const BG_CYAN: &str = "\x1b[106m";
/// Set color of text to dim cyan
pub(crate) const DIM_CYAN: &str = "\x1b[36m";
/// Set background of text to dim cyan
pub(crate) const BG_DIM_CYAN: &str = "\x1b[46m";

/// Set color of text to white
pub(crate) const WHITE: &str = "\x1b[97m";
/// Set background of text to white
pub(crate) const BG_WHITE: &str = "\x1b[107m";
/// Set color of text to dim white
pub(crate) const DIM_WHITE: &str = "\x1b[37m";
/// Set background of text to dim white
pub(crate) const BG_DIM_WHITE: &str = "\x1b[47m";

/// Set color of text to default
pub(crate) const DEFAULT: &str = "\x1b[99m";
/// Set background of text to default
pub(crate) const BG_DEFAULT: &str = "\x1b[109m";
/// Set color of text to default
pub(crate) const DIM_DEFAULT: &str = "\x1b[39m";
/// Set background of text to default
pub(crate) const BG_DIM_DEFAULT: &str = "\x1b[49m";
