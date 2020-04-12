// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::lines::{first_inequality_fm_head, first_inequality_fm_tail, LinesIfce};

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
        let to_index = self.lines.len() - context.1;
        for fm_index in 0..context.0 {
            if let Some(start_index) =
                lines.find_first_sub_lines(&self.lines[fm_index..to_index], adj_start_index)
            {
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
