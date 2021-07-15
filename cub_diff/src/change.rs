// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{convert::From, fmt};

use crate::lines::{
    first_inequality_fm_head, first_inequality_fm_tail, CompleteLinesExt, LinesIfce,
};

#[derive(Debug, Clone)]
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
        context: &Context,
    ) -> Option<(Self, Context)> {
        for head_redn in 0..context.head_len {
            for tail_redn in 0..context.tail_len {
                let to_index = self.lines.len() - tail_redn;
                if let Some(found_start_index) =
                    lines.find_first_sub_lines(&self.lines[head_redn..to_index], start_index)
                {
                    let snippet = Self {
                        start_index: found_start_index,
                        lines: self.lines[head_redn..to_index].to_vec(),
                    };
                    let context_redn = Context {
                        head_len: head_redn,
                        tail_len: tail_redn,
                    };
                    return Some((snippet, context_redn));
                }
            }
        }
        None
    }
}

#[derive(Debug, Default, Clone, Copy)]
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
        let head_len = first_inequality_fm_head(lines.0, lines.1).expect("programmer error");
        let tail_len = first_inequality_fm_tail(lines.0, lines.1).expect("programmer error");
        Self { head_len, tail_len }
    }
}
impl From<(&Vec<&str>, &Vec<&str>)> for Context {
    fn from(lines: (&Vec<&str>, &Vec<&str>)) -> Self {
        let head_len = first_inequality_fm_head(&lines.0, &lines.1).expect("programmer error");
        let tail_len = first_inequality_fm_tail(&lines.0, &lines.1).expect("programmer error");
        Self { head_len, tail_len }
    }
}

#[derive(Debug, Clone)]
pub struct Change<'a> {
    ante_snippet: Snippet<'a>,
    post_snippet: Snippet<'a>,
    context: Context,
}

