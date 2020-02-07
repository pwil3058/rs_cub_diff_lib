//Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

extern crate crypto_hash;
extern crate inflate;
extern crate lcs;
extern crate regex;
#[macro_use]
extern crate lazy_static;

extern crate pw_pathux;

use std::slice::Iter;

pub mod abstract_diff;
pub mod context_diff;
pub mod diff;
pub mod diff_stats;
pub mod git_binary_diff;
pub mod lines;
pub mod patch;
pub mod preamble;
pub mod text_diff;
pub mod unified_diff;

pub const TIMESTAMP_RE_STR: &str = r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}(\.\d{9})? [-+]{1}\d{4}";
pub const ALT_TIMESTAMP_RE_STR: &str =
    r"[A-Z][a-z]{2} [A-Z][a-z]{2} \d{2} \d{2}:\d{2}:\d{2} \d{4} [-+]{1}\d{4}";
pub const PATH_RE_STR: &str = r###""([^"]+)"|(\S+)"###;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DiffFormat {
    Unified,
    Context,
    GitBinary,
}

pub trait ApplyOffset {
    fn apply_offset(self, offset: i64) -> Self;
}

impl ApplyOffset for usize {
    fn apply_offset(self, offset: i64) -> usize {
        (self as i64 + offset) as usize
    }
}

pub struct MultiListIter<'a, T> {
    iters: Vec<Iter<'a, T>>,
    current_iter: usize,
}

impl<'a, T> MultiListIter<'a, T> {
    pub fn new(iters: Vec<Iter<'a, T>>) -> MultiListIter<T> {
        MultiListIter {
            iters,
            current_iter: 0,
        }
    }

    pub fn push(&mut self, iter: Iter<'a, T>) {
        self.iters.push(iter);
    }

    pub fn prepend(&mut self, iter: Iter<'a, T>) {
        self.iters.insert(self.current_iter, iter);
    }

    pub fn append(&mut self, rhs: &mut MultiListIter<'a, T>) {
        let mut v = rhs.iters[rhs.current_iter..].to_vec();
        self.iters.append(&mut v);
    }
}

impl<'a, T> Iterator for MultiListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_iter < self.iters.len() {
                if let Some(item) = self.iters[self.current_iter].next() {
                    return Some(item);
                }
            } else {
                break;
            };
            self.current_iter += 1
        }
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
