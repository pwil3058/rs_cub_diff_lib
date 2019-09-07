// Copyright 2019 Peter Williams <pwil3058@gmail.com>

use std::io::Write;
use std::sync::Arc;

use crypto_hash::{Algorithm, Hasher};

use crate::context_diff::{ContextDiff, ContextDiffParser};
use crate::git_binary_diff::{GitBinaryDiff, GitBinaryDiffParser};
use crate::lines::Line;
use crate::preamble::{GitPreamble, Preamble, PreambleIfce, PreambleParser};
use crate::text_diff::{DiffParseResult, TextDiffParser};
use crate::unified_diff::{UnifiedDiff, UnifiedDiffParser};
use crate::MultiListIter;

pub enum Diff {
    Unified(UnifiedDiff),
    Context(ContextDiff),
    GitBinary(GitBinaryDiff),
    GitPreambleOnly(GitPreamble),
}

impl Diff {
    pub fn len(&self) -> usize {
        match self {
            Diff::Unified(diff) => diff.len(),
            Diff::Context(diff) => diff.len(),
            Diff::GitBinary(diff) => diff.len(),
            Diff::GitPreambleOnly(diff) => diff.len(),
        }
    }

    pub fn iter(&self) -> MultiListIter<Line> {
        match self {
            Diff::Unified(diff) => diff.iter(),
            Diff::Context(diff) => diff.iter(),
            Diff::GitBinary(diff) => MultiListIter::new(vec![diff.iter()]),
            Diff::GitPreambleOnly(diff) => MultiListIter::new(vec![diff.iter()]),
        }
    }

    pub fn get_ante_file_path(&self, strip_level: usize) -> Option<String> {
        match self {
            Diff::Unified(diff) => Some(diff.get_ante_file_path(strip_level)),
            Diff::Context(diff) => Some(diff.get_ante_file_path(strip_level)),
            Diff::GitBinary(_) => None,
            Diff::GitPreambleOnly(preamble) => Some(preamble.get_ante_file_path(strip_level)),
        }
    }

    pub fn get_post_file_path(&self, strip_level: usize) -> Option<String> {
        match self {
            Diff::Unified(diff) => Some(diff.get_post_file_path(strip_level)),
            Diff::Context(diff) => Some(diff.get_post_file_path(strip_level)),
            Diff::GitBinary(_) => None,
            Diff::GitPreambleOnly(preamble) => Some(preamble.get_post_file_path(strip_level)),
        }
    }

    pub fn get_file_path(&self, strip_level: usize) -> Option<String> {
        match self {
            Diff::Unified(diff) => Some(diff.get_file_path(strip_level)),
            Diff::Context(diff) => Some(diff.get_file_path(strip_level)),
            Diff::GitBinary(_) => None,
            Diff::GitPreambleOnly(preamble) => Some(preamble.get_file_path(strip_level)),
        }
    }

    pub fn adds_trailing_white_space(&self) -> bool {
        match self {
            Diff::Unified(diff) => diff.adds_trailing_white_space(),
            Diff::Context(diff) => diff.adds_trailing_white_space(),
            _ => false,
        }
    }
}

pub struct DiffParser {
    context_diff_parser: ContextDiffParser,
    unified_diff_parser: UnifiedDiffParser,
    git_binary_diff_parser: GitBinaryDiffParser,
}

impl DiffParser {
    pub fn new() -> DiffParser {
        DiffParser {
            context_diff_parser: ContextDiffParser::new(),
            unified_diff_parser: UnifiedDiffParser::new(),
            git_binary_diff_parser: GitBinaryDiffParser::new(),
        }
    }

    pub fn get_diff_at(&self, lines: &[Line], start_index: usize) -> DiffParseResult<Option<Diff>> {
        // try diff types in occurence likelihood order
        if let Some(result) = self.unified_diff_parser.get_diff_at(lines, start_index)? {
            Ok(Some(Diff::Unified(result)))
        } else if let Some(result) = self
            .git_binary_diff_parser
            .get_diff_at(lines, start_index)?
        {
            Ok(Some(Diff::GitBinary(result)))
        } else if let Some(result) = self.context_diff_parser.get_diff_at(lines, start_index)? {
            Ok(Some(Diff::Context(result)))
        } else {
            Ok(None)
        }
    }
}

pub struct DiffPlus {
    preamble: Option<Preamble>,
    diff: Diff,
}

impl DiffPlus {
    pub fn len(&self) -> usize {
        if let Some(ref preamble) = self.preamble {
            preamble.len() + self.diff.len()
        } else {
            self.diff.len()
        }
    }

    pub fn iter(&self) -> MultiListIter<Line> {
        let mut iter = self.diff.iter();
        if let Some(preamble) = &self.preamble {
            iter.prepend(preamble.iter());
        };
        iter
    }

