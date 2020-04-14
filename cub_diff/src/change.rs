// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

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
        offset: isize,
        context: (usize, usize),
    ) -> Option<(usize, usize, usize)> {
        let adj_start_index = (self.start_index as isize + offset) as usize;
        for head_redn in 0..context.0 {
            for tail_redn in 0..context.1 {
                let to_index = self.lines.len() - tail_redn;
                if let Some(start_index) =
                    lines.find_first_sub_lines(&self.lines[head_redn..to_index], adj_start_index)
                {
                    return Some((start_index, head_redn, tail_redn));
                }
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Change<'a, 'b> {
    ante_snippet: Snippet<'a>,
    post_snippet: Snippet<'b>,
    head_context_len: usize,
    tail_context_len: usize,
}

impl<'a, 'b> Change<'a, 'b> {
    pub fn new(ante_snippet: Snippet<'a>, post_snippet: Snippet<'b>) -> Self {
        let head_context_len =
            first_inequality_fm_head(&ante_snippet.lines, &post_snippet.lines).unwrap();
        let tail_context_len =
            first_inequality_fm_tail(&ante_snippet.lines, &post_snippet.lines).unwrap();
        Self {
            ante_snippet,
            post_snippet,
            head_context_len,
            tail_context_len,
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
        }
    }
    changed
}
