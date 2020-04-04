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

    #[derive(Debug)]
    pub struct CompleteLines<'a> {
        string: &'a str,
        start_index: usize,
    }

    impl<'a> CompleteLines<'a> {
        pub fn new(string: &'a str) -> Self {
            Self {
                string,
                start_index: 0,
            }
        }
    }

    impl<'a> Iterator for CompleteLines<'a> {
        type Item = &'a str;

        fn next(&mut self) -> Option<Self::Item> {
            if self.start_index < self.string.len() {
                let start_index = self.start_index;
                let end_index = if let Some(offset_index) = self.string[start_index..].find('\n') {
                    start_index + offset_index + 1
                } else {
                    self.string.len()
                };
                self.start_index = end_index;
                Some(&self.string[start_index..end_index])
            } else {
                None
            }
        }
    }

    pub trait CompleteLinesExt {
        fn complete_lines(&self) -> CompleteLines;
    }

    impl CompleteLinesExt for String {
        fn complete_lines(&self) -> CompleteLines {
            CompleteLines::new(self)
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn complete_lines() {
            assert_eq!(CompleteLines::new("").count(), 0);
            assert_eq!(CompleteLines::new("\n").count(), 1);
            assert_eq!(CompleteLines::new("\n\n").count(), 2);
            assert_eq!(CompleteLines::new("one").count(), 1);
            assert_eq!(CompleteLines::new("one\n").count(), 1);
            assert_eq!(CompleteLines::new("one\n").next(), Some("one\n"));
            assert_eq!(CompleteLines::new("one\ntwo\n").count(), 2);
            assert_eq!(CompleteLines::new("one\ntwo\n").next(), Some("one\n"));
            assert_eq!(CompleteLines::new("one\ntwo\n").last(), Some("two\n"));
        }

        #[test]
        fn complet_lines_ext() {
            let string = "one\n\ntwo\nthree".to_string();
            let mut iter = string.complete_lines();
            assert_eq!(iter.next(), Some("one\n"));
            assert_eq!(iter.next(), Some("\n"));
            assert_eq!(iter.next(), Some("two\n"));
            assert_eq!(iter.next(), Some("three"));
            assert_eq!(iter.next(), None);
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

    #[cfg(test)]
    mod tests {
        use super::*;

        fn lines_from_string(string: &str) -> Vec<&str> {
            let mut lines: Vec<&str> = vec![];
            let mut start_index = 0;
            for (end_index, _) in string.match_indices('\n') {
                lines.push(&string[start_index..=end_index]);
                start_index = end_index + 1;
            }
            if start_index < string.len() {
                lines.push(&string[start_index..]);
            }
            lines
        }

        #[test]
        fn source_lines() {
            let string = " #[derive(Debug)]
 pub enum DiffParseError {
     MissingAfterFileData(usize),
+    ParseNumberError(ParseIntError),
+    UnexpectedEndOfInput,
-    CatchAll,
+    UnexpectedEndHunk(DiffFormat, usize),
+    SyntaxError(DiffFormat, usize),
 }
"
            .to_string();
            let lines = lines_from_string(&string);
            let source_lines = extract_source_lines(&lines, 1, |l| l.starts_with('+'));
            assert_eq!(source_lines.len(), 5);
            assert_eq!(source_lines[0], "#[derive(Debug)]\n");
            assert_eq!(source_lines[1], "pub enum DiffParseError {\n");
            assert_eq!(source_lines[2], "    MissingAfterFileData(usize),\n");
            assert_eq!(source_lines[3], "    CatchAll,\n");
            assert_eq!(source_lines[4], "}\n");
            let source_lines = extract_source_lines(&lines, 1, |l| l.starts_with('-'));
            assert_eq!(source_lines.len(), 8);
            assert_eq!(source_lines[0], "#[derive(Debug)]\n");
            assert_eq!(source_lines[1], "pub enum DiffParseError {\n");
            assert_eq!(source_lines[2], "    MissingAfterFileData(usize),\n");
            assert_eq!(source_lines[3], "    ParseNumberError(ParseIntError),\n");
            assert_eq!(source_lines[4], "    UnexpectedEndOfInput,\n");
            assert_eq!(
                source_lines[5],
                "    UnexpectedEndHunk(DiffFormat, usize),\n"
            );
            assert_eq!(source_lines[6], "    SyntaxError(DiffFormat, usize),\n");
            assert_eq!(source_lines[7], "}\n");
        }
    }
}
