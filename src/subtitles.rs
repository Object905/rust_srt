use std::str::FromStr;
use std::ops::Index;
use std::fs::File;
use std::io::{Error, Write, ErrorKind};
use std::path::Path;
use std::fmt::{self, Display, Formatter};

use timestamp::Timestamp;
use subline::SubLine;
use utils;

#[derive(Debug, Clone, PartialEq)]
pub struct Subtitles {
    pub inner: Vec<SubLine>,
}

impl Subtitles {
    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Construct ```Subtitles``` from given file path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Subtitles, Error> {
        let mut content = try!(utils::read_file(&path));
        content = utils::prepare(&content);

        if !utils::check(&content) {
            return Err(Error::new(ErrorKind::InvalidData,
                                  "Given file does not match with srt format specification"));
        }
        Ok(try!(Subtitles::from_str(&content)))
    }

    /// Saves ```Subtitles``` into given file path according srt subtitles format.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let mut file = try!(File::create(&path));
        for line in &self.inner {
            try!(write!(&mut file, "{}", line));
        }
        try!(write!(&mut file, "\r\n\r\n"));
        Ok(())
    }

    /// Get ```&SubLine``` by it's index.
    ///
    /// # Panics
    /// Panics if inner structure is broken.
    /// E.g. inner vector is not sorted or SubLine's indices is not consistent.
    pub fn by_index(&self, index: usize) -> Option<&SubLine> {
        let indexed_subline = self.inner.get(index - 1);

        indexed_subline.as_ref().map(|elem| {
            if elem.index != index as u32 {
                panic!("Subtitles's inner structure is broken, inner vector of Subtitles must always be \
            sorted and also unterlying SubLine's indices must be correct");
            }
        });

        indexed_subline
    }

    /// Get ```&mut SubLine``` by it's index.
    ///
    /// # Panics
    /// Panics if inner structure is broken.
    /// E.g. inner vector is not sorted or SubLine's indices is not consistent.
    pub fn by_index_mut(&mut self, index: usize) -> Option<&mut SubLine> {
        let indexed_subline = self.inner.get_mut(index - 1);

        indexed_subline.as_ref().map(|elem| {
            if elem.index != index as u32 {
                panic!("Subtitles's inner structure is broken, inner vector of Subtitles must always be \
            sorted and also unterlying SubLine's indices must be correct");
            }
        });
        indexed_subline
    }

    /// Get ```&SubLine``` for which given ```time```
    /// lies in the range ```start...end``` (inclusive).
    pub fn by_time(&self, time: Timestamp) -> Option<&SubLine> {
        let mut min = 0;
        let mut max = self.inner.len() - 1;
        let mut guess_index;

        while max >= min {
            guess_index = (max + min) / 2;
            let guess = self.inner.get(guess_index).unwrap();

            if (guess.start <= time) && (time <= guess.end) {
                return Some(&guess);
            } else if time > guess.end {
                min = guess_index + 1;
            } else {
                max = guess_index - 1;
            }
        }
        None
    }

    /// Get ```&mut SubLine``` for which given ```time```
    /// lies in the range ```start...end``` (inclusive).
    pub fn by_time_mut(&mut self, time: Timestamp) -> Option<&mut SubLine> {
        let mut min = 0;
        let mut max = self.inner.len() - 1;
        let mut guess_index;
        let mut result: usize = 0;
        let mut found = false;

        while max >= min {
            guess_index = (max + min) / 2;

            let guess = self.inner.get(guess_index).unwrap();

            if (guess.start <= time) && (time <= guess.end) {
                result = guess_index;
                found = true;
                break;
            } else if time > guess.end {
                min = guess_index + 1;
            } else {
                max = guess_index - 1;
            }
        }
        if found {
            self.inner.get_mut(result)
        } else {
            None
        }
    }

    /// Get ```&SubLine``` for which given ```time```
    /// lies in the range ```start..start_of_the_next_line``` (exclusive).
    pub fn nearest_by_time(&self, time: Timestamp) -> Option<&SubLine> {
        let mut min = 0;
        let mut max = self.inner.len() - 1;
        let mut guess_index;

        while max >= min {
            guess_index = (max + min) / 2;
            let guess = self.inner.get(guess_index).unwrap();

            if (guess.start <= time) && (time <= guess.end) {
                return Some(&guess);
            } else if time > guess.end {
                min = guess_index + 1;
            } else {
                max = guess_index - 1;
            }
        }
        self.inner.get(max)
    }

    /// Get ```&mut SubLine``` for which given ```time```
    /// lies in the range ```start..start_of_the_next_line``` (exclusive).
    pub fn nearest_by_time_mut(&mut self, time: Timestamp) -> Option<&mut SubLine> {
        let mut min = 0;
        let mut max = self.inner.len() - 1;
        let mut guess_index;
        let mut result: usize = 0;
        let mut found_exact = false;

        while max >= min {
            guess_index = (max + min) / 2;

            let guess = self.inner.get(guess_index).unwrap();

            if (guess.start <= time) && (time <= guess.end) {
                result = guess_index;
                found_exact = true;
                break;
            } else if time > guess.end {
                min = guess_index + 1;
            } else {
                max = guess_index - 1;
            }
        }
        if found_exact {
            self.inner.get_mut(result)
        } else {
            self.inner.get_mut(max)
        }
    }

    /// Pushes given line in the end.
    ///
    /// # Panics
    /// Panics if given ```line```'s index is not ```latest_sub_index + 1```.
    /// E.g. pushed line is going to break consistency.
    pub fn push(&mut self, line: SubLine) {
        let last_index: u32 = self.inner.last().map(|line| line.index).unwrap_or(0);
        if line.index != last_index + 1 {
            panic!("Pushed line is going to break consistency, invalid index");
        }
        self.inner.push(line);
    }

    /// Inserts given ```element``` into ```Subtitles```,
    /// shifting all its right element's indices.
    ///
    /// # Panics
    /// Panics if ```element``'s index is greater than the ```Subtitles``` length.
    pub fn insert(&mut self, element: SubLine) {
        let index = (element.index - 1) as usize;
        for line in &mut self.inner[index..] {
            line.index += 1;
        }
        
        self.inner.insert(index, element);
    }
    
    /// Removes the last element from a Subtitles and returns it, or None if it is empty.
    pub fn pop(&mut self) -> Option<SubLine> {
        self.inner.pop()
    }
}


