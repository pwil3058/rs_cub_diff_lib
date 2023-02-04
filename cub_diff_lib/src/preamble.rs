//Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::collections::{hash_map, HashMap};
use std::slice::Iter;

use regex::Regex;

use pw_pathux::str_path::*;

use crate::lines::{Line, Lines};
use crate::PATH_RE_STR;

pub trait PreambleIfce {
    fn len(&self) -> usize;
    fn iter(&self) -> Iter<Line>;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait PreambleParserIfce<P: PreambleIfce> {
    fn new() -> Self;
    fn get_preamble_at(&self, lines: &[Line], start_index: usize) -> Option<P>;
}

pub struct GitPreamble {
    lines: Lines,
    ante_file_path: String,
    post_file_path: String,
    extras: HashMap<String, (String, usize)>,
}

// TODO: should we be returning Path or &Path instead of PathBuf
impl GitPreamble {
    pub fn ante_file_path_as_str(&self) -> &str {
        self.ante_file_path.as_str()
    }

    pub fn post_file_path_as_str(&self) -> &str {
        self.post_file_path.as_str()
    }

    pub fn get_ante_file_path(&self, strip_level: usize) -> String {
        self.ante_file_path.path_stripped_of_n_levels(strip_level)
    }

    pub fn get_post_file_path(&self, strip_level: usize) -> String {
        self.post_file_path.path_stripped_of_n_levels(strip_level)
    }

    pub fn get_file_path(&self, strip_level: usize) -> String {
        if self.post_file_path == "/dev/null" {
            self.ante_file_path.path_stripped_of_n_levels(strip_level)
        } else {
            self.post_file_path.path_stripped_of_n_levels(strip_level)
        }
    }

    pub fn iter_extras(&self) -> hash_map::Iter<String, (String, usize)> {
        self.extras.iter()
    }

    pub fn get_extra(&self, name: &str) -> Option<&str> {
        match self.extras.get(name) {
            Some(extra) => Some(&extra.0),
            None => None,
        }
    }

    pub fn get_extra_line_index(&self, name: &str) -> Option<usize> {
        self.extras.get(name).map(|extra| extra.1)
    }
}

impl PreambleIfce for GitPreamble {
    fn len(&self) -> usize {
        self.lines.len()
    }

    fn iter(&self) -> Iter<Line> {
        self.lines.iter()
    }
}

pub struct GitPreambleParser {
    diff_cre: Regex,
    extras_cres: Vec<Regex>,
}

impl Default for GitPreambleParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PreambleParserIfce<GitPreamble> for GitPreambleParser {
    fn new() -> GitPreambleParser {
        let diff_cre_str = format!(
            r"^diff\s+--git\s+({PATH_RE_STR})\s+({PATH_RE_STR})(\n)?$"
        );
        let diff_cre = Regex::new(&diff_cre_str).unwrap();

        let extras_cres = [
            r"^(old mode)\s+(\d*)(\n)?$",
            r"^(new mode)\s+(\d*)(\n)?$",
            r"^(deleted file mode)\s+(\d*)(\n)?$",
            r"^(new file mode)\s+(\d*)(\n)?$",
            r"^(similarity index)\s+((\d*)%)(\n)?$",
            r"^(dissimilarity index)\s+((\d*)%)(\n)?$",
            r"^(index)\s+(([a-fA-F0-9]+)..([a-fA-F0-9]+)( (\d*))?)(\n)?$",
            &format!(r"^(copy from)\s+({PATH_RE_STR})(\n)?$"),
            &format!(r"^(copy to)\s+({PATH_RE_STR})(\n)?$"),
            &format!(r"^(rename from)\s+({PATH_RE_STR})(\n)?$"),
            &format!(r"^(rename to)\s+({PATH_RE_STR})(\n)?$"),
        ]
        .iter()
        .map(|cre_str| Regex::new(cre_str).unwrap())
        .collect();

        GitPreambleParser {
            diff_cre,
            extras_cres,
        }
    }

    fn get_preamble_at(&self, lines: &[Line], start_index: usize) -> Option<GitPreamble> {
        let captures = self.diff_cre.captures(&lines[start_index])?;
        let ante_file_path = if let Some(path) = captures.get(3) {
            path.as_str().to_string()
        } else {
            captures.get(4).unwrap().as_str().to_string() // TODO: confirm unwrap is OK here
        };
        let post_file_path = if let Some(path) = captures.get(6) {
            path.as_str().to_string()
        } else {
            captures.get(7).unwrap().as_str().to_string() // TODO: confirm unwrap is OK here
        };

        let mut extras: HashMap<String, (String, usize)> = HashMap::new();
        for (index, line) in lines.iter().enumerate().skip(start_index + 1) {
            let mut found = false;
            for cre in self.extras_cres.iter() {
                if let Some(captures) = cre.captures(line) {
                    extras.insert(
                        captures.get(1).unwrap().as_str().to_string(),
                        (
                            captures.get(2).unwrap().as_str().to_string(),
                            index - start_index,
                        ),
                    );
                    found = true;
                    break;
                };
            }
            if !found {
                break;
            }
        }
        Some(GitPreamble {
            lines: lines[start_index..=start_index + extras.len()].to_vec(),
            ante_file_path,
            post_file_path,
            extras,
        })
    }
}

pub struct DiffPreamble {
    lines: Lines,
    ante_file_path: String,
    post_file_path: String,
}

impl DiffPreamble {
    pub fn ante_file_path_as_str(&self) -> &str {
        self.ante_file_path.as_str()
    }

