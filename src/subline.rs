use std::fmt::{self, Display, Formatter};

use timestamp::Timestamp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubLine {
    pub index: u32,
    pub start: Timestamp,
    pub end: Timestamp,
    pub text: String,
}

impl Display for SubLine {
    /// Formats ```SubLine``` according srt subtitles format.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,
               "{index}\r\n{s_h:02}:{s_m:02}:{s_s:02},{s_ms:03} --> \
                {e_h:02}:{e_m:02}:{e_s:02},{e_ms:03}\r\n{text}\r\n\r\n",
               index = self.index,
               s_h = self.start.hours,
               s_m = self.start.minutes,
               s_s = self.start.seconds,
               s_ms = self.start.miliseconds,
               e_h = self.end.hours,
               e_m = self.end.minutes,
               e_s = self.end.seconds,
               e_ms = self.end.miliseconds,
               text = self.text)
    }
}

impl SubLine {
    /// Get duration (```self.end - self.start```)
    /// # Panics
    ///
    /// Panics if ```self.start``` is bigger than ```self.end```.
    pub fn duration(&self) -> Timestamp {
        self.end - self.start
    }
    /// Constructs a new ```SubLine```.
    ///
    /// # Panics
    ///
    /// Panics if ```start``` timestamp is bigger than the ```end``` timestamp.
    pub fn new(index: u32, text: String, start: Timestamp, end: Timestamp) -> SubLine {
        if start > end {
            panic!("start timestamp is bigger than the end timestamp");
        }

        SubLine {
            index: index,
            text: text,
            start: start,
            end: end,
        }
    }
}

#[cfg(test)]
mod subline_tests {
    use super::*;
    use timestamp::Timestamp;
    #[test]
    fn display() {
        let subline = SubLine {
            text: "Some text lalala".to_owned(),
            index: 1,
            start: Timestamp::new(0, 55, 9, 8),
            end: Timestamp::new(1, 1, 1, 1),
        };

        let in_text = "1\r\n00:55:09,008 --> 01:01:01,001\r\nSome text lalala\r\n\r\n".to_owned();

        assert_eq!(format!("{}", subline), in_text);
    }
}
