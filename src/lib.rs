#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::fs::File;
use std::path::Path;
use std::fmt::{Display, Formatter};
use std::convert::AsRef;
use std::io::{Error, ErrorKind, Read, Write};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubLine {
    pub text: String,
    pub index: u32,
    pub start: u32,
    pub end: u32,
    pub start_timestamp: [u32; 4],
    pub end_timestamp: [u32; 4],
}

impl Display for SubLine {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f,
               "{index}\r\n{s_h:02}:{s_m:02}:{s_s:02},{s_ms:03} --> \
                {e_h:02}:{e_m:02}:{e_s:02},{e_ms:03}\r\n{text}\r\n\r\n",
               index = self.index,
               s_h = self.start_timestamp[0],
               s_m = self.start_timestamp[1],
               s_s = self.start_timestamp[2],
               s_ms = self.start_timestamp[3],
               e_h = self.end_timestamp[0],
               e_m = self.end_timestamp[1],
               e_s = self.end_timestamp[2],
               e_ms = self.end_timestamp[3],
               text = self.text)
    }
}

impl SubLine {
    pub fn get_duration(&self) -> u32 {
        self.end - self.start
    }

    pub fn shift(&mut self, offset: i32) -> Result<(), &'static str> {
        if offset > 0 {
            self.start += offset as u32;
            self.end += offset as u32;
            Ok(())
        } else if offset < 0 {
            if (-offset as u32 >= self.start) || (-offset as u32 >= self.end) {
                return Err("start(or end) is going to be negative with given offset");
            };
            self.start -= -offset as u32;
            self.end -= -offset as u32;
            Ok(())
        } else {
            Err("offset can't be 0")
        }
    }
}

pub trait SaveSub<P: AsRef<Path>> {
    fn save_to(&self, path: P) -> Result<(), Error>;
}

pub trait SubLineVector {
    fn insert_line(&mut self, item: SubLine);

    fn by_index(&self, index: usize) -> Option<&SubLine>;
    fn by_index_mut(&mut self, index: usize) -> Option<&mut SubLine>;

    fn by_timestamp(&self, timestamp: &[u32; 4]) -> Option<&SubLine> {
        let miliseconds = to_miliseconds(&timestamp);
        self.by_miliseconds(miliseconds)
    }

    fn by_timestamp_mut(&mut self, timestamp: &[u32; 4]) -> Option<&mut SubLine> {
        let miliseconds = to_miliseconds(&timestamp);
        self.by_miliseconds_mut(miliseconds)
    }

    fn by_miliseconds(&self, time: u32) -> Option<&SubLine>;
    fn by_miliseconds_mut(&mut self, time: u32) -> Option<&mut SubLine>;

    fn nearest_by_miliseconds(&self, time: u32) -> Option<&SubLine>;
    fn nearest_by_miliseconds_mut(&mut self, time: u32) -> Option<&mut SubLine>;
}

pub type Subtitles = Vec<SubLine>;

impl<P: AsRef<Path>> SaveSub<P> for Subtitles {
    fn save_to(&self, path: P) -> Result<(), Error> {
        let mut file = try!(File::create(&path));
        for line in self {
            try!(write!(&mut file, "{repr}", repr = line));
        }
        try!(write!(&mut file, "\r\n\r\n"));
        Ok(())
    }
}

impl SubLineVector for Subtitles {
    fn insert_line(&mut self, item: SubLine) {
        let vector_index = (item.index - 1) as usize;
        let index = item.index as usize;
        self.insert(vector_index, item);
        for i in &mut self[index..] {
            i.index += 1;
        }
    }

    fn by_index(&self, index: usize) -> Option<&SubLine> {
        self.get(index - 1)
    }

    fn by_index_mut(&mut self, index: usize) -> Option<&mut SubLine> {
        self.get_mut(index - 1)
    }

    fn nearest_by_miliseconds(&self, time: u32) -> Option<&SubLine> {
        let mut min = 0;
        let mut max = self.len() - 1;
        let mut guess_index;

        while max >= min {
            guess_index = (max + min) / 2;
            let guess = self.get(guess_index).unwrap();

            if (guess.start <= time) && (time <= guess.end) {
                return Some(&guess);
            } else if time > guess.end {
                min = guess_index + 1;
            } else {
                max = guess_index - 1;
            }
        }
        self.get(max)
    }

