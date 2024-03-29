// Copyright 2019 Peter Williams <pwil3058@gmail.com>

use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::slice::Iter;

use regex::Captures;

use pw_pathux::str_path::*;

use crate::abstract_diff::{AbstractDiff, AbstractHunk, ApplnResult};
use crate::git_binary_diff::git_delta::DeltaError;
use crate::lines::*;
use crate::DiffFormat;
use crate::MultiListIter;

// TODO: implement Error for DiffParseError
#[derive(Debug)]
pub enum DiffParseError {
    MissingAfterFileData(usize),
    ParseNumberError(ParseIntError, usize),
    UnexpectedEndOfInput,
    UnexpectedEndHunk(DiffFormat, usize),
    UnexpectedInput(DiffFormat, String),
    SyntaxError(DiffFormat, usize),
    Base85Error(String),
    ZLibInflateError(String),
    GitDeltaError(DeltaError),
    IOError(io::Error),
}

impl fmt::Display for DiffParseError {
    // TODO: flesh out fmt::Display implementation for DiffParseError
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bug the developer to fix this!")
    }
}

impl Error for DiffParseError {
    // TODO: flesh out error::Error implementation for DiffParseError
    fn description(&self) -> &str {
        "I'm the superhero of diff parsing errors"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

pub type DiffParseResult<T> = Result<T, DiffParseError>;

#[derive(Debug, PartialEq, Clone)]
pub struct PathAndTimestamp {
    file_path: String,
    time_stamp: Option<String>,
}

#[derive(Debug)]
pub struct TextDiffHeader {
    pub lines: Lines,
    pub ante_pat: PathAndTimestamp,
    pub post_pat: PathAndTimestamp,
}

pub trait TextDiffHunk {
    fn len(&self) -> usize;
    fn iter(&self) -> Iter<Line>;

    fn ante_lines(&self) -> Lines;
    fn post_lines(&self) -> Lines;

    fn adds_trailing_white_space(&self) -> bool;

    fn get_abstract_diff_hunk(&self) -> AbstractHunk;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct TextDiff<H: TextDiffHunk> {
    lines_consumed: usize, // time saver
    header: TextDiffHeader,
    hunks: Vec<H>,
}

impl<H> TextDiff<H>
where
    H: TextDiffHunk,
{
    pub fn len(&self) -> usize {
        self.lines_consumed
    }

    pub fn is_empty(&self) -> bool {
        self.lines_consumed == 0
    }

    pub fn header(&self) -> &TextDiffHeader {
        &self.header
    }

    pub fn hunks(&self) -> &Vec<H> {
        &self.hunks
    }

    pub fn iter(&self) -> MultiListIter<Line> {
        let mut list = Vec::new();
        list.push(self.header.lines.iter());
        for hunk in self.hunks.iter() {
            list.push(hunk.iter())
        }
        MultiListIter::<Line>::new(list)
    }

    pub fn get_ante_file_path(&self, strip_level: usize) -> String {
        self.header
            .ante_pat
            .file_path
            .path_stripped_of_n_levels(strip_level)
    }

    pub fn get_post_file_path(&self, strip_level: usize) -> String {
        self.header
            .post_pat
            .file_path
            .path_stripped_of_n_levels(strip_level)
    }

    pub fn get_file_path(&self, strip_level: usize) -> String {
        if self.header.post_pat.file_path != "/dev/null" {
            self.header
                .post_pat
                .file_path
                .path_stripped_of_n_levels(strip_level)
        } else {
            self.header
                .ante_pat
                .file_path
                .path_stripped_of_n_levels(strip_level)
        }
    }

    pub fn adds_trailing_white_space(&self) -> bool {
        for hunk in self.hunks.iter() {
            if hunk.adds_trailing_white_space() {
                return true;
            }
        }
        false
    }

    pub fn apply_to_lines<W>(
        &mut self,
        lines: &[Line],
        reverse: bool,
        err_w: &mut W,
        repd_file_path: Option<&str>,
    ) -> ApplnResult
    where
        W: io::Write,
    {
        let hunks = self
            .hunks
            .iter()
            .map(|h| h.get_abstract_diff_hunk())
            .collect();
        let abstract_diff = AbstractDiff::new(hunks);
        abstract_diff.apply_to_lines(lines, reverse, err_w, repd_file_path)
    }

    pub fn apply_to_contents<R, W>(
        &mut self,
        reader: &mut R,
        reverse: bool,
        err_w: &mut W,
        repd_file_path: Option<&str>,
    ) -> DiffParseResult<ApplnResult>
    where
        R: io::Read,
        W: io::Write,
    {
        let lines = Lines::read(reader).map_err(DiffParseError::IOError)?;
        Ok(self.apply_to_lines(&lines, reverse, err_w, repd_file_path))
    }
}

pub trait TextDiffParser<H: TextDiffHunk> {
    fn new() -> Self;
    fn ante_file_rec<'t>(&self, line: &'t Line) -> Option<Captures<'t>>;
    fn post_file_rec<'t>(&self, line: &'t Line) -> Option<Captures<'t>>;
    fn get_hunk_at(&self, lines: &[Line], index: usize) -> DiffParseResult<Option<H>>;

    fn _get_file_data_fm_captures(&self, captures: &Captures) -> PathAndTimestamp {
        let file_path = if let Some(path) = captures.get(2) {
            path.as_str()
        } else {
            captures.get(3).unwrap().as_str() // TODO: confirm unwrap is OK here
        };
        let time_stamp = captures.get(4).map(|ts| ts.as_str().to_string());
        PathAndTimestamp {
            file_path: file_path.to_string(),
            time_stamp,
        }
    }

    fn get_text_diff_header_at(
        &self,
        lines: &[Line],
        start_index: usize,
    ) -> DiffParseResult<Option<TextDiffHeader>> {
        let ante_pat = if let Some(ref captures) = self.ante_file_rec(&lines[start_index]) {
            self._get_file_data_fm_captures(captures)
        } else {
            return Ok(None);
        };
        let post_pat = if let Some(ref captures) = self.post_file_rec(&lines[start_index + 1]) {
            self._get_file_data_fm_captures(captures)
        } else {
            return Err(DiffParseError::MissingAfterFileData(start_index));
        };
        let lines = lines[start_index..start_index + 2].to_vec();
        Ok(Some(TextDiffHeader {
            lines,
            ante_pat,
            post_pat,
        }))
    }

    fn get_diff_at(
        &self,
        lines: &[Line],
        start_index: usize,
    ) -> DiffParseResult<Option<TextDiff<H>>> {
        if lines.len() - start_index < 2 {
            return Ok(None);
        }
        let mut index = start_index;
        let header = if let Some(header) = self.get_text_diff_header_at(lines, index)? {
            index += header.lines.len();
            header
        } else {
            return Ok(None);
        };
        let mut hunks: Vec<H> = Vec::new();
        while index < lines.len() {
            if let Some(hunk) = self.get_hunk_at(lines, index)? {
                index += hunk.len();
                hunks.push(hunk);
            } else {
                break;
            }
        }
        let diff = TextDiff::<H> {
            lines_consumed: index - start_index,
            header,
            hunks,
        };
        Ok(Some(diff))
    }
}

pub fn extract_source_lines<F: Fn(&Line) -> bool>(
    lines: &[Line],
    trim_left_n: usize,
    skip: F,
) -> Lines {
    let mut trimmed_lines: Lines = vec![];
    for (index, line) in lines.iter().enumerate() {
        if skip(line) || line.starts_with('\\') {
            continue;
        }
        if (index + 1) == lines.len() || !lines[index + 1].starts_with('\\') {
            trimmed_lines.push(Arc::new(line[trim_left_n..].to_string()));
        } else {
            trimmed_lines.push(Arc::new(
                line[trim_left_n..].trim_end_matches('\n').to_string(),
            ));
        }
    }
    trimmed_lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::{Captures, Regex};

    use crate::abstract_diff::AbstractChunk;
    use crate::{ALT_TIMESTAMP_RE_STR, PATH_RE_STR, TIMESTAMP_RE_STR};

    #[derive(Debug)]
    struct DummyDiffParser {
        ante_file_cre: Regex,
        post_file_cre: Regex,
    }

    struct DummyDiffHunk {
        lines: Lines,
    }

    impl TextDiffHunk for DummyDiffHunk {
        fn len(&self) -> usize {
            self.lines.len()
        }

        fn iter(&self) -> Iter<Line> {
            self.lines.iter()
        }

        fn ante_lines(&self) -> Lines {
            vec![]
        }

        fn post_lines(&self) -> Lines {
            vec![]
        }

        fn get_abstract_diff_hunk(&self) -> AbstractHunk {
            let a1 = AbstractChunk {
                start_index: 1,
                lines: Vec::<Line>::new(),
            };
            let a2 = AbstractChunk {
                start_index: 1,
                lines: Vec::<Line>::new(),
            };
            AbstractHunk::new(a1, a2)
        }

        fn adds_trailing_white_space(&self) -> bool {
            false
        }
    }

    impl TextDiffParser<DummyDiffHunk> for DummyDiffParser {
        fn new() -> Self {
            let e_ts_re_str = format!("({TIMESTAMP_RE_STR}|{ALT_TIMESTAMP_RE_STR})");
            let e = format!(r"^--- ({PATH_RE_STR})(\s+{e_ts_re_str})?(.*)(\n)?$");
            let ante_file_cre = Regex::new(&e).unwrap();
            let e_ts_re_str = format!("({TIMESTAMP_RE_STR}|{ALT_TIMESTAMP_RE_STR})");
            let e = format!(r"^\+\+\+ ({PATH_RE_STR})(\s+{e_ts_re_str})?(.*)(\n)?$");
            let post_file_cre = Regex::new(&e).unwrap();
            DummyDiffParser {
                ante_file_cre,
                post_file_cre,
            }
        }

        fn ante_file_rec<'t>(&self, line: &'t Line) -> Option<Captures<'t>> {
            self.ante_file_cre.captures(line)
        }

        fn post_file_rec<'t>(&self, line: &'t Line) -> Option<Captures<'t>> {
            self.post_file_cre.captures(line)
        }

        fn get_hunk_at(
            &self,
            _lines: &[Line],
            _index: usize,
        ) -> DiffParseResult<Option<DummyDiffHunk>> {
            Ok(None)
        }
    }

    #[test]
    fn get_file_data_works() {
        let mut lines: Lines = Vec::new();
        lines.push(Line::new("--- a/path/to/original\n".to_string()));
        lines.push(Line::new("+++ b/path/to/new\n".to_string()));
        let ddp = DummyDiffParser::new();
        let tdh = ddp.get_text_diff_header_at(&lines, 0).unwrap().unwrap();
        assert_eq!(
            tdh.ante_pat,
            PathAndTimestamp {
                file_path: String::from("a/path/to/original"),
                time_stamp: None
            }
        );
        assert_eq!(
            tdh.post_pat,
            PathAndTimestamp {
                file_path: String::from("b/path/to/new"),
                time_stamp: None
            }
        );
    }
}
