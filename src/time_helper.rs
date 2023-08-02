use chrono::{DateTime, Datelike, Duration, Timelike};
use std::ops::Add;

pub fn now() -> DateTime<chrono_tz::Tz> {
    chrono::Utc::now().with_timezone(&chrono_tz::Europe::Amsterdam)
}

pub trait TimeHelpers {
    fn next_weekday(self, weekday: chrono::Weekday) -> Self;
    fn next_invitation_time(self) -> Self;
    fn next_session_date(self) -> Self;
}

impl TimeHelpers for DateTime<chrono_tz::Tz> {
    fn next_weekday(self, weekday: chrono::Weekday) -> DateTime<chrono_tz::Tz> {
        let mut time = self;
        if time.weekday() == weekday {
            time = time.add(Duration::days(1))
        }
        while time.weekday() != weekday {
            time = time.add(Duration::days(1))
        }
        time
    }

    fn next_invitation_time(self) -> DateTime<chrono_tz::Tz> {
        let now = self;
        let time = now.next_weekday(chrono::Weekday::Tue);
        let time = time.with_hour(10).unwrap();
        let time = time.with_minute(0).unwrap();
        let time = time.with_second(0).unwrap();
        let time = time.with_nanosecond(0).unwrap();
        if time > now {
            time
        } else {
            time.add(Duration::days(1))
        }
    }

    fn next_session_date(self) -> DateTime<chrono_tz::Tz> {
        self.next_weekday(chrono::Weekday::Thu)
            .add(Duration::weeks(2))
            .with_hour(19)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::TimeHelpers;

    #[test]
    fn next_weekday() {
        let now = chrono_tz::Europe::Amsterdam
            .with_ymd_and_hms(2023, 8, 2, 10, 0, 0)
            .unwrap();
        let invitation_time = now.next_weekday(chrono::Weekday::Thu);
        assert_eq!(
            invitation_time,
            chrono_tz::Europe::Amsterdam
                .with_ymd_and_hms(2023, 8, 3, 10, 0, 0)
                .unwrap()
        );

        let now = chrono_tz::Europe::Amsterdam
            .with_ymd_and_hms(2023, 8, 3, 10, 0, 0)
            .unwrap();
        let invitation_time = now.next_weekday(chrono::Weekday::Thu);
        assert_eq!(
            invitation_time,
            chrono_tz::Europe::Amsterdam
                .with_ymd_and_hms(2023, 8, 10, 10, 0, 0)
                .unwrap()
        );
    }

    #[test]
    fn next_invitation() {
        let now = chrono_tz::Europe::Amsterdam
            .with_ymd_and_hms(2023, 7, 30, 12, 12, 12)
            .unwrap();
        let invitation_time = now.next_invitation_time();
        assert_eq!(
            invitation_time,
            chrono_tz::Europe::Amsterdam
                .with_ymd_and_hms(2023, 8, 1, 10, 0, 0)
                .unwrap()
        );

        let now = chrono_tz::Europe::Amsterdam
            .with_ymd_and_hms(2023, 8, 1, 10, 0, 1)
            .unwrap();
        let invitation_time = now.next_invitation_time();
        assert_eq!(
            invitation_time,
            chrono_tz::Europe::Amsterdam
                .with_ymd_and_hms(2023, 8, 8, 10, 0, 0)
                .unwrap()
        );
    }

    #[test]
    fn next_session() {
        let now = chrono_tz::Europe::Amsterdam
            .with_ymd_and_hms(2023, 8, 2, 10, 0, 0)
            .unwrap();
        assert_eq!(
            now.next_session_date(),
            chrono_tz::Europe::Amsterdam
                .with_ymd_and_hms(2023, 8, 17, 19, 0, 0)
                .unwrap()
        )
    }
}
