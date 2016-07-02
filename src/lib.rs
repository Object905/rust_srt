#[macro_use] extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, String>;

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

fn parse(content: String, path: &str) -> Subtitles {
    let mut result: Vec<SubLine> = vec![];

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
    Subtitles { field: result, path: path.to_string() }
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

#[derive(Debug, Clone)]
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
                return Err("start can't be negative".to_string())
            };
            self.start -= -offset as u32;
            self.end -= -offset as u32;
            Ok(())
        } else {
            Err("offset can't be 0".to_string())
        }
    }
}

#[derive(Debug)]
pub struct Subtitles {
    field: Vec<SubLine>,
    path: String,
}

impl fmt::Display for Subtitles {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "length: {}\n{}\n..........\n{}\n",
               self.get_length(),
               self.field.first().unwrap(),
               self.field.last().unwrap())
    }
}

impl IntoIterator for Subtitles {
    type Item = SubLine;
    type IntoIter = ::std::vec::IntoIter<SubLine>;
    fn into_iter(self) -> Self::IntoIter {
        self.field.into_iter()
    }
}

impl Subtitles {
    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn by_index(&self, index: usize) -> Option<SubLine> {
        self.field.get(index - 1).map(|n| n.clone())
    }

    pub fn get_length(&self) -> usize {
        self.field.len()
    }

    pub fn by_timestamp(&self, timestamp: &[u32; 4]) -> Option<SubLine> {
        let miliseconds = to_miliseconds(&timestamp);
        self.by_miliseconds(miliseconds)
    }

    pub fn by_miliseconds(&self, time: u32) -> Option<SubLine> {
        // binary search
        let mut min = 0;
        let mut max = self.field.len() - 1;
        let mut guess_index;

        while max >= min {
            guess_index = (max + min) / 2;
            let guess = self.field.get(guess_index).unwrap();

            if (guess.start <= time) && (time <= guess.end) {
                return Some(guess.clone());
            } else if time > guess.end {
                min = guess_index + 1;
            } else {
                max = guess_index - 1;
            }
        }
        None
    }

    pub fn mut_by_miliseconds(&mut self, time: u32) -> Option<&mut SubLine> {
        // binary search
        let mut min = 0;
        let mut max = self.field.len() - 1;
        let mut guess_index;
        let mut result: usize = 0;
        let mut state = false;

        while max >= min {
            guess_index = (max + min) / 2;

            let guess = self.field.get(guess_index).unwrap();

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
            Some(&mut self.field[result])
        } else { None }
    }

    pub fn mut_by_timestamp(&mut self, timestamp: &[u32; 4]) -> Option<&mut SubLine> {
        let miliseconds = to_miliseconds(&timestamp);
        self.mut_by_miliseconds(miliseconds)
    }

    pub fn mut_by_index(&mut self, index: usize) -> Option<&mut SubLine>{
        self.field.get_mut(index - 1)
    }

    pub fn shift_all(&mut self, offset: i32) -> Result<()> {
        for line in &mut self.field {
            try!(line.shift(offset));
        }
        Ok(())
    }
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

    Ok(parse(content, path))
}

#[test]
fn it_works() {
    let path = "ex1";
    let mut subs = prepare(path).unwrap();

    let line = subs.by_index(3).unwrap();
    {
        let another_one = subs.by_miliseconds(261072).unwrap();
        let same_line = subs.by_timestamp(&[0, 4, 22, 500]).unwrap();
        assert_eq!(line.get_index(), same_line.get_index());
        assert_eq!(same_line.get_text(), another_one.get_text());
    }

    println!("{}\n{:?} --> {:?}\n{}\n",
             line.get_index(),
             line.get_start(),
             line.get_end(),
             line.get_text());
    println!("duration: {}", line.get_duration());

    {
        let mut mutline = subs.mut_by_miliseconds(261072).unwrap();
        mutline.shift(-1).unwrap();
    }

    {
        let newline = subs.by_index(3).unwrap();
        assert_eq!(newline.get_start(), line.get_start() - 1);
    }
    
    subs.shift_all(1).unwrap();

    {
        let newline = subs.by_index(3).unwrap();
        assert_eq!(newline.get_start(), line.get_start());
    }

}