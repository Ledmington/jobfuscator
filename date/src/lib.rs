#![forbid(unsafe_code)]

use std::time::{SystemTime, UNIX_EPOCH};

pub struct Date {
    day: u8,
    month: u8,
    year: u16,
}

fn get_max_days_in_month(month: u8, year: u16) -> u8 {
    match month {
        0 => 31,
        1 => {
            if year % 4 == 0 && year % 100 != 0 {
                29
            } else {
                28
            }
        }
        2 => 31,
        3 => 30,
        4 => 31,
        5 => 30,
        6 => 31,
        7 => 31,
        8 => 30,
        9 => 31,
        10 => 30,
        11 => 31,
        _ => unreachable!(),
    }
}

impl From<SystemTime> for Date {
    fn from(st: SystemTime) -> Self {
        let seconds: u64 = st.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let days: u64 = seconds / 60 / 60 / 24;

        let mut day: u8 = 0;
        let mut month: u8 = 0;
        let mut year: u16 = 1970;
        let mut total_days: u64 = 0;

        while total_days < days {
            day += 1;
            total_days += 1;

            let days_in_current_month = get_max_days_in_month(month, year);
            assert!(day <= days_in_current_month);
            if day == days_in_current_month {
                day = 0;
                month += 1;
                if month == 12 {
                    month = 0;
                    year += 1;
                }
            }
        }

        Date { day, month, year }
    }
}

impl Date {
    pub fn day(self: &Self) -> String {
        self.day.to_string()
    }

    pub fn month(self: &Self) -> String {
        match self.month {
            0 => "Jan".to_owned(),
            1 => "Feb".to_owned(),
            2 => "Mar".to_owned(),
            3 => "Apr".to_owned(),
            4 => "May".to_owned(),
            5 => "Jun".to_owned(),
            6 => "Jul".to_owned(),
            7 => "Aug".to_owned(),
            8 => "Sep".to_owned(),
            9 => "Oct".to_owned(),
            10 => "Nov".to_owned(),
            11 => "Dec".to_owned(),
            _ => unreachable!(),
        }
    }

    pub fn year(self: &Self) -> String {
        self.year.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn conversions() {
        let cases = [
            (
                UNIX_EPOCH + Duration::new(1776280712, 866753383),
                "Apr 15, 2026",
            ),
            (
                UNIX_EPOCH + Duration::new(1327622400, 867753386),
                "Jan 27, 2012",
            ),
        ];
        for (input, expected) in cases {
            let actual = Date::from(input);
            let actual_string = format!("{} {}, {}", actual.month(), actual.day(), actual.year());
            assert_eq!(expected, actual_string);
        }
    }
}