impl<'a> Change<'a> {
    pub fn new(ante_snippet: Snippet<'a>, post_snippet: Snippet<'a>) -> Self {
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
    already_merged: u64,
    failures: u64,
}

pub fn apply_changes(changes: &[Change], text: &str) -> Changed {
    let lines: Vec<&str> = text.complete_lines().collect();
    let mut changed = Changed::default();
    let mut current_offset: isize = 0;
    let mut lines_index: usize = 0;
    for (change_index, change) in changes.iter().enumerate() {
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
            lines_index = index + ante.lines.len();
            changed.successes += 1;
        } else if let Some((reduced_snippet, context_redn)) =
            ante.match_lines_fuzzy(&lines, lines_index, &change.context)
        {
            for line in &lines[lines_index..reduced_snippet.start_index] {
                changed.string += line;
            }
            let end = post.lines.len() - context_redn.tail_len;
            for line in &post.lines[context_redn.head_len..end] {
                changed.string += line;
            }
            lines_index = reduced_snippet.start_index + reduced_snippet.lines.len();
            current_offset = reduced_snippet.start_index as isize - ante.start_index as isize;
            changed.merges += 1;
        } else if post.matches_lines(&lines, current_offset) {
            // already applied
            lines_index += post.lines.len();
            current_offset += post.lines.len() as isize - ante.lines.len() as isize;
            changed.already_applied += 1;
        } else if let Some((reduced_snippet, context_redn)) =
            post.match_lines_fuzzy(&lines, lines_index, &change.context)
        {
            // already merged
            lines_index = reduced_snippet.start_index + reduced_snippet.lines.len();
            current_offset = reduced_snippet.start_index as isize
                - context_redn.head_len as isize
                - post.start_index as isize;
            changed.already_merged += 1;
        } else if lines_index < lines.len() {
            // TODO: this needs to be smarter
            changed.string += "<<<<<<<\n";
            for line in &ante.lines {
                changed.string += line;
            }
            changed.string += "=======\n";
            for line in &post.lines {
                changed.string += line;
            }
            changed.string += ">>>>>>>\n";
            changed.failures += 1;
        } else {
            // unexpected end of file
            changed.failures += (changes.len() - change_index) as u64;
            break;
        }
    }
    for line in lines[lines_index..].iter() {
        changed.string += line;
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

#[cfg(test)]
mod test {
    use crate::change::{apply_changes, Change, Snippet};
    use crate::lines::CompleteLinesExt;

    #[test]
    fn test_change_application() {
        let ante_text =
            "zero\none\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\nten\neleven\ntwelve"
                .to_string();
        let ante_lines: Vec<&str> = ante_text.complete_lines().collect();
        let post_text =
            "zero\none\ntwo\nthree mod\nfour\nfive\nextra\nsix\nseven\neight mod\nnine mod\nten\neleven\ntwelve\nextra\n".to_string();
        let post_lines: Vec<&str> = post_text.complete_lines().collect();
        let ante_snippets = vec![
            Snippet {
                start_index: 2,
                lines: ante_lines[2..5].to_vec(),
            },
            Snippet {
                start_index: 5,
                lines: ante_lines[5..7].to_vec(),
            },
            Snippet {
                start_index: 7,
                lines: ante_lines[7..11].to_vec(),
            },
            Snippet {
                start_index: 12,
                lines: ante_lines[12..].to_vec(),
            },
        ];
        println!("ante snippets: {:?}", ante_snippets);
        let post_snippets = vec![
            Snippet {
                start_index: 2,
                lines: post_lines[2..5].to_vec(),
            },
            Snippet {
                start_index: 5,
                lines: post_lines[5..8].to_vec(),
            },
            Snippet {
                start_index: 8,
                lines: post_lines[8..12].to_vec(),
            },
            Snippet {
                start_index: 13,
                lines: post_lines[13..].to_vec(),
            },
        ];
        println!("post snippets: {:?}", post_snippets);
        let mut changes: Vec<Change> = vec![];
        for (a, p) in ante_snippets.iter().zip(post_snippets.iter()) {
            let change = Change::new(a.clone(), p.clone());
            changes.push(change);
        }
        let changed = apply_changes(&changes, &ante_text);
        assert_eq!(changed.string, post_text);
        let changed = apply_changes(
            &changes,
            &"zero\none\ntwo\nthree\nfour\nmove\nfive\nsix\nseven\neight\nnine\nten\neleven\ntwelve",
        );
        assert_eq!(changed.string, "zero\none\ntwo\nthree mod\nfour\nmove\nfive\nextra\nsix\nseven\neight mod\nnine mod\nten\neleven\ntwelve\nextra\n");
        //assert!(false);
    }

    #[test]
    fn test_change_application_with_tail() {
        let ante_text =
            "zero\none\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\nten\neleven\ntwelve\nthirteen\nfourteen\n"
                .to_string();
        let ante_lines: Vec<&str> = ante_text.complete_lines().collect();
        let post_text =
            "zero\none\ntwo\nthree mod\nfour\nfive\nextra\nsix\nseven\neight mod\nnine mod\nten\neleven\ntwelve\nextra\nthirteen\nfourteen\n".to_string();
        let post_lines: Vec<&str> = post_text.complete_lines().collect();
        let ante_snippets = vec![
            Snippet {
                start_index: 2,
                lines: ante_lines[2..5].to_vec(),
            },
            Snippet {
                start_index: 5,
                lines: ante_lines[5..7].to_vec(),
            },
            Snippet {
                start_index: 7,
                lines: ante_lines[7..11].to_vec(),
            },
            Snippet {
                start_index: 12,
                lines: ante_lines[12..14].to_vec(),
            },
        ];
        println!("ante snippets: {:?}", ante_snippets);
        let post_snippets = vec![
            Snippet {
                start_index: 2,
                lines: post_lines[2..5].to_vec(),
            },
            Snippet {
                start_index: 5,
                lines: post_lines[5..8].to_vec(),
            },
            Snippet {
                start_index: 8,
                lines: post_lines[8..12].to_vec(),
            },
            Snippet {
                start_index: 13,
                lines: post_lines[13..16].to_vec(),
            },
        ];
        println!("post snippets: {:?}", post_snippets);
        let mut changes: Vec<Change> = vec![];
        for (a, p) in ante_snippets.iter().zip(post_snippets.iter()) {
            let change = Change::new(a.clone(), p.clone());
            changes.push(change);
        }
        let changed = apply_changes(&changes, &ante_text);
        assert_eq!(changed.string, post_text);
        let changed = apply_changes(
            &changes,
            &"zero\none\ntwo\nthree\nfour\nmove\nfive\nsix\nseven\neight\nnine\nten\neleven\ntwelve\nthirteen\nfourteen\n",
        );
        assert_eq!(changed.string, "zero\none\ntwo\nthree mod\nfour\nmove\nfive\nextra\nsix\nseven\neight mod\nnine mod\nten\neleven\ntwelve\nextra\nthirteen\nfourteen\n");
        //assert!(false);
    }
}
