#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::result;
use std::fmt::{Display, Formatter};

pub type Result<T> = result::Result<T, String>;

#[derive(Clone, Debug, PartialEq)]
pub struct SubLine {
    pub text: String,
    pub index: u32,
    pub start: u32,
    pub end: u32,
}


impl Display for SubLine {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let start = recover_timestamp(&self.start);
        let end = recover_timestamp(&self.end);
        write!(f,
               "{index}\r\n\
{s_h:02}:{s_m:02}:{s_s:02},{s_ms:03} --> {e_h:02}:{e_m:02}:{e_s:02},{e_ms:03}\r\n\
{text}\r\n\r\n",
               index = self.index,
               s_h = start[0],
               s_m = start[1],
               s_s = start[2],
               s_ms = start[3],
               e_h = end[0],
               e_m = end[1],
               e_s = end[2],
               e_ms = end[3],
               text = self.text)
    }
}

impl SubLine {
    pub fn get_start_timestamp(&self) -> [u32; 4] {
        recover_timestamp(&self.start)
    }

    pub fn get_end_timestamp(&self) -> [u32; 4] {
        recover_timestamp(&self.end)
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
                return Err("start(or end) can't be negative".to_string());
            };
            self.start -= -offset as u32;
            self.end -= -offset as u32;
            Ok(())
        } else {
            Err("offset can't be 0".to_string())
        }
    }
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

    fn save_to(&self, path: &str) -> Result<()>;
}

pub type Subtitles = Vec<SubLine>;

impl SubLineVector for Subtitles {
    fn insert_line(&mut self, item: SubLine) {
        let vector_index = (item.index - 1) as usize;
        let index = item.index as usize;
        self.insert(vector_index, item);
        for i in &mut self[index..] {
            i.index += 1
        }
    }

    fn save_to(&self, path: &str) -> Result<()> {
        let mut file = try!(File::create(&path).map_err(|e| e.to_string()));
        for line in self {
            let start = line.get_start_timestamp();
            let end = line.get_end_timestamp();
            try!(write!(&mut file,
                        "{index}\r\n{s_h:02}:{s_m:02}:{s_s:02},{s_ms:03} --> \
                         {e_h:02}:{e_m:02}:{e_s:02},{e_ms:03}\r\n{text}\r\n\r\n",
                        index = line.index,
                        s_h = start[0],
                        s_m = start[1],
                        s_s = start[2],
                        s_ms = start[3],
                        e_h = end[0],
                        e_m = end[1],
                        e_s = end[2],
                        e_ms = end[3],
                        text = line.text)
                .map_err(|e| e.to_string()));
        }
        try!(write!(&mut file, "\r\n").map_err(|e| e.to_string()));
        Ok(())
    }

    fn by_index(&self, index: usize) -> Option<&SubLine> {
        self.get(index - 1)
    }

    fn by_index_mut(&mut self, index: usize) -> Option<&mut SubLine> {
        self.get_mut(index - 1)
    }

    fn nearest_by_miliseconds(&self, time: u32) -> Option<&SubLine> {
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
        self.get(min - 1)
    }

    fn nearest_by_miliseconds_mut(&mut self, time: u32) -> Option<&mut SubLine> {
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
        if state { Some(&mut self[result]) } else { self.get_mut(min - 1) }
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
        if state { Some(&mut self[result]) } else { None }
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

fn recover_timestamp(miliseconds: &u32) -> [u32; 4] {
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
    static ref EOF_EMPTY_LINES: Regex = Regex::new(r"\s*?\z").unwrap();
    static ref UNIFY_NEWLINE_STYLE: Regex = Regex::new(r"(?x)
            ([^\n(\r\n)]\r[^\n(\r\n)])|
            ([^\r(\r\n)]\n[^\r(\r\n)])").unwrap();

}

fn read_file(sub_path: &str) -> Result<String> {
    let path = Path::new(sub_path);
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
    let mut content = try!(read_file(&path).map_err(|e| e.to_string()));
    content = EOF_EMPTY_LINES.replace_all(&content, "\r\n\r\n");
    content = UNIFY_NEWLINE_STYLE.replace_all(&content, "\r\n");

    if !SUBS.is_match(&content) {
        return Err("Bad srt file".to_string());
    }

    Ok(parse(content))
}