    pub fn preamble(&self) -> &Option<Preamble> {
        &self.preamble
    }

    pub fn diff(&self) -> &Diff {
        &self.diff
    }

    pub fn get_ante_file_path(&self, strip_level: usize) -> String {
        if let Some(ref preamble) = self.preamble {
            preamble.get_ante_file_path(strip_level)
        } else {
            // unwrap() should be safe as binary patches always have a git preamble
            self.diff.get_ante_file_path(strip_level).unwrap()
        }
    }

    pub fn get_post_file_path(&self, strip_level: usize) -> String {
        if let Some(ref preamble) = self.preamble {
            preamble.get_post_file_path(strip_level)
        } else {
            // unwrap() should be safe as binary patches always have a git preamble
            self.diff.get_post_file_path(strip_level).unwrap()
        }
    }

    pub fn get_file_path(&self, strip_level: usize) -> String {
        if let Some(ref preamble) = self.preamble {
            preamble.get_file_path(strip_level)
        } else {
            // unwrap() should be safe as binary patches always have a git preamble
            self.diff.get_file_path(strip_level).unwrap()
        }
    }

    pub fn adds_trailing_white_space(&self) -> bool {
        self.diff.adds_trailing_white_space()
    }

    pub fn hash_digest(&self) -> Vec<u8> {
        let mut hasher = Hasher::new(Algorithm::SHA256);
        if let Some(preamble) = &self.preamble {
            for line in preamble.iter() {
                hasher
                    .write_all(&line.as_bytes())
                    .expect("hasher blew up!!!");
            }
        };
        for line in self.diff.iter() {
            hasher
                .write_all(&line.as_bytes())
                .expect("hasher blew up!!!");
        }
        hasher.finish()
    }
}

pub struct DiffPlusParser {
    preamble_parser: PreambleParser,
    diff_parser: DiffParser,
}

impl DiffPlusParser {
    pub fn new() -> DiffPlusParser {
        DiffPlusParser {
            preamble_parser: PreambleParser::new(),
            diff_parser: DiffParser::new(),
        }
    }

    pub fn get_diff_plus_at(
        &self,
        lines: &[Line],
        start_index: usize,
    ) -> DiffParseResult<Option<DiffPlus>> {
        if let Some(preamble) = self.preamble_parser.get_preamble_at(lines, start_index) {
            if let Some(diff) = self
                .diff_parser
                .get_diff_at(lines, start_index + preamble.len())?
            {
                Ok(Some(DiffPlus {
                    preamble: Some(preamble),
                    diff,
                }))
            } else if let Preamble::Git(git_preamble) = preamble {
                Ok(Some(DiffPlus {
                    preamble: None,
                    diff: Diff::GitPreambleOnly(git_preamble),
                }))
            } else {
                Ok(None)
            }
        } else if let Some(diff) = self.diff_parser.get_diff_at(lines, start_index)? {
            Ok(Some(DiffPlus {
                preamble: None,
                diff,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn parse_lines(&self, lines: &[Line]) -> DiffParseResult<Vec<Arc<DiffPlus>>> {
        let mut diff_pluses = vec![];
        let mut index = 0;
        while index < lines.len() {
            if let Some(diff_plus) = self.get_diff_plus_at(lines, index)? {
                index += diff_plus.len();
                diff_pluses.push(Arc::new(diff_plus));
            } else {
                index += 1;
            }
        }
        Ok(diff_pluses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use crate::lines::*;

    #[test]
    fn get_diff_plus_at_works_for_text_diffs() {
        let lines = Lines::read_from(&Path::new("./test_diffs/test_1.diff")).unwrap();
        let parser = DiffPlusParser::new();
        let result = parser.get_diff_plus_at(&lines, 0);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        let result = parser.get_diff_plus_at(&lines, 12);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let diff = result.unwrap();
        assert!(diff.iter().count() == diff.len());
    }

    #[test]
    fn get_diff_plus_at_works_for_binary_diffs() {
        let lines = Lines::read_from(&Path::new("./test_diffs/test_2.binary_diff")).unwrap();
        let parser = DiffPlusParser::new();
        let result = parser.get_diff_plus_at(&lines, 0);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        let result = parser.get_diff_plus_at(&lines, 12);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let diff = result.unwrap();
        assert!(diff.iter().count() == diff.len());

        for start_index in &[0, 9, 19, 28, 37, 46] {
            let result = parser.get_diff_plus_at(&lines, *start_index);
            assert!(result.is_ok());
            let result = result.unwrap();
            assert!(result.is_some());
            let diff = result.unwrap();
            assert!(diff.iter().count() == diff.len());
        }
    }
}
