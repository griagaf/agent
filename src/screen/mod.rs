mod ranking;
mod types;

#[cfg(windows)]
mod control_types;
#[cfg(not(windows))]
mod unsupported;
#[cfg(windows)]
mod windows;

pub use types::{Match, Rect, Target};

#[cfg(windows)]
pub use windows::{click, exists, find, text_of, type_text, windows};

#[cfg(not(windows))]
pub use unsupported::{click, exists, find, text_of, type_text, windows};