    pub fn post_file_path_as_str(&self) -> &str {
        self.post_file_path.as_str()
    }

    pub fn get_ante_file_path(&self, strip_level: usize) -> String {
        self.ante_file_path.path_stripped_of_n_levels(strip_level)
    }

    pub fn get_post_file_path(&self, strip_level: usize) -> String {
        self.post_file_path.path_stripped_of_n_levels(strip_level)
    }

    pub fn get_file_path(&self, strip_level: usize) -> String {
        if self.post_file_path == "/dev/null" {
            self.ante_file_path.path_stripped_of_n_levels(strip_level)
        } else {
            self.post_file_path.path_stripped_of_n_levels(strip_level)
        }
    }
}

impl PreambleIfce for DiffPreamble {
    fn len(&self) -> usize {
        self.lines.len()
    }

    fn iter(&self) -> Iter<Line> {
        self.lines.iter()
    }
}

pub struct DiffPreambleParser {
    cre: Regex,
}

impl Default for DiffPreambleParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PreambleParserIfce<DiffPreamble> for DiffPreambleParser {
    fn new() -> Self {
        let cre_str = format!(
            r"^diff(\s.+)\s+({PATH_RE_STR})\s+({PATH_RE_STR})(\n)?$"
        );
        DiffPreambleParser {
            cre: Regex::new(&cre_str).unwrap(),
        }
    }

    fn get_preamble_at(&self, lines: &[Line], start_index: usize) -> Option<DiffPreamble> {
        let captures = self.cre.captures(&lines[start_index])?;
        if let Some(m) = captures.get(1) {
            if m.as_str().contains("--git") {
                return None;
            }
        }
        let ante_file_path = if let Some(path) = captures.get(3) {
            path.as_str().to_string()
        } else {
            captures.get(4).unwrap().as_str().to_string() // TODO: confirm unwrap is OK here
        };
        let post_file_path = if let Some(path) = captures.get(6) {
            path.as_str().to_string()
        } else {
            captures.get(7).unwrap().as_str().to_string() // TODO: confirm unwrap is OK here
        };
        Some(DiffPreamble {
            lines: lines[start_index..=start_index].to_vec(),
            ante_file_path,
            post_file_path,
        })
    }
}

pub enum Preamble {
    Git(GitPreamble),
    Diff(DiffPreamble),
}

impl Preamble {
    pub fn len(&self) -> usize {
        match self {
            Preamble::Git(preamble) => preamble.len(),
            Preamble::Diff(preamble) => preamble.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> Iter<Line> {
        match self {
            Preamble::Git(preamble) => preamble.iter(),
            Preamble::Diff(preamble) => preamble.iter(),
        }
    }

    pub fn get_ante_file_path(&self, strip_level: usize) -> String {
        match self {
            Preamble::Git(preamble) => preamble.get_ante_file_path(strip_level),
            Preamble::Diff(preamble) => preamble.get_ante_file_path(strip_level),
        }
    }

    pub fn get_post_file_path(&self, strip_level: usize) -> String {
        match self {
            Preamble::Git(preamble) => preamble.get_post_file_path(strip_level),
            Preamble::Diff(preamble) => preamble.get_post_file_path(strip_level),
        }
    }

    pub fn get_file_path(&self, strip_level: usize) -> String {
        match self {
            Preamble::Git(preamble) => preamble.get_file_path(strip_level),
            Preamble::Diff(preamble) => preamble.get_file_path(strip_level),
        }
    }
}

#[derive(Default)]
pub struct PreambleParser {
    git_preamble_parser: GitPreambleParser,
    diff_preamble_parser: DiffPreambleParser,
}

impl PreambleParser {
    pub fn new() -> PreambleParser {
        PreambleParser {
            git_preamble_parser: GitPreambleParser::new(),
            diff_preamble_parser: DiffPreambleParser::new(),
        }
    }

    pub fn get_preamble_at(&self, lines: &[Line], start_index: usize) -> Option<Preamble> {
        if let Some(preamble) = self.git_preamble_parser.get_preamble_at(lines, start_index) {
            Some(Preamble::Git(preamble))
        } else { self
            .diff_preamble_parser
            .get_preamble_at(lines, start_index).map(Preamble::Diff)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn it_works() {
        let mut lines: Lines = Vec::new();
        for s in &[
            "diff --git a/src/preamble.rs b/src/preamble.rs\n",
            "new file mode 100644\n",
            "index 0000000..0503e55\n",
        ] {
            lines.push(Arc::new(s.to_string()))
        }

        let parser = GitPreambleParser::new();

        let preamble = parser.get_preamble_at(&lines, 0);
        assert!(preamble.is_some());
        let preamble = preamble.unwrap();
        assert!(preamble.get_extra_line_index("index") == Some(2));
    }
}
