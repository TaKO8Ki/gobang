use chrono::{DateTime, Local, NaiveDateTime, Utc};
use unicode_width::UnicodeWidthStr;

pub mod scroll_vertical;

/// macro to simplify running code that might return Err.
/// It will show a popup in that case
#[macro_export]
macro_rules! try_or_popup {
    ($self:ident, $msg:literal, $e:expr) => {
        if let Err(err) = $e {
            ::log::error!("{} {}", $msg, err);
            $self
                .queue
                .push(InternalEvent::ShowErrorMsg(format!("{}\n{}", $msg, err)));
        }
    };
}
