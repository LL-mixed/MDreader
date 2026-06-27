// DateBuckets — port of macOS DateBuckets.swift.
// Buckers a timestamp into 今天/昨天/更早 using the LOCAL calendar (start-of-day comparison),
// and formats timestamps as "yyyy/MM/dd HH:mm".

use chrono::{Local, NaiveTime, TimeZone};

const DAY_MILLIS: i64 = 86_400_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DayBucket {
    Today,
    Yesterday,
    Earlier,
}

impl DayBucket {
    pub fn title(self) -> &'static str {
        match self {
            DayBucket::Today => "今天",
            DayBucket::Yesterday => "昨天",
            DayBucket::Earlier => "更早",
        }
    }

    /// macOS DayBucket.allCases ordering (used to render sections in this order).
    pub fn all() -> [DayBucket; 3] {
        [DayBucket::Today, DayBucket::Yesterday, DayBucket::Earlier]
    }
}

fn start_of_day_local(millis: i64) -> i64 {
    let dt = Local.timestamp_millis_opt(millis).unwrap();
    let midnight = dt.date_naive().and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    Local.from_local_datetime(&midnight).unwrap().timestamp_millis()
}

pub fn bucket(date_millis: i64, now_millis: i64) -> DayBucket {
    let today_start = start_of_day_local(now_millis);
    let item_start = start_of_day_local(date_millis);
    if item_start >= today_start {
        DayBucket::Today
    } else if item_start >= today_start - DAY_MILLIS {
        DayBucket::Yesterday
    } else {
        DayBucket::Earlier
    }
}

pub fn format(millis: i64) -> String {
    let dt = Local.timestamp_millis_opt(millis).unwrap();
    dt.format("%Y/%m/%d %H:%M").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now_at_1300() -> i64 {
        let today = Local::now().date_naive();
        let dt = today.and_hms_opt(13, 0, 0).unwrap();
        Local.from_local_datetime(&dt).unwrap().timestamp_millis()
    }

    #[test]
    fn same_day_is_today() {
        let now = now_at_1300();
        assert_eq!(bucket(now, now), DayBucket::Today);
        // 00:05 today is still today
        let today = Local::now().date_naive();
        let early = today.and_hms_opt(0, 5, 0).unwrap();
        let early_millis = Local.from_local_datetime(&early).unwrap().timestamp_millis();
        assert_eq!(bucket(early_millis, now), DayBucket::Today);
    }

    #[test]
    fn previous_day_is_yesterday() {
        let now = now_at_1300();
        assert_eq!(bucket(now - DAY_MILLIS, now), DayBucket::Yesterday);
    }

    #[test]
    fn three_days_ago_is_earlier() {
        let now = now_at_1300();
        assert_eq!(bucket(now - 3 * DAY_MILLIS, now), DayBucket::Earlier);
    }

    #[test]
    fn format_produces_non_empty_string_with_slash() {
        let now = now_at_1300();
        let s = format(now - (9 * 3600_000 + 30 * 60_000)); // 9:30 today-ish
        assert!(!s.is_empty());
        assert!(s.contains('/'));
    }
}
