//Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::slice::Iter;
use std::str::FromStr;

use regex::{Captures, Regex};

use crate::abstract_diff::{AbstractChunk, AbstractHunk};
use crate::lines::{Line, LineIfce, Lines};
use crate::text_diff::{
    extract_source_lines, DiffParseError, DiffParseResult, TextDiff, TextDiffHunk, TextDiffParser,
};
use crate::{DiffFormat, ALT_TIMESTAMP_RE_STR, PATH_RE_STR, TIMESTAMP_RE_STR};

pub struct ContextDiffChunk {
    offset: usize,
    start_line_num: usize,
    _length: usize,
    numlines: usize,
}

pub struct ContextDiffHunk {
    pub lines: Lines,
    pub ante_chunk: ContextDiffChunk,
    pub post_chunk: ContextDiffChunk,
}

pub type ContextDiff = TextDiff<ContextDiffHunk>;

impl TextDiffHunk for ContextDiffHunk {
    fn len(&self) -> usize {
        self.lines.len()
    }

    fn iter(&self) -> Iter<Line> {
        self.lines.iter()
    }

    fn ante_lines(&self) -> Lines {
        if self.ante_chunk.numlines == 1 {
            let start = self.post_chunk.offset;
            let end = self.post_chunk.offset + self.post_chunk.numlines;
            extract_source_lines(&self.lines[start..end], 2, |l| l.starts_with('+'))
        } else {
            let start = self.ante_chunk.offset;
            let end = self.ante_chunk.offset + self.ante_chunk.numlines;
            extract_source_lines(&self.lines[start..end], 2, |_| false)
        }
    }

    fn post_lines(&self) -> Lines {
        let start = self.post_chunk.offset;
        let end = self.post_chunk.offset + self.post_chunk.numlines;
        extract_source_lines(&self.lines[start..end], 2, |_| false)
    }

    fn adds_trailing_white_space(&self) -> bool {
        let start = self.post_chunk.offset;
        let end = self.post_chunk.offset + self.post_chunk.numlines;
        for line in self.lines[start..end].iter() {
            if (line.starts_with('+') || line.starts_with('!')) && line.has_trailing_white_space() {
                return true;
            }
        }
        false
    }

    fn get_abstract_diff_hunk(&self) -> AbstractHunk {
        // NB: convert starting line numbers to 0 based indices
        let ante_chunk = AbstractChunk {
            start_index: self.ante_chunk.start_line_num - 1,
            lines: self.ante_lines(),
        };
        let post_chunk = AbstractChunk {
            start_index: self.post_chunk.start_line_num - 1,
            lines: self.post_lines(),
        };
        AbstractHunk::new(ante_chunk, post_chunk)
    }
}

pub struct ContextDiffParser {
    ante_file_cre: Regex,
    post_file_cre: Regex,
    hunk_start_cre: Regex,
    hunk_ante_cre: Regex,
    hunk_post_cre: Regex,
}

impl ContextDiffParser {
    fn start_and_length_from_captures(
        captures: Captures,
        line_number: usize,
    ) -> DiffParseResult<(usize, usize)> {
        let start: usize = if let Some(capture) = captures.get(1) {
            usize::from_str(capture.as_str())
                .map_err(|e| DiffParseError::ParseNumberError(e, line_number))?
        } else {
            return Err(DiffParseError::SyntaxError(
                DiffFormat::Context,
                line_number,
            ));
        };
        let finish: usize = if let Some(capture) = captures.get(3) {
            usize::from_str(capture.as_str())
                .map_err(|e| DiffParseError::ParseNumberError(e, line_number))?
        } else {
            start
        };
        let length = if start == 0 && finish == 0 {
            0
        } else {
            finish - start + 1
        };

        Ok((start, length))
    }

    fn get_ante_sal_at(
        &self,
        lines: &[Line],
        start_index: usize,
    ) -> DiffParseResult<(usize, usize)> {
        if let Some(captures) = self.hunk_ante_cre.captures(&lines[start_index]) {
            Self::start_and_length_from_captures(captures, start_index + 1)
        } else {
            Err(DiffParseError::SyntaxError(
                DiffFormat::Context,
                start_index + 1,
            ))
        }
    }

    fn get_post_sal_at(
        &self,
        lines: &[Line],
        start_index: usize,
    ) -> DiffParseResult<Option<(usize, usize)>> {
        if let Some(captures) = self.hunk_post_cre.captures(&lines[start_index]) {
            let sal = Self::start_and_length_from_captures(captures, start_index + 1)?;
            Ok(Some(sal))
        } else {
            Ok(None)
        }
    }
}

