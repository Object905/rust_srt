#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::fmt;

fn read_file(path: &Path) -> String {
    let mut file = File::open(&path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    return content;
}

fn parse(content: String) {
    let re = Regex::new(r"\S.*\z").unwrap();
    let prepared = re.replace_all(&content, "\n\n");
    let mut result: Vec<Line> = vec![];

    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?x)
            (\d+)
            (\r\n|\r|\n)
            (\d{2}):(\d{2}):(\d{2}),(\d{3})
            \s-->\s
            (\d{2}):(\d{2}):(\d{2}),(\d{3})
            (\r\n|\r|\n)
            ([\s\S]*?)
            (\r\n|\r|\n){2}").unwrap();
    }

    for cap in RE.captures_iter(&prepared) {
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
            duration: end - start,
        };
        result.push(line)
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

#[derive(Debug)]
struct Line {
    text: String,
    index: u32,
    start_timestamp: [u32; 4],
    end_timestamp: [u32; 4],
    start: u32,
    end: u32,
    duration: u32,
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}\n{:?} --> {:?}\n{}",
               self.index,
               self.start_timestamp,
               self.end_timestamp,
               self.text)
    }
}

#[derive(Debug)]
struct Subtitles {
    field: Vec<Line>,
}

#[test]
fn it_works() {
    let path = Path::new("/home/obj/ex1");
    let st = read_file(&path);
    parse(st);
}