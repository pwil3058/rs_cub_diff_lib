pub trait PreambleParserIfce<P: PreambleIfce> {
impl PreambleParserIfce<GitPreamble> for GitPreambleParser {
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

    pub fn ante_file_path_buf(&self) -> PathBuf {
        self.ante_file_path.clone().into()
    }

    pub fn post_file_path_buf(&self) -> PathBuf {
        self.post_file_path.clone().into()
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

impl PreambleParserIfce<DiffPreamble> for DiffPreambleParser {
    fn new() -> Self {
        let cre_str = format!(
            r"^diff(\s.+)\s+({0})\s+({1})(\n)?$",
            PATH_RE_STR, PATH_RE_STR
        );
        DiffPreambleParser {
            cre: Regex::new(&cre_str).unwrap(),
        }
    }

    fn get_preamble_at(&self, lines: &Lines, start_index: usize) -> Option<DiffPreamble> {
        let captures = if let Some(captures) = self.cre.captures(&lines[start_index]) {
            captures
        } else {
            return None;
        };
        if let Some(m) = captures.get(1) {
            if m.as_str().find("--git").is_some() {
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
            lines: lines[start_index..start_index + 1].to_vec(),
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

    pub fn iter(&self) -> Iter<Line> {
        match self {
            Preamble::Git(preamble) => preamble.iter(),
            Preamble::Diff(preamble) => preamble.iter(),
        }
    }
}

pub struct PreambleParser {
    git_preamble_parser: GitPreambleParser,
}

impl PreambleParser {
    pub fn new() -> PreambleParser {
        PreambleParser {
            git_preamble_parser: GitPreambleParser::new(),
        }
    }

    pub fn get_preamble_at(&self, lines: &Lines, start_index: usize) -> Option<Preamble> {
        if let Some(preamble) = self.git_preamble_parser.get_preamble_at(lines, start_index) {
            Some(Preamble::Git(preamble))
        } else {
            None
        }
    }
}
