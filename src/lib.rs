#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::fmt;
use std::result;

type Result<T> = result::Result<T, String>;

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
    let mut result: Vec<Line> = vec![];

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

        let line = Line {
            index: cap.at(1).unwrap().parse::<u32>().unwrap(),
            start_timestamp: start_timestamp,
            end_timestamp: end_timestamp,
            text: cap.at(12).unwrap().to_string(),
            start: start,
            end: end,
        };
        result.push(line)
    }
    Subtitles { field: result }
}


fn to_miliseconds(timestamp: &[u32; 4]) -> u32 {
    let mut result: u32 = 0;
    result += timestamp[3];
    result += timestamp[2] * 1000;
    result += timestamp[1] * 1000 * 60;
    result += timestamp[0] * 1000 * 60 * 60;
    result
}

#[derive(Debug, Clone)]
struct Line {
    text: String,
    index: u32,
    start_timestamp: [u32; 4],
    end_timestamp: [u32; 4],
    start: u32,
    end: u32,
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}\n{:?} --> {:?} ({}>>{})\n{}",
               self.index,
               self.start_timestamp,
               self.end_timestamp,
               self.start,
               self.end,
               self.text)
    }
}


impl Line {
    fn get_start(&self) -> u32 {
        self.start
    }

    fn get_start_timestamp(&self) -> [u32;4] {
        self.start_timestamp
    }

    fn get_end(&self) -> u32 {
        self.end
    }

    fn get_duration(&self) -> u32 {
        self.end - self.start
    }
}

#[derive(Debug)]
struct Subtitles {
    field: Vec<Line>,
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

impl Subtitles {
    fn at_index(&self, index: usize) -> Option<Line> {
        self.field.get(index - 1).map(|n| n.clone())
    }

    fn get_length(&self) -> usize {
        self.field.len()
    }

    fn by_timestamp(&self, timestamp: [u32; 4]) -> Option<Line> {
        let miliseconds = to_miliseconds(&timestamp);
        self.by_miliseconds(miliseconds)
    }

    fn by_miliseconds(&self, time: u32) -> Option<Line> {
        let mut min = 0;
        let mut max = self.field.len() - 1;
        let mut guess_index;

        while max >= min {
            guess_index = (max + min) / 2;
            let guess = &self.field[guess_index];

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
}

fn prepare(path: &str) -> Result<Subtitles> {
    let sub_path = Path::new(path);
    let mut content = try!(read_file(&sub_path).map_err(|e| e.to_string()));

    let eof_empty_lines = Regex::new(r"\S\s*?\z").unwrap();
    let unify_newline_style = Regex::new(r"(?x)
            ([^\n(\r\n)]\r[^\n(\r\n)])|
            ([^\r(\r\n)]\n[^\r(\r\n)])").unwrap();

    content = eof_empty_lines.replace_all(&content, "\r\n\r\n");
    content = unify_newline_style.replace_all(&content, "\r\n");

    if !SUBS.is_match(&content){ return Err("Bad srt file".to_string()); }

    Ok(parse(content))
}

#[test]
fn it_works() {
    let path = "ex1";
    let subs = prepare(path).unwrap();
    println!("{}", subs.at_index(3).unwrap());
    println!("{}", subs.by_miliseconds(261072).unwrap());
    println!("{}", subs.by_timestamp([0, 4, 22, 500]).unwrap());
}