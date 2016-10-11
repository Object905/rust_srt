use std::ops::{Add, Sub, AddAssign, SubAssign};
use std::convert::From;



#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub miliseconds: u32,
}

impl Timestamp {
    /// Constructs new Timestamp, equalizing values.
    /// That means you can construct Timestamp's from miliseconds, seconds or minutes only.
    ///
    /// # Examples
    ///
    /// ```
    /// use srt::Timestamp;
    ///
    /// let t1 = Timestamp::new(1, 120, 120, 1000);
    /// let t2 = Timestamp::new(3, 2, 1, 0);
    /// assert_eq!(t1, t2);
    ///
    /// let t3 = Timestamp::new(0, 0, 3, 600000);
    /// let t4 = Timestamp::new(0, 10, 3, 0);
    /// assert_eq!(t3, t4);
    /// ```
    pub fn new(mut hours: u32, mut minutes: u32, mut seconds: u32, mut miliseconds: u32) -> Timestamp {

        if miliseconds >= 1000 {
            let to_seconds = miliseconds / 1000;
            seconds += to_seconds;
            miliseconds -= to_seconds * 1000;   
        }

        if seconds >= 60 {
            let to_minutes = seconds / 60;
            minutes += to_minutes;
            seconds -= to_minutes * 60;
        }

        if minutes >= 60 {
            let to_hours = minutes / 60;
            hours += to_hours;
            minutes -= to_hours * 60;
        }

        Timestamp {
            hours: hours,
            minutes: minutes,
            seconds: seconds,
            miliseconds: miliseconds,
        }
    }
    /// Constructs new Timestamp from given overall microseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use srt::Timestamp;
    ///
    /// let t1 = Timestamp::from_microseconds(61001000);
    /// let t2 = Timestamp::new(0, 1, 1, 1);
    /// assert_eq!(t1, t2);
    /// ```
    pub fn from_microseconds(microseconds: u64) -> Timestamp {
        let miliseconds = microseconds / 1000;
        Timestamp::new(0, 0, 0, miliseconds as u32)
    }

    pub fn total_miliseconds(&self) -> u64 {
        let mut result: u64 = 0;

        result += self.miliseconds as u64;
        result += (self.seconds as u64) * 1_000;
        result += (self.minutes as u64) * 60_000;
        result += (self.hours as u64) * 360_000;
        result
    }
}

impl<'a> From<&'a [u32; 4]> for Timestamp {
    /// Make timestamp from &[hours, minutes, seconds, miliseconds]
    ///
    /// # Examples
    ///
    /// ```
    /// use srt::Timestamp;
    /// 
    /// let timestamp_array = [1337, 1, 1, 1];
    /// let timestamp = Timestamp::from(&timestamp_array);
    /// assert_eq!(timestamp_array[0], timestamp.hours);
    /// ```
    fn from(timestamp: &'a [u32; 4]) -> Timestamp {
        Timestamp::new(timestamp[0], timestamp[1], timestamp[2], timestamp[3])
    }
}

impl Add for Timestamp {
    type Output = Timestamp;

    fn add(self, other: Timestamp) -> Timestamp {
        let mut miliseconds = (self.miliseconds + other.miliseconds) as u64;
        let mut seconds = (self.seconds + other.seconds) as u64;
        let mut minutes = (self.minutes + other.minutes) as u64;
        let mut hours = (self.hours + other.hours) as u64;

        if miliseconds >= 1000 {
            let to_seconds = miliseconds / 1000;
            seconds += to_seconds;
            miliseconds -= to_seconds * 1000;   
        }

        if seconds >= 60 {
            let to_minutes = seconds / 60;
            minutes += to_minutes;
            seconds -= to_minutes * 60;
        }

        if minutes >= 60 {
            let to_hours = minutes / 60;
            hours += to_hours;
            minutes -= to_hours * 60;
        }

        Timestamp {
            hours: hours as u32,
            minutes: minutes as u32,
            seconds: seconds as u32,
            miliseconds: miliseconds as u32,
        }
    }
}

