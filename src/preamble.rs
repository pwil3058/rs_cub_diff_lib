//
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
use crate::PATH_RE_STR;
            None => None,
            None => None,
impl PreambleIfce for GitPreamble {
        let diff_cre_str = format!(
            r"^diff\s+--git\s+({})\s+({})(\n)?$",
            PATH_RE_STR, PATH_RE_STR
        );
        ]
        .iter()
        .map(|cre_str| Regex::new(cre_str).unwrap())
        .collect();

        GitPreambleParser {
            diff_cre,
            extras_cres,
        }
            return None;
                        (
                            captures.get(2).unwrap().as_str().to_string(),
                            index - start_index,
                        ),
                    break;
                break;
        Some(GitPreamble {
            extras,
            "index 0000000..0503e55\n",