impl FromStr for Subtitles {
    type Err = Error;
    /// Construct Subtitles from str.
    ///
    /// Given str must be properly formated:
    /// Newlne styles must be windows like (\r\n).
    /// And in the end of str must be exacly 4 newlines.
    fn from_str(content: &str) -> Result<Subtitles, Error> {
        let mut result = Vec::with_capacity(400);

        for cap in utils::SUBS.captures_iter(&content) {

            let index: u32 = cap.at(1).unwrap().parse().unwrap();

            let start_timestamp: [u32; 4] = [cap.at(2).unwrap().parse().unwrap(),
                                             cap.at(3).unwrap().parse().unwrap(),
                                             cap.at(4).unwrap().parse().unwrap(),
                                             cap.at(5).unwrap().parse().unwrap()];
            let end_timestamp: [u32; 4] = [cap.at(6).unwrap().parse().unwrap(),
                                           cap.at(7).unwrap().parse().unwrap(),
                                           cap.at(8).unwrap().parse().unwrap(),
                                           cap.at(9).unwrap().parse().unwrap()];

            let start = Timestamp::from(&start_timestamp);
            let end = Timestamp::from(&end_timestamp);

            let text = cap.at(10).unwrap().to_owned();

            let line = SubLine {
                index: index,
                text: text,
                start: start,
                end: end,
            };
            result.push(line);
        }
        result.shrink_to_fit();
        Ok(Subtitles::from(result))
    }
}

impl From<Vec<SubLine>> for Subtitles {
    fn from(vec: Vec<SubLine>) -> Subtitles {
        Subtitles { inner: vec }
    }
}

