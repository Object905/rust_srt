#[macro_use] extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, String>;

#[derive(Clone)]
pub struct SubLine {
    text: String,
    index: u32,
    start: u32,
    end: u32,
}

impl fmt::Display for SubLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}\n{:?} --> {:?} ({}>>{})\n{}",
               self.index,
               self.start,
               self.end,
               self.start,
               self.end,
               self.text)
    }
}

impl SubLine {
    pub fn get_start(&self) -> u32 {
        self.start
    }

    pub fn get_text(&self) -> String {
        self.text.clone()
    }

    pub fn get_start_timestamp(&self) -> [u32; 4] {
        recover_timestamp(&self.start)
    }

    pub fn get_end_timestamp(&self) -> [u32; 4] {
        recover_timestamp(&self.end)
    }

    pub fn get_index(&self) -> u32 {
        self.index
    }

    pub fn get_end(&self) -> u32 {
        self.end
    }

    pub fn get_duration(&self) -> u32 {
        self.end - self.start
    }

    pub fn shift(&mut self, offset: i32) -> Result<()> {
        if offset > 0 {
            self.start += offset as u32;
            self.end += offset as u32;
            Ok(())
        } else if offset < 0 {
            if (-offset as u32 >= self.start) || (-offset as u32 >= self.end) {
                return Err("start(end) can't be negative".to_string())
            };
            self.start -= -offset as u32;
            self.end -= -offset as u32;
            Ok(())
        } else {
            Err("offset can't be 0".to_string())
        }
    }
}

trait SubLineVector {
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
}

pub type Subtitles = Vec<SubLine>;

impl SubLineVector for Subtitles {
    fn by_index(&self, index: usize) -> Option<&SubLine> {
        self.get(index - 1)
    }

    fn by_index_mut(&mut self, index: usize) -> Option<&mut SubLine>{
        self.get_mut(index - 1)
    }

    fn by_miliseconds(&self, time: u32) -> Option<&SubLine> {
        // binary search
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
        // binary search
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
            Some(&mut self[result])
        } else { None }
    }
}

fn to_miliseconds(timestamp: &[u32; 4]) -> u32 {
    let mut result: u32 = 0;
    result += timestamp[3];
    result += timestamp[2] * 1000;
    result += timestamp[1] * 1000 * 60;
    result += timestamp[0] * 1000 * 60 * 60;
    result
}

fn recover_timestamp(miliseconds: &u32) -> [u32;4] {
    let hours = miliseconds / 3600000;
    let minutes = (miliseconds - hours * 3600000) / 60000;
    let seconds = (miliseconds - (hours * 3600000 + minutes * 60000)) / 1000;
    let miliseconds = miliseconds - (hours * 3600000 + minutes * 60000 + seconds * 1000);
    [hours, minutes, seconds, miliseconds]
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
    }

fn read_file(path: &Path) -> Result<String> {
    let mut file = try!(File::open(&path).map_err(|e| e.to_string()));
    let mut content = String::new();
    try!(file.read_to_string(&mut content).map_err(|e| e.to_string()));
    Ok(content)
}

fn parse(content: String) -> Subtitles {
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
        };
        result.push(line)
    }
    result
}

pub fn prepare(path: &str) -> Result<Subtitles> {
    let sub_path = Path::new(path);
    let mut content = try!(read_file(&sub_path).map_err(|e| e.to_string()));

    let eof_empty_lines = Regex::new(r"\S\s*?\z").unwrap();
    let unify_newline_style = Regex::new(r"(?x)
            ([^\n(\r\n)]\r[^\n(\r\n)])|
            ([^\r(\r\n)]\n[^\r(\r\n)])")
        .unwrap();

    content = eof_empty_lines.replace_all(&content, "\r\n\r\n");
    content = unify_newline_style.replace_all(&content, "\r\n");

    if !SUBS.is_match(&content) {
        return Err("Bad srt file".to_string());
    }

    Ok(parse(content))
}


#[test]
fn test1 () {
    let path = "ex1";
    let mut subs = prepare(path).unwrap();
    let test = subs.by_index(3).unwrap().clone();

    {
        let another_one = subs.by_miliseconds(261072).unwrap();
        let same_line = subs.by_timestamp(&[0, 4, 22, 500]).unwrap();
        assert_eq!(test.get_index(), same_line.get_index());
        assert_eq!(same_line.get_index(), another_one.get_index());
    }
    {
        let mut mutline = subs.by_index_mut(3).unwrap();
        mutline.shift(-5).unwrap();
    }
    {
        let changed_line = subs.by_index(3).unwrap();
        assert_eq!(changed_line.get_start(), test.get_start() - 5);
    }
}