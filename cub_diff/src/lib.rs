// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

#[macro_use]
extern crate lazy_static;

pub mod lines {
    use regex::Regex;

    lazy_static! {
        static ref HAS_TWS_CRE: Regex = Regex::new(r"[ \t\f\v]+(\n)?$").unwrap();
    }

    pub trait LineIfce {
        fn has_trailing_white_space(&self) -> bool;
    }

    impl LineIfce for &str {
        fn has_trailing_white_space(&self) -> bool {
            HAS_TWS_CRE.is_match(self)
        }
    }

    pub trait LinesIfce {
        // Does we contain "sub_lines" starting at "index"?
        fn contains_sub_lines_at(&self, sub_lines: &[&str], index: usize) -> bool;

        // Find index of the first instance of "sub_lines" at or after "start_index"
        fn find_first_sub_lines(&self, sub_lines: &[&str], start_index: usize) -> Option<usize>;
    }

    impl LinesIfce for &[&str] {
        fn contains_sub_lines_at(&self, sub_lines: &[&str], index: usize) -> bool {
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

        fn find_first_sub_lines(&self, sub_lines: &[&str], start_index: usize) -> Option<usize> {
            for index in start_index..=start_index + self.len() - sub_lines.len() {
                if self.contains_sub_lines_at(sub_lines, index) {
                    return Some(index);
                }
            }
            None
        }
    }

    pub fn first_inequality_fm_head(lines1: &[&str], lines2: &[&str]) -> Option<usize> {
        if let Some(index) = lines1.iter().zip(lines2.iter()).position(|(a, b)| a != b) {
            Some(index)
        } else if lines1.len() == lines2.len() {
            None
        } else {
            Some(lines1.len().min(lines2.len()))
        }
    }

    pub fn first_inequality_fm_tail(lines1: &[&str], lines2: &[&str]) -> Option<usize> {
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
}

pub mod text_diff {
    use std::iter::Iterator;

    pub fn extract_source_lines<'a, F: Fn(&str) -> bool>(
        lines: &[&'a str],
        trim_left_n: usize,
        skip: F,
    ) -> Vec<&'a str> {
        let mut trimmed_lines: Vec<&str> = vec![];
        for (index, ref line) in lines.iter().enumerate() {
            if skip(line) || line.starts_with('\\') {
                continue;
            }
            if (index + 1) == lines.len() || !lines[index + 1].starts_with('\\') {
                trimmed_lines.push(&line[trim_left_n..]);
            } else {
                trimmed_lines.push(&line[trim_left_n..].trim_end_matches('\n'));
            }
        }
        trimmed_lines
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