impl AddAssign for Timestamp {
    fn add_assign(&mut self, timestamp: Timestamp) {
        self.hours += timestamp.hours;
        self.minutes += timestamp.minutes;
        self.seconds += timestamp.seconds;
        self.miliseconds += timestamp.miliseconds;

        if self.miliseconds >= 1000 {
            let to_seconds = self.miliseconds / 1000;
            self.seconds += to_seconds;
            self.miliseconds -= to_seconds * 1000;   
        }

        if self.seconds >= 60 {
            let to_minutes = self.seconds / 60;
            self.minutes += to_minutes;
            self.seconds -= to_minutes * 60;
        }

        if self.minutes >= 60 {
            let to_hours = self.minutes / 60;
            self.hours += to_hours;
            self.minutes -= to_hours * 60;
        }
    }
}

impl Sub for Timestamp {
    type Output = Timestamp;

    /// # Panics
    ///
    /// Panics if self < other, timestamp can't be negative.
    fn sub(self, other: Timestamp) -> Timestamp {
        if self < other {
            panic!("attempt to subtract with overflow, timestamp can't be negative");
        }
        let mut hours = self.hours - other.hours;

        let mut minutes = self.minutes;
        if minutes < other.minutes {
            // Convert 1 hour to minutes
            minutes += 60;
            hours -= 1;
        }
        minutes -= other.minutes;

        let mut seconds = self.seconds;
        if seconds < other.seconds {
            // Conver 1 minute to seconds
            if minutes == 0 {
                // Conver 1 hour in minutes
                minutes += 60;
                hours -= 1;
            }
            seconds += 60;
            minutes -= 1;
        }
        seconds -= other.seconds;

        let mut miliseconds = self.miliseconds;
        if miliseconds < other.miliseconds {
            // Conver 1 seconds to miliseconds
            if seconds == 0 {
                // Conver 1 minute to seconds
                if minutes == 0 {
                    // Conver 1 hour to minutes
                    minutes += 60;
                    hours -= 1;
                }
                seconds += 60;
                minutes -= 1;
            }
            miliseconds += 1000;
            seconds -= 1;
        }
        miliseconds -= other.miliseconds;

        Timestamp {
            hours: hours,
            minutes: minutes,
            seconds: seconds,
            miliseconds: miliseconds,
        }
    }
}

impl SubAssign for Timestamp {
    /// # Panics
    ///
    /// Panics if self < other, timestamp can't be negative.
    fn sub_assign(&mut self, mut other: Timestamp) {
        if self < &mut other {
            panic!("attempt to subtract with overflow, timestamp can't be negative");
        }

        self.hours -= other.hours;

        if self.minutes < other.minutes {
            // Convert 1 hour to minutes
            self.minutes += 60;
            self.hours -= 1;
        }
        self.minutes -= other.minutes;

        if self.seconds < other.seconds {
            // Conver 1 minute to seconds
            if self.minutes == 0 {
                // Conver 1 hour in minutes
                self.minutes += 60;
                self.hours -= 1;
            }
            self.seconds += 60;
            self.minutes -= 1;
        }
        self.seconds -= other.seconds;

        if self.miliseconds < other.miliseconds {
            // Conver 1 seconds to miliseconds
            if self.seconds == 0 {
                // Conver 1 minute to seconds
                if self.minutes == 0 {
                    // Conver 1 hour to minutes
                    self.minutes += 60;
                    self.hours -= 1;
                }
                self.seconds += 60;
                self.minutes -= 1;
            }
            self.miliseconds += 1000;
            self.seconds -= 1;
        }
        self.miliseconds -= other.miliseconds;
    }
}


#[cfg(test)]
mod timestamp_test {
    use super::*;

    #[test]
    fn ord() {
        let mut t1 = Timestamp::new(1, 1, 1, 1);
        let t2 = t1.clone();

        { // t1 == t2 
            assert!(t1 == t2);
            assert!(!(t1 != t2));

            assert!(t1 >= t2);
            assert!(t1 <= t2);

            assert!(!(t1 > t2));
            assert!(!(t1 < t2));
        }

        t1.miliseconds += 1;
        { // t1 > t2
            assert!(!(t1 == t2));
            assert!(t1 != t2);

            assert!(t1 >= t2);
            assert!(!(t1 <= t2));

            assert!(t1 > t2);
            assert!(!(t1 < t2));
        }

        t1.miliseconds -= 2;
        { // t1 < t2
            assert!(!(t1 == t2));
            assert!(t1 != t2);

            assert!(!(t1 >= t2));
            assert!(t1 <= t2);

            assert!(!(t1 > t2));
            assert!(t1 < t2);
        }
    }

