use std::fmt;

use chrono::Duration;

#[derive(Debug, PartialEq)]
pub struct WorkTime {
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
}

static WORK_DAY: i64 = 8;

impl WorkTime {
    pub fn new(duration: Duration) -> Self {
        let days = duration.num_days();
        let duration : Duration = duration - Duration::hours(24 - WORK_DAY) * days as i32;
        let days = duration.num_hours() / WORK_DAY;
        let duration = duration - Duration::hours(days * WORK_DAY);
        let hours = duration.num_hours();
        let duration = duration - Duration::hours(hours);
        let minutes = duration.num_minutes();

        Self {
            days,
            hours,
            minutes,
        }
    }

    pub fn from_seconds(seconds: i64) -> Self {
        Self::new(Duration::seconds(seconds))
    }

    pub fn total_hours(&self) -> i64 {
        WORK_DAY * self.days + self.hours
    }
}

impl fmt::Display for WorkTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} hours {} minutes", self.total_hours(), self.minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_work_time() {
        // arrange
        let duration = Duration::minutes(90);

        // act
        let time = WorkTime::new(duration);

        // assert
        assert_eq!(time, WorkTime { days: 0, hours: 1, minutes: 30 });
    }

    #[test]
    fn create_work_day() {
        // arrange
        let duration = Duration::days(1);

        // act
        let time = WorkTime::new(duration);

        // assert
        assert_eq!(time, WorkTime { days: 1, hours: 0, minutes: 0 });
    }

    #[test]
    fn work_week_should_have_40_hours() {
        // arrange
        let duration = Duration::days(5);

        // act
        let time = WorkTime::new(duration);

        // assert
        assert_eq!(time.total_hours(), 40);
    }

    #[test]
    fn create_more_than_work_day() {
        // arrange
        let duration = Duration::hours(9);

        // act
        let time = WorkTime::new(duration);

        // assert
        assert_eq!(time, WorkTime { days: 1, hours: 1, minutes: 0 });
    }

    #[test]
    fn create_empty_work_time() {
        // arrange
        let duration = Duration::seconds(20);

        // act
        let time = WorkTime::new(duration);

        // assert
        assert_eq!(time, WorkTime { days: 0,  hours: 0, minutes: 0 });
    }
}