impl Default for ContextDiffParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TextDiffParser<ContextDiffHunk> for ContextDiffParser {
    fn new() -> ContextDiffParser {
        let e_ts_re_str = format!("({TIMESTAMP_RE_STR}|{ALT_TIMESTAMP_RE_STR})");
        let ante_file_cre_str = format!(r"^\*\*\* ({PATH_RE_STR})(\s+{e_ts_re_str})?(\n)?$");
        let post_file_cre_str = format!(r"^--- ({PATH_RE_STR})(\s+{e_ts_re_str})?(\n)?$");

        ContextDiffParser {
            ante_file_cre: Regex::new(&ante_file_cre_str).unwrap(),
            post_file_cre: Regex::new(&post_file_cre_str).unwrap(),
            hunk_start_cre: Regex::new(r"^\*{15}\s*(.*)(\n)?$").unwrap(),
            hunk_ante_cre: Regex::new(r"^\*\*\*\s+(\d+)(,(\d+))?\s+\*\*\*\*\s*(.*)(\n)?$").unwrap(),
            hunk_post_cre: Regex::new(r"^---\s+(\d+)(,(\d+))?\s+----(.*)(\n)?$").unwrap(),
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
        lines: &[Line],
        start_index: usize,
    ) -> DiffParseResult<Option<ContextDiffHunk>> {
        if !self.hunk_start_cre.is_match(&lines[start_index]) {
            return Ok(None);
        }
        let ante_start_index = start_index + 1;
        let ante_sal = self.get_ante_sal_at(lines, ante_start_index)?;
        let mut index = ante_start_index + 1;
        let mut ante_count = 0;
        let mut post_count = 0;
        let mut o_post_sal: Option<(usize, usize)> = None;
        let mut post_start_index = index;
        while ante_count < ante_sal.1 {
            post_start_index = index;
            o_post_sal = self.get_post_sal_at(lines, index)?;
            if o_post_sal.is_some() {
                break;
            }
            ante_count += 1;
            index += 1;
        }
        if o_post_sal.is_none() {
            if lines[index].starts_with(r"\ ") {
                index += 1;
            }
            post_start_index = index;
            o_post_sal = self.get_post_sal_at(lines, index)?;
            if o_post_sal.is_none() {
                return Err(DiffParseError::SyntaxError(DiffFormat::Context, index + 1));
            }
        }
        let post_sal = o_post_sal.unwrap();
        while post_count < post_sal.1 {
            if !(lines[index].starts_with("! ")
                || lines[index].starts_with("+ ")
                || lines[index].starts_with(' '))
            {
                if post_count == 0 {
                    break;
                }
                return Err(DiffParseError::SyntaxError(DiffFormat::Context, index + 1));
            }
            post_count += 1;
            index += 1;
        }
        if index < lines.len() && lines[index].starts_with(r"\ ") {
            index += 1;
        }
        let ante_chunk = ContextDiffChunk {
            offset: ante_start_index - start_index,
            start_line_num: ante_sal.0,
            _length: ante_sal.1,
            numlines: post_start_index - ante_start_index,
        };
        let post_chunk = ContextDiffChunk {
            offset: post_start_index - start_index,
            start_line_num: post_sal.0,
            _length: post_sal.1,
            numlines: index - post_start_index,
        };
        let hunk = ContextDiffHunk {
            lines: lines[start_index..index].to_vec(),
            ante_chunk,
            post_chunk,
        };
        Ok(Some(hunk))
    }
}

#[cfg(test)]
mod tests {
    use crate::context_diff::ContextDiffParser;
    use crate::lines::{Lines, LinesIfce};
    use crate::text_diff::TextDiffParser;

    static CONTEXT_DIFF: &str = "*** lao	2002-02-21 23:30:39.942229878 -0800
--- tzu	2002-02-21 23:30:50.442260588 -0800
***************
*** 1,7 ****
- The Way that can be told of is not the eternal Way;
- The name that can be named is not the eternal name.
  The Nameless is the origin of Heaven and Earth;
! The Named is the mother of all things.
  Therefore let there always be non-being,
    so we may see their subtlety,
  And let there always be being,
--- 1,6 ----
  The Nameless is the origin of Heaven and Earth;
! The named is the mother of all things.
! 
  Therefore let there always be non-being,
    so we may see their subtlety,
  And let there always be being,
***************
*** 9,11 ****
--- 8,13 ----
  The two are the same,
  But after they are produced,
    they have different names.
+ They both may be called deep and profound.
+ Deeper and more profound,
+ The door of all subtleties!
";

    #[test]
    fn context_diff_parse() {
        let diff_lines = Lines::from_string(CONTEXT_DIFF);
        let parser = ContextDiffParser::new();
        assert!(parser.get_diff_at(&diff_lines, 0).is_ok());
        assert!(parser.get_diff_at(&diff_lines, 0).unwrap().is_some());
        assert!(parser.get_diff_at(&diff_lines, 1).unwrap().is_none());
    }
}
