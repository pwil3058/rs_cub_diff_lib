// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{convert::From, fmt};

use crate::lines::{
    first_inequality_fm_head, first_inequality_fm_tail, CompleteLinesExt, LinesIfce,
};

#[derive(Debug)]
pub struct Snippet<'a> {
    start_index: usize,
    lines: Vec<&'a str>,
}

impl<'a> Snippet<'a> {
    fn matches_lines(&self, lines: &[&str], offset: isize) -> bool {
        let start_index = (self.start_index as isize + offset) as usize;
        lines.contains_sub_lines_at(&self.lines, start_index)
    }

    fn match_lines_fuzzy(
        &self,
        lines: &[&str],
        start_index: usize,
        context: Context,
    ) -> Option<(usize, usize, usize)> {
        for head_redn in 0..context.head_len {
            for tail_redn in 0..context.tail_len {
                let to_index = self.lines.len() - tail_redn;
                if let Some(found_start_index) =
                    lines.find_first_sub_lines(&self.lines[head_redn..to_index], start_index)
                {
                    return Some((found_start_index, head_redn, tail_redn));
                }
            }
        }
        None
    }
}

#[derive(Debug, Default)]
pub struct Context {
    pub head_len: usize,
    pub tail_len: usize,
}

impl Context {
    pub fn len(&self) -> usize {
        self.head_len + self.tail_len
    }

    pub fn empty(&self) -> bool {
        self.len() == 0
    }
}

impl From<(&[&str], &[&str])> for Context {
    fn from(lines: (&[&str], &[&str])) -> Self {
        let head_len = first_inequality_fm_head(lines.0, lines.1).expect("programmer erroe");
        let tail_len = first_inequality_fm_tail(lines.0, lines.1).expect("programmer erroe");
        Self { head_len, tail_len }
    }
}
impl From<(&Vec<&str>, &Vec<&str>)> for Context {
    fn from(lines: (&Vec<&str>, &Vec<&str>)) -> Self {
        let head_len = first_inequality_fm_head(&lines.0, &lines.1).expect("programmer erroe");
        let tail_len = first_inequality_fm_tail(&lines.0, &lines.1).expect("programmer erroe");
        Self { head_len, tail_len }
    }
}

#[derive(Debug)]
pub struct Change<'a, 'b> {
    ante_snippet: Snippet<'a>,
    post_snippet: Snippet<'b>,
    context: Context,
}

impl<'a, 'b> Change<'a, 'b> {
    pub fn new(ante_snippet: Snippet<'a>, post_snippet: Snippet<'b>) -> Self {
        let context = Context::from((&ante_snippet.lines, &post_snippet.lines));
        Self {
            ante_snippet,
            post_snippet,
            context,
        }
    }
}

#[derive(Debug, Default)]
pub struct Changed {
    string: String,
    successes: u64,
    merges: u64,
    already_applied: u64,
    failures: u64,
}

pub fn apply_changes(changes: &[Change], text: &str) -> Changed {
    let lines: Vec<&str> = text.complete_lines().collect();
    let mut changed = Changed::default();
    let mut current_offset: isize = 0;
    let mut lines_index: usize = 0;
    for change in changes.iter() {
        let ante = &change.ante_snippet;
        let post = &change.post_snippet;
        if ante.matches_lines(&lines, current_offset) {
            let index = (ante.start_index as isize + current_offset) as usize;
            for line in &lines[lines_index..index] {
                changed.string += line;
            }
            for line in &post.lines {
                changed.string += line;
            }
            lines_index =
                (((ante.start_index + ante.lines.len()) as isize) + current_offset) as usize;
            changed.successes += 1;
            continue;
        };
        // if let Some((index, head_context_redn, tail_context_redn)) = ante.match_lines_fuzzy(
        //     &lines,
        //     lines_index,
        //     (change.head_context_len, change.tail_context_len),
        // ) {
        //     for line in &lines[lines_index..index] {
        //         changed.string += line;
        //     }
        //     let end = ante.lines.len() - tail_context_redn;
        //     for line in &ante.lines[head_context_redn..end] {
        //         changed.string += line;
        //     }
        //     lines_index = index + ante.lines.len() - head_context_redn - tail_context_redn;
        //     current_offset =
        //         index as isize - ante.start_index as isize - head_context_redn as isize;
        //     let applied_posn =
        //         hunk.get_applied_posn(result.lines.len(), cpd.post_context_redn, reverse);
        //     if let Some(file_path) = repd_file_path {
        //         writeln!(
        //             err_w,
        //             "{}: Hunk #{} merged at {}.",
        //             file_path,
        //             hunk_index + 1,
        //             applied_posn
        //         )
        //         .unwrap();
        //     } else {
        //         writeln!(
        //             err_w,
        //             "Hunk #{} merged at {}.",
        //             hunk_index + 1,
        //             applied_posn
        //         )
        //         .unwrap();
        //     }
        //     result.merges += 1;
        //     continue;
        // }
    }
    changed
}

#[derive(Debug)]
pub struct AppliedPosnData {
    start_posn: usize,
    length: usize,
}

impl fmt::Display for AppliedPosnData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line {} ({} lines)", self.start_posn, self.length)
    }
}
