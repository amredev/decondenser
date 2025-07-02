use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects};

// Borrowed from `clap-cargo`:
// https://github.com/crate-ci/clap-cargo/blob/v0.15.2/src/style.rs#L8-L17
pub(crate) const CLI_STYLES: Styles = Styles::styled()
    .header(
        AnsiColor::Green
            .on_default()
            .effects(Effects::DOTTED_UNDERLINE),
    )
    .usage(
        AnsiColor::Green
            .on_default()
            .effects(Effects::DOTTED_UNDERLINE),
    )
    .literal(AnsiColor::White.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::White.on_default())
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::White.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD));