    fn nearest_by_miliseconds_mut(&mut self, time: u32) -> Option<&mut SubLine> {
        let mut min = 0;
        let mut max = self.len() - 1;
        let mut guess_index;
        let mut result: usize = 0;
        let mut state = false;

        while max >= min {
            guess_index = (max + min) / 2;

            let guess = self.get(guess_index).unwrap();

            if (guess.start <= time) && (time <= guess.end) {
                result = guess_index;
                state = true;
                break;
            } else if time > guess.end {
                min = guess_index + 1;
            } else {
                max = guess_index - 1;
            }
        }
        if state {
            self.get_mut(result)
        } else {
            self.get_mut(max)
        }
    }

    fn by_miliseconds(&self, time: u32) -> Option<&SubLine> {
        let mut min = 0;
        let mut max = self.len() - 1;
        let mut guess_index;

        while max >= min {
            guess_index = (max + min) / 2;
            let guess = self.get(guess_index).unwrap();

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

    fn by_miliseconds_mut(&mut self, time: u32) -> Option<&mut SubLine> {
        let mut min = 0;
        let mut max = self.len() - 1;
        let mut guess_index;
        let mut result: usize = 0;
        let mut state = false;

        while max >= min {
            guess_index = (max + min) / 2;

            let guess = self.get(guess_index).unwrap();

            if (guess.start <= time) && (time <= guess.end) {
                result = guess_index;
                state = true;
                break;
            } else if time > guess.end {
                min = guess_index + 1;
            } else {
                max = guess_index - 1;
            }
        }
        if state { self.get_mut(result) } else { None }
    }
}

pub fn to_miliseconds(timestamp: &[u32; 4]) -> u32 {
    let mut result: u32 = 0;
    result += timestamp[3];
    result += timestamp[2] * 1000;
    result += timestamp[1] * 1000 * 60;
    result += timestamp[0] * 1000 * 60 * 60;
    result
}

lazy_static! {
    pub static ref SUBS: Regex = Regex::new(r"(?x)
        (\d+)
        (\r\n)
        (\d{2}):(\d{2}):(\d{2}),(\d{3})
        \s-->\s
        (\d{2}):(\d{2}):(\d{2}),(\d{3})
        (\r\n)
        ([\S\s]*?)
        (\r\n){2}?").unwrap();
    static ref EOF_EMPTY_LINES: Regex = Regex::new(r"\s*?\z").unwrap();
    static ref UNIFY_NEWLINE_STYLE: Regex = Regex::new(r"(?x)
            ([^\n(\r\n)]\r[^\n(\r\n)])|
            ([^\r(\r\n)]\n[^\r(\r\n)])").unwrap();

}

fn read_file<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let mut file = try!(File::open(&path));
    let mut content = String::new();
    try!(file.read_to_string(&mut content));
    Ok(content)
}

fn parse(content: &str) -> Subtitles {
    let mut result: Subtitles = Subtitles::new();

    for cap in SUBS.captures_iter(&content) {
        let start_timestamp = [cap.at(3).unwrap().parse::<u32>().unwrap(),
                               cap.at(4).unwrap().parse::<u32>().unwrap(),
                               cap.at(5).unwrap().parse::<u32>().unwrap(),
                               cap.at(6).unwrap().parse::<u32>().unwrap()];
        let end_timestamp = [cap.at(7).unwrap().parse::<u32>().unwrap(),
                             cap.at(8).unwrap().parse::<u32>().unwrap(),
                             cap.at(9).unwrap().parse::<u32>().unwrap(),
                             cap.at(10).unwrap().parse::<u32>().unwrap()];

        let start = to_miliseconds(&start_timestamp);
        let end = to_miliseconds(&end_timestamp);

        let line = SubLine {
            index: cap.at(1).unwrap().parse::<u32>().unwrap(),
            text: cap.at(12).unwrap().to_string(),
            start: start,
            end: end,
            start_timestamp: start_timestamp,
            end_timestamp: end_timestamp,
        };
        result.push(line)
    }
    result
}

pub fn prepare<P: AsRef<Path>>(path: P) -> Result<Subtitles, Error> {
    let mut content = try!(read_file(&path));
    content = EOF_EMPTY_LINES.replace_all(&content, "\r\n\r\n");
    content = UNIFY_NEWLINE_STYLE.replace_all(&content, "\r\n");

    if !SUBS.is_match(&content) {
        return Err(Error::new(ErrorKind::InvalidData,
                              "Given file does not match with srt format specification"));
    }

    Ok(parse(&content))
}