    #[test]
    fn add() {
        {
            let t1 = Timestamp::new(1, 1, 1, 1);
            let t2 = t1.clone();

            let t3 = Timestamp::new(2, 2, 2, 2);
            assert_eq!(t1 + t2, t3);
        }

        {
            let t1 = Timestamp::new(1, 58, 58, 900);
            let t2 = t1.clone();

            let t3 = Timestamp::new(3, 57, 57, 800);
            assert_eq!(t1 + t2, t3);
        }

        {
            let t1 = Timestamp::new(0, 59, 59, 999);
            let t2 = Timestamp::new(0, 0, 0, 1);

            let t3 = Timestamp::new(1, 0, 0, 0);
            assert_eq!(t1 + t2, t3);
        }
    }

    #[test]
    fn add_assign() {
        {
            let mut t1 = Timestamp::new(1, 1, 1, 1);
            let t2 = Timestamp::new(2, 2, 2, 2);
            t1 += t1;

            assert_eq!(t1, t2);
        }
        {
            let mut t1 = Timestamp::new(0, 59, 59, 999);
            let t2 = Timestamp::new(0, 0, 0, 1);
            t1 += t2;

            let t3 = Timestamp::new(1, 0, 0, 0);
            assert_eq!(t1, t3);
        }
    }

    #[test]
    fn new() {
        {
            let t1 = Timestamp::new(1, 121, 120, 1100);
            let t2 = Timestamp::new(3, 3, 1, 100);
            assert_eq!(t1, t2);
        }
        {
            let t1 = Timestamp::new(1, 120, 120, 1000);
            let t2 = Timestamp::new(3, 2, 1, 0);
            assert_eq!(t1, t2);
        }
        {
            let t1 = Timestamp::new(0, 0, 3, 600000);
            let t2 = Timestamp::new(0, 10, 3, 0);
            assert_eq!(t1, t2);
        }
    }

    #[test]
    fn sub() {
        {
            let t1 = Timestamp::new(1, 1, 1, 0);
            let t2 = Timestamp::new(0, 59, 59, 999);

            let t3 = Timestamp::new(0, 1, 1, 1);
            assert_eq!(t1 - t2, t3);
        }
        {
            let t1 = Timestamp::new(2, 2, 2, 2);
            let t2 = Timestamp::new(1, 1, 1, 1);

            let t3 = t2.clone();
            assert_eq!(t1 - t2, t3);
        }
        {
            let t1 = Timestamp::new(2, 0, 0, 0);
            let t2 = Timestamp::new(1, 59, 59, 999);

            let t3 = Timestamp::new(0, 0, 0, 1);
            assert_eq!(t1 - t2, t3);
        }
    }

    #[test]
    fn sub_assign() {
        {
            let mut t1 = Timestamp::new(1, 1, 1, 0);
            let t2 = Timestamp::new(0, 59, 59, 999);
            t1 -= t2;

            let t3 = Timestamp::new(0, 1, 1, 1);
            assert_eq!(t1, t3);
        }
        {
            let mut t1 = Timestamp::new(2, 2, 2, 2);
            let t2 = Timestamp::new(1, 1, 1, 1);
            t1 -= t2;

            assert_eq!(t1, t2);
        }
        {
            let mut t1 = Timestamp::new(2, 0, 0, 0);
            let t2 = Timestamp::new(1, 59, 59, 999);
            t1 -= t2;

            let t3 = Timestamp::new(0, 0, 0, 1);
            assert_eq!(t1, t3);
        }
    }

    #[test]
    fn from_microseconds() {
        let t1 = Timestamp::new(0, 1, 1, 1);
        let t2 = Timestamp::new(0, 0, 0, 61001);
        let t3 = Timestamp::from_microseconds(61001000);

        assert_eq!(t1, t2);
        assert_eq!(t2, t3);
        assert_eq!(t1, t3);
    }
}