impl<'a> IntoIterator for &'a Subtitles {
    type Item = &'a SubLine;
    type IntoIter = ::std::slice::Iter<'a, SubLine>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a mut Subtitles {
    type Item = &'a mut SubLine;
    type IntoIter = ::std::slice::IterMut<'a, SubLine>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl IntoIterator for Subtitles {
    type Item = SubLine;
    type IntoIter = ::std::vec::IntoIter<SubLine>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl Display for Subtitles {
    /// Formats Subtitles according srt subtitles format.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for line in self {
            try!(write!(f, "{}", line));
        }
        write!(f, "\r\n\r\n")
    }
}

impl Index<usize> for Subtitles {
    type Output = SubLine;
    /// Indexes inner vector. That means that indexing start at 0.
    fn index<'a>(&'a self, index: usize) -> &'a Self::Output {
        &self.inner[index]
    }
}

#[cfg(test)]
mod subtitles_tests {
    use super::*;
    use subline::SubLine;
    use timestamp::Timestamp;
    use utils;
    use std::str::FromStr;

    static PATH: &'static str = "example.srt";

    lazy_static! {
        static ref SUBS: Subtitles = Subtitles::from_file(PATH).unwrap();
    }

    #[test]
    fn from_file() {
        let subs = Subtitles::from_file(PATH).unwrap();

        assert_eq!(subs.len(), 619);
        let latest_sub = SubLine {
            text: "Last".to_owned(),
            index: 619,
            start: Timestamp {
                hours: 1,
                minutes: 6,
                seconds: 40,
                miliseconds: 216,
            },
            end: Timestamp {
                hours: 1,
                minutes: 6,
                seconds: 50,
                miliseconds: 792,
            },
        };
        assert_eq!(&latest_sub, subs.by_index(619).unwrap());
    }

    #[test]
    fn from_str() {
        let mut sub_str = utils::read_file(PATH).unwrap();
        sub_str = utils::prepare(&sub_str);

        let subs = Subtitles::from_str(&sub_str).unwrap();
        let same_subs = Subtitles::from_file(PATH).unwrap();

        assert_eq!(subs, same_subs);
    }

    #[test]
    fn to_string() {
        let mut sub_str = utils::read_file(PATH).unwrap();
        sub_str = utils::prepare(&sub_str);

        let subs = Subtitles::from_str(&sub_str).unwrap();
        let same_subs = Subtitles::from_file(PATH).unwrap();

        sub_str += &"\r\n\r\n";
        assert_eq!(sub_str, subs.to_string());
        assert_eq!(sub_str, same_subs.to_string());
    }

    #[test]
    fn iterator() {
        let mut subs = SUBS.clone();
        let primal_subs = subs.clone();

        for line in &subs {
            let _: &SubLine = line;
        }
        let _: &SubLine = subs.by_index(1).unwrap();

        for line in &mut subs {
            let _: &mut SubLine = line;
            line.text += " lol";
            let offset = Timestamp::new(1, 1, 30, 500);
            line.end += offset;
            line.start += offset;
        }
        println!("{:?}", subs.by_index(1).unwrap().start);
        assert!(subs.by_index(3).unwrap().text.ends_with(" lol"));
        assert!(subs.by_index(10).unwrap().text.ends_with(" lol"));

        let mut new_subs_vec = Vec::with_capacity(subs.len());

        for mut line in subs {
            line.text = line.text.trim_right_matches(" lol").to_owned();
            let offset = Timestamp::new(1, 1, 30, 500);
            line.end -= offset;
            line.start -= offset;
            new_subs_vec.push(line);
        }
        let new_subs = Subtitles::from(new_subs_vec);

        assert_eq!(new_subs, primal_subs);
    }

    #[test]
    fn insert() {
        let mut subs = SUBS.clone();

        let sub14 = subs.by_index(14).unwrap().clone();
        let mut sub15 = subs.by_index(15).unwrap().clone();

        let one_ms = Timestamp::new(0, 0, 0, 1);
        let start = sub14.end + one_ms;
        let end = sub15.start - one_ms;

        let line = SubLine::new(15, "foo bar foo".to_owned(), start, end);
        subs.insert(line.clone());
        
        let newline = subs.by_index(15).unwrap();
        assert_eq!(newline, &line);

        // sub15's index shifted becomes 16
        sub15.index += 1;
        let new_sub16 = subs.by_index(16).unwrap();
        assert_eq!(new_sub16, &sub15);
    }
}
