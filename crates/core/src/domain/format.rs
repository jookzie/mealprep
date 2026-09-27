use time::{Date, format_description::BorrowedFormatItem, macros::format_description};

use crate::{Entity, Error, Result};

const ISO_DATE: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");

time::serde::format_description!(pub(super) iso_date, Date, "[year]-[month]-[day]");

/// Parses the given `YYYY-MM-DD` text, the form a date takes at every boundary.
pub fn parse_iso_date(value: &str) -> Result<Date> {
    Date::parse(value, ISO_DATE).map_err(|source| Error::Invalid {
        entity: Entity::CalendarDay,
        reason: format!("cannot read {value:?} as a YYYY-MM-DD date: {source}"),
    })
}

/// Formats the given date as `YYYY-MM-DD`, the form it is stored and sent in.
pub fn format_iso_date(date: Date) -> String {
    date.format(ISO_DATE)
        .expect("internal error: a date always formats as year, month and day")
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    #[test]
    fn round_trips_iso_dates() {
        let parsed = parse_iso_date("2026-09-07").unwrap();
        assert_eq!(parsed, date!(2026 - 09 - 07));
        assert_eq!(format_iso_date(parsed), "2026-09-07");
    }

    #[test]
    fn rejects_impossible_dates() {
        assert!(parse_iso_date("2026-02-30").is_err());
        assert!(parse_iso_date("07/09/2026").is_err());
    }
}
