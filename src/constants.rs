use time::{macros::datetime, PrimitiveDateTime};

/// FIT base start time 1989-12-31T00:00:00.000.
pub const FIT_DEFAULT_DATETIME: PrimitiveDateTime = datetime!(1989-12-31 00:00:00.000);