#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::fmt;
use std::io::Error;

fn read_file(path: &Path) -> Result<String, Error> {
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => return Err(e),
    };
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    Ok(content)
}

fn parse(content: &mut String) -> Subtitles {
    let eof_space_remover = Regex::new(r"\S\s*?\z").unwrap();
    let content = eof_space_remover.replace_all(&content, "\r\n\r\n");
    let newline_unificator =
        Regex::new(r"([^\n(\r\n)]\r[^\n(\r\n)])|([^\r(\r\n)]\n[^\r(\r\n)])") // to win style
        .unwrap(); // may fail to mixed style
    let content = newline_unificator.replace_all(&content, "\r\n");

    let mut result: Vec<Line> = vec![];

    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?x)
            (\d+)
            (\r\n)
            (\d{2}):(\d{2}):(\d{2}),(\d{3})
            \s-->\s
            (\d{2}):(\d{2}):(\d{2}),(\d{3})
            (\r\n)
            ([\S\s]*?)
            (\r\n){2}?").unwrap();
    }

    for cap in RE.captures_iter(&content) {
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

#[derive(Debug)]
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
    fn at_index(&self, index: usize) -> &Line {
        &self.field[index - 1]
    }

    fn get_length(&self) -> usize {
        self.field.len()
    }

    fn by_timestamp(&self, timestamp: [u32; 4]) -> Option<&Line> {
        let miliseconds = to_miliseconds(&timestamp);
        let mut min = 0;
        let mut max = self.field.len() - 1;
        let mut guess_index;

        while max > min {
            guess_index = (max + min) / 2;
            let guess = &self.field[guess_index];

            if (guess.start <= miliseconds) && (miliseconds <= guess.end) {
                return Some(guess);
            } else if miliseconds > guess.end {
                min = guess_index + 1;
            } else {
                max = guess_index - 1;
            }
        }
        None
    }
}

fn prepare(path: &str) -> Result<Subtitles, Error> {
    let sub_path = Path::new(path);
    let mut content = match read_file(&sub_path) {
        Ok(content) => content,
        Err(e) => return Err(e),

    };
    let subs = parse(&mut content);
    Ok(subs)
}

#[test]
fn it_works() {
    let path = "/home/obj/ex1";
    let subs = prepare(path).unwrap();
    println!("{}", subs.at_index(3));
    println!("{}", subs.at_index(619));
    println!("{}", subs.at_index(13));
}