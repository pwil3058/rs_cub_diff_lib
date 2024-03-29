// Copyright 2019 Peter Williams <pwil3058@gmail.com>

use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, Read};
use std::path::Path;
pub use std::sync::Arc;

use lazy_static;
use regex::Regex;

pub type Line = Arc<String>;
pub type Lines = Vec<Line>;

pub trait LineIfce {
    fn new(s: &str) -> Line {
        Arc::new(String::from(s))
    }

    fn conflict_start_marker() -> Line {
        Arc::new(String::from("<<<<<<<"))
    }

    fn conflict_separation_marker() -> Line {
        Arc::new(String::from("======="))
    }

    fn conflict_end_marker() -> Line {
        Arc::new(String::from(">>>>>>>"))
    }

    fn has_trailing_white_space(&self) -> bool;
}

lazy_static! {
    static ref HAS_TWS_CRE: Regex = Regex::new(r"[ \t\f\v]+(\n)?$").unwrap();
}

impl LineIfce for Line {
    fn has_trailing_white_space(&self) -> bool {
        HAS_TWS_CRE.is_match(self)
    }
}

pub trait LinesIfce {
    fn read<R: Read>(read: R) -> io::Result<Lines> {
        //let file = File::open(path)?;
        let mut reader = BufReader::new(read);
        let mut lines = vec![];
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 {
                break;
            } else {
                lines.push(Arc::new(line))
            }
        }
        Ok(lines)
    }

    fn read_from(file_path: &Path) -> io::Result<Lines> {
        let file = File::open(file_path)?;
        Self::read(file)
    }

    fn from_string(string: &str) -> Lines {
        let mut lines: Lines = vec![];
        let mut start_index = 0;
        for (end_index, _) in string.match_indices('\n') {
            lines.push(Arc::new(string[start_index..=end_index].to_string()));
            start_index = end_index + 1;
        }
        if start_index < string.len() {
            lines.push(Arc::new(string[start_index..].to_string()));
        }
        lines
    }

    // Does we contain "sub_lines" starting at "index"?
    fn contains_sub_lines_at(&self, sub_lines: &[Line], index: usize) -> bool;

    // Find index of the first instance of "sub_lines" at or after "start_index"
    fn find_first_sub_lines(&self, sub_lines: &[Line], start_index: usize) -> Option<usize>;
}

impl LinesIfce for &[Line] {
    fn contains_sub_lines_at(&self, sub_lines: &[Line], index: usize) -> bool {
        if sub_lines.len() + index > self.len() {
            return false;
        }
        for (line, sub_line) in self[index..index + sub_lines.len()].iter().zip(sub_lines) {
            if line != sub_line {
                return false;
            }
        }
        true
    }

    fn find_first_sub_lines(&self, sub_lines: &[Line], start_index: usize) -> Option<usize> {
        (start_index..=start_index + self.len() - sub_lines.len()).find(|&index| self.contains_sub_lines_at(sub_lines, index))
    }
}

impl LinesIfce for Lines {
    fn contains_sub_lines_at(&self, sub_lines: &[Line], index: usize) -> bool {
        if sub_lines.len() + index > self.len() {
            return false;
        }
        for (line, sub_line) in self[index..index + sub_lines.len()].iter().zip(sub_lines) {
            if line != sub_line {
                return false;
            }
        }
        true
    }

    fn find_first_sub_lines(&self, sub_lines: &[Line], start_index: usize) -> Option<usize> {
        (start_index..=start_index + self.len() - sub_lines.len()).find(|&index| self.contains_sub_lines_at(sub_lines, index))
    }
}

pub fn first_inequality_fm_head(lines1: &[Line], lines2: &[Line]) -> Option<usize> {
    if let Some(index) = lines1.iter().zip(lines2.iter()).position(|(a, b)| a != b) {
        Some(index)
    } else if lines1.len() == lines2.len() {
        None
    } else {
        Some(lines1.len().min(lines2.len()))
    }
}

pub fn first_inequality_fm_tail(lines1: &[Line], lines2: &[Line]) -> Option<usize> {
    if let Some(index) = lines1
        .iter()
        .rev()
        .zip(lines2.iter().rev())
        .position(|(a, b)| a != b)
    {
        Some(index)
    } else if lines1.len() > lines2.len() {
        Some(lines1.len() - lines2.len())
    } else if lines2.len() > lines1.len() {
        Some(lines2.len() - lines1.len())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_from_strings_works() {
        let test_string = " aaa\nbbb \nccc ddd\njjj";
        let lines = Lines::from_string(test_string);
        assert!(lines.len() == 4);
        let lines = Lines::from_string(test_string);
        assert!(lines.len() == 4);
        assert!(*lines[0] == " aaa\n");
        assert!(*lines[3] == "jjj");
        let test_string = " aaa\nbbb \nccc ddd\njjj\n";
        let lines = Lines::from_string(test_string);
        assert!(lines.len() == 4);
        let lines = Lines::from_string(test_string);
        assert!(lines.len() == 4);
        assert!(*lines[0] == " aaa\n");
        assert!(*lines[3] == "jjj\n");
    }
}
