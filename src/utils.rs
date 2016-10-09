use std::fs::File;
use std::path::Path;
use std::io::{Error, Read};

use regex::Regex;

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let mut file = try!(File::open(&path));
    let mut content = String::new();
    try!(file.read_to_string(&mut content));
    Ok(content)
}

pub fn prepare(content: &str) -> String {
    let result = EOF_EMPTY_LINES.replacen(&content, 1, "\r\n\r\n");
    UNIFY_NEWLINE_STYLE.replace_all(&result, "\r\n")
}

pub fn check(content: &str) -> bool {
    SUBS.is_match(&content)
}

lazy_static! {
    pub static ref SUBS: Regex = Regex::new(r"(?x)
        (\d+)
        \r\n
        (\d{2}):(\d{2}):(\d{2}),(\d{3})
        \s-->\s
        (\d{2}):(\d{2}):(\d{2}),(\d{3})
        \r\n
        ([\S\s]*?)
        (?:\r\n){2}?").unwrap();

    static ref EOF_EMPTY_LINES: Regex = Regex::new(r"\s*?\z").unwrap();
    static ref UNIFY_NEWLINE_STYLE: Regex = Regex::new(r"(?:\r\n|\n)").unwrap();
}

#[cfg(test)]
mod utils_tests {
    use super::*;

    #[test]
    fn _check() {
        println!("");
        let mut test_srt = r"1
00:01:38,958 --> 00:01:49,609
Firs line

2
00:04:19,604 --> 00:04:20,970
<i>Your Grace.</i>

3
00:04:21,072 --> 00:04:24,707
The trial will be
getting under way soon.

4
00:04:57,141 --> 00:04:58,541
You got my money?

5
00:04:58,643 --> 00:05:00,943
Later.
Go away.

    
 
".to_owned();
        println!("before prepare: {:?}", test_srt);
        assert!(!check(&test_srt));

        test_srt = prepare(&test_srt);
        println!("after prepare: {:?}", test_srt);
        let num_subs = SUBS.captures_iter(&test_srt).count();
        assert_eq!(num_subs, 5);
        assert!(check(&test_srt));

        test_srt.pop();
        let num_subs = SUBS.captures_iter(&test_srt).count();
        assert_eq!(num_subs, 4);
    }

    #[test]
    fn _prepare() {
        println!("");
        let test_srt = "1\n00:01:38,958 --> 00:01:49,609\nFirs line\n\n2\n00:04:19,604 --> 00:04:20,970\n<i>Your Grace.</i>\n\n".to_owned();
        println!("before prepare: {:?}", test_srt);
        assert!(!check(&test_srt));

        let mut prepaired_test_srt = prepare(&test_srt);
        println!("after prepare: {:?}", prepaired_test_srt);
        let num_subs = SUBS.captures_iter(&prepaired_test_srt).count();
        assert_eq!(num_subs, 2);
        println!("{:?}", prepaired_test_srt);
        assert!(check(&prepaired_test_srt));

        let another_prepaired_test_srt = test_srt.replace("\n", "\r\n");
        println!("{:?}", another_prepaired_test_srt);
        println!("\n\n");
        assert_eq!(prepaired_test_srt, another_prepaired_test_srt);

        let mut additional_test_srt = "3\n00:04:21,072 --> 00:04:24,707\nThird line\n\n".to_owned();
        additional_test_srt = prepare(&additional_test_srt);

        prepaired_test_srt += &additional_test_srt;
        let num_subs = SUBS.captures_iter(&prepaired_test_srt).count();
        assert_eq!(num_subs, 3);
